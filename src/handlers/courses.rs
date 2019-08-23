use crate::{models::{self, WorksheetsInCourse}, schema, database::DatabaseConnection};
use actix_web::{web, Error, HttpRequest, HttpResponse, Scope};
use diesel::{
    prelude::*,
    r2d2::{self, ConnectionManager},
};
use futures::future::{Future, IntoFuture};
use uuid::Uuid;

pub fn get_scope() -> Scope {
    web::scope("/courses")
        .service(
            web::resource("")
                .route(web::get().to_async(get_courses))
                .route(web::post().to_async(create_course)),
        )
        .service(
            web::resource("/{id}")
                .route(web::get().to_async(get_course))
                .route(web::put().to_async(update_course))
                .route(web::delete().to_async(delete_course)),
        )
}

fn get_courses(req: HttpRequest) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let extensions = req.extensions();
    let conn = extensions
        .get::<r2d2::PooledConnection<ConnectionManager<DatabaseConnection>>>()
        .unwrap();
    let current_user = extensions
        .get::<actix_web_jwt_middleware::AuthenticationData>()
        .unwrap()
        .claims
        .sub
        .clone()
        .unwrap();

    let query = schema::courses::table
        .inner_join(
            schema::access::table
                .on(schema::courses::columns::id.eq(schema::access::columns::object_id)),
        )
        .filter(schema::access::columns::user_id.eq(current_user))
        .select((
            schema::courses::columns::id,
            schema::courses::columns::name,
            schema::courses::columns::description,
        ))
        .load::<models::QueryableCourse>(&*conn);

    match query {
        Ok(query_courses) => {
            let mut courses: Vec<models::Course> = Vec::new();
            for course in query_courses {
                let worksheets_query = schema::worksheets_in_courses::table
                    .filter(schema::worksheets_in_courses::columns::course_id.eq(&course.id))
                    .select(schema::worksheets_in_courses::columns::worksheet_id)
                    .order(schema::worksheets_in_courses::position)
                    .load::<String>(&*conn);
                courses.push(models::Course {
                    id: course.id,
                    name: course.name,
                    description: course.description,
                    worksheets: worksheets_query.unwrap(),
                });
            }
            Box::new(Ok(HttpResponse::Ok().json(courses)).into_future())
        }
        Err(e) => {
            log::error!("Couldn't get courses: {:?}", e);
            Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future())
        }
    }
}

fn create_course(
    req: HttpRequest,
    json: web::Json<models::Course>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let extensions = req.extensions();
    let conn = extensions
        .get::<r2d2::PooledConnection<ConnectionManager<DatabaseConnection>>>()
        .unwrap();
    let sub = extensions
        .get::<actix_web_jwt_middleware::AuthenticationData>()
        .unwrap()
        .claims
        .sub
        .clone()
        .unwrap();

    match conn.transaction::<Uuid, diesel::result::Error, _>(|| {
        // create course object
        let course = json.into_inner();
        let course_id = Uuid::new_v4();
        let new_course = models::QueryableCourse {
            id: course_id.to_string(),
            name: course.name,
            description: course.description,
        };

        // insert access for user
        diesel::insert_into(schema::access::table)
            .values(models::Access {
                user_id: sub,
                object_id: course_id.to_string(),
            })
            .execute(&*conn)?;

        // set worksheets belonging to course
        for (position, worksheet) in course.worksheets.iter().enumerate() {
            diesel::insert_into(schema::worksheets_in_courses::table)
                .values(models::WorksheetsInCourse {
                    worksheet_id: worksheet.to_string(),
                    course_id: course_id.to_string(),
                    position: position as i32,
                })
                .execute(&*conn)?;
        }

        // insert course object
        diesel::insert_into(schema::courses::table)
            .values(new_course)
            .execute(&*conn)?;

        Ok(course_id)
    }) {
        Ok(course_id) => Box::new(Ok(HttpResponse::Ok().body(course_id.to_string())).into_future()),
        Err(e) => {
            log::error!("Could not create course: {:?}", e);
            Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future())
        }
    }
}

fn get_course(
    req: HttpRequest,
    id: web::Path<Uuid>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let extensions = req.extensions();
    let conn = extensions
        .get::<r2d2::PooledConnection<ConnectionManager<DatabaseConnection>>>()
        .unwrap();

    match schema::courses::table
        .find(format!("{}", id))
        .get_result::<models::QueryableCourse>(&*conn)
    {
        Ok(course) => {
            let worksheets_query = schema::worksheets_in_courses::table
                .filter(schema::worksheets_in_courses::columns::course_id.eq(format!("{}", id)))
                .select(schema::worksheets_in_courses::columns::worksheet_id)
                .order(schema::worksheets_in_courses::position)
                .load::<String>(&*conn);

            Box::new(
                Ok(HttpResponse::Ok().json(models::Course {
                    id: course.id,
                    name: course.name,
                    description: course.description,
                    worksheets: worksheets_query.unwrap(),
                }))
                .into_future(),
            )
        }
        Err(e) => match e {
            diesel::result::Error::NotFound => {
                Box::new(Ok(HttpResponse::NotFound().finish()).into_future())
            }
            e => {
                log::error!("Couldn't get course: {:?}", e);
                Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future())
            }
        },
    }
}

fn update_course(
    req: HttpRequest,
    id: web::Path<Uuid>,
    json: web::Json<models::Course>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let extensions = req.extensions();
    let conn = extensions
        .get::<r2d2::PooledConnection<ConnectionManager<DatabaseConnection>>>()
        .unwrap();

    let course = json.into_inner();
    let id = format!("{}", id.into_inner());
    match conn.transaction::<(), diesel::result::Error, _>(|| {
        // update course
        diesel::update(schema::courses::table.filter(schema::courses::id.eq(id)))
            .set(models::QueryableCourse::from_course(course.clone()))
            .execute(&*conn)?;

        // update which worksheets belong to course
        // first delete old ones
        diesel::delete(
            schema::worksheets_in_courses::table
                .filter(schema::worksheets_in_courses::course_id.eq(course.id.clone())),
        )
        .execute(&*conn)?;
        // then list new ones
        let mut pos = -1;
        let course_id = course.id.clone();
        let worksheets_in_course: Vec<WorksheetsInCourse> = course
            .worksheets
            .iter()
            .map(|worksheet_id| {
                pos += 1;
                models::WorksheetsInCourse {
                    worksheet_id: worksheet_id.to_string(),
                    course_id: course_id.clone(),
                    position: pos,
                }
            })
            .collect();
        // finally insert them into the table
        for sheet in worksheets_in_course {
            diesel::insert_into(schema::worksheets_in_courses::table)
                .values(sheet)
                .execute(&*conn)?;
        }
        Ok(())
    }) {
        Ok(_) => Box::new(Ok(HttpResponse::Ok().finish()).into_future()),
        Err(e) => {
            log::error!("Couldn't update course: {:?}", e);
            Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future())
        }
    }
}

fn delete_course(
    req: HttpRequest,
    id: web::Path<Uuid>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let extensions = req.extensions();
    let conn = extensions
        .get::<r2d2::PooledConnection<ConnectionManager<DatabaseConnection>>>()
        .unwrap();

    let uuid = id.into_inner();

    match conn.transaction::<(), diesel::result::Error, _>(|| {
        diesel::delete(
            schema::access::table.filter(schema::access::object_id.eq(uuid.to_string())),
        )
        .execute(&*conn)?;

        diesel::delete(
            schema::worksheets_in_courses::table
                .filter(schema::worksheets_in_courses::course_id.eq(uuid.to_string())),
        )
        .execute(&*conn)?;

        diesel::delete(schema::courses::table.find(format!("{}", uuid))).execute(&*conn)?;
        Ok(())
    }) {
        Ok(_) => Box::new(Ok(HttpResponse::Ok().finish()).into_future()),
        Err(e) => {
            log::error!("Couldn't delete course: {:?}", e);
            Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future())
        }
    }
}
