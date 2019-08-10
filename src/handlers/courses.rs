use crate::models;
use crate::models::WorksheetsInCourse;
use crate::schema;
use crate::AppData;
use actix_web::{web, Error, HttpRequest, HttpResponse, Scope};
use diesel::prelude::*;
use futures::future::{Future, IntoFuture};
use uuid::Uuid;

pub fn get_scope(auth: actix_web_jwt_middleware::JwtAuthentication) -> Scope {
    web::scope("/courses")
        .service(
            web::resource("")
                .wrap(auth.clone())
                .route(web::get().to_async(get_courses))
                .route(web::post().to_async(create_course)),
        )
        .service(
            web::resource("/{id}")
                .wrap(auth.clone())
                .route(web::put().to_async(update_course))
                .route(web::delete().to_async(delete_course)),
        )
        .service(web::resource("/{id}").route(web::get().to_async(get_course)))
}

fn get_courses(req: HttpRequest) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let appdata: &AppData = req.app_data().unwrap();

    let conn = match appdata.get_db_connection() {
        Ok(connection) => connection,
        Err(_) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        }
    };

    let query = schema::courses::table
        .inner_join(
            schema::access::table
                .on(schema::courses::columns::id.eq(schema::access::columns::object_id)),
        )
        .filter(schema::access::columns::user_id.eq(appdata.current_user.to_string()))
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
                    worksheets: worksheets_query.ok(),
                });
            }
            Box::new(Ok(HttpResponse::Ok().json(courses)).into_future())
        }
        Err(e) => Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future()),
    }
}

fn create_course(
    req: HttpRequest,
    json: web::Json<models::Course>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let appdata: &AppData = req.app_data().unwrap();

    let conn = match appdata.get_db_connection() {
        Ok(connection) => connection,
        Err(_) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        }
    };

    // create course object
    let course = json.into_inner();
    let course_id = Uuid::new_v4();
    let new_course = models::QueryableCourse {
        id: course_id.to_string(),
        name: course.name,
        description: course.description,
    };

    // insert access for user
    match diesel::insert_into(schema::access::table)
        .values(models::Access {
            user_id: appdata.current_user.to_string(),
            object_id: course_id.to_string(),
        })
        .execute(&*conn)
    {
        Ok(result) => {}
        Err(e) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        }
    }

    // set worksheets belonging to course
    for (position, worksheet) in course.worksheets.unwrap().iter().enumerate() {
        match diesel::insert_into(schema::worksheets_in_courses::table)
            .values(models::WorksheetsInCourse {
                worksheet_id: worksheet.to_string(),
                course_id: course_id.to_string(),
                position: position as i32,
            })
            .execute(&*conn)
        {
            Ok(result) => {}
            Err(e) => {
                return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
            }
        }
    }

    // insert course object
    match diesel::insert_into(schema::courses::table)
        .values(new_course)
        .execute(&*conn)
    {
        Ok(result) => Box::new(Ok(HttpResponse::Ok().json(course_id)).into_future()),
        Err(e) => Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future()),
    }
}

fn get_course(
    req: HttpRequest,
    id: web::Path<Uuid>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let appdata: &AppData = req.app_data().unwrap();

    let conn = match appdata.get_db_connection() {
        Ok(connection) => connection,
        Err(_) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        }
    };

    let query = schema::courses::table
        .find(format!("{}", id))
        .get_result::<models::QueryableCourse>(&*conn);

    match query {
        Ok(course) => {
            let worksheets_query = schema::worksheets_in_courses::table
                .filter(schema::worksheets_in_courses::columns::course_id.eq(format!("{}", id)))
                .select((schema::worksheets_in_courses::columns::worksheet_id))
                .order(schema::worksheets_in_courses::position)
                .load::<String>(&*conn);

            Box::new(
                Ok(HttpResponse::Ok().json(models::Course {
                    id: course.id,
                    name: course.name,
                    description: course.description,
                    worksheets: worksheets_query.ok(),
                }))
                .into_future(),
            )
        }
        Err(e) => match e {
            diesel::result::Error::NotFound => {
                Box::new(Ok(HttpResponse::NotFound().finish()).into_future())
            }
            _ => Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future()),
        },
    }
}

fn update_course(
    req: HttpRequest,
    id: web::Path<Uuid>,
    json: web::Json<models::Course>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let appdata: &AppData = req.app_data().unwrap();

    let conn = match appdata.get_db_connection() {
        Ok(connection) => connection,
        Err(_) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        }
    };

    let course = json.into_inner();

    // update course
    let query = diesel::update(schema::courses::table.find(format!("{}", id)))
        .set(models::QueryableCourse::from_course(course.clone()))
        .execute(&*conn);
    match query {
        Ok(result) => {}
        Err(e) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        }
    }

    // update which worksheets belong to course
    match diesel::delete(
        schema::worksheets_in_courses::table
            .filter(schema::worksheets_in_courses::course_id.eq(course.id.clone())),
    )
    .execute(&*conn)
    {
        Ok(result) => {
            let mut pos = -1;
            let course_id = course.id.clone();
            let worksheets_in_course: Vec<WorksheetsInCourse> = course
                .worksheets
                .unwrap()
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
            match diesel::insert_into(schema::worksheets_in_courses::table)
                .values(worksheets_in_course)
                .execute(&*conn)
            {
                Ok(result) => {
                    return Box::new(Ok(HttpResponse::Ok().finish()).into_future());
                }
                Err(e) => {
                    return Box::new(
                        Ok(HttpResponse::InternalServerError().finish()).into_future(),
                    );
                }
            }
        }
        Err(e) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        }
    }
}

fn delete_course(
    req: HttpRequest,
    id: web::Path<Uuid>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let appdata: &AppData = req.app_data().unwrap();

    let conn = match appdata.get_db_connection() {
        Ok(connection) => connection,
        Err(_) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        }
    };

    let uuid = id.into_inner();

    match diesel::delete(
        schema::worksheets_in_courses::table
            .filter(schema::worksheets_in_courses::course_id.eq(uuid.to_string())),
    )
    .execute(&*conn)
    {
        Ok(result) => {}
        Err(e) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        }
    }

    let query = diesel::delete(schema::courses::table.find(format!("{}", uuid))).execute(&*conn);
    match query {
        Ok(result) => Box::new(Ok(HttpResponse::Ok().finish()).into_future()),
        Err(e) => Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future()),
    }
}
