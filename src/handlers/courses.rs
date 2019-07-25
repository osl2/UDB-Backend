use futures::future::{Future, IntoFuture};
use uuid::Uuid;
use actix_web::{
    get, put, post, delete, web, Error, HttpRequest, HttpResponse, Scope
};
use crate::AppData;
use crate::schema;
use crate::models;
use diesel::prelude::*;

pub fn get_scope() -> Scope {
    web::scope("/courses")
    .service(get_courses)
    .service(create_course)
    .service(get_course)
    .service(update_course)
    .service(delete_course)
}

#[get("")]
fn get_courses(req: HttpRequest) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let appdata: &AppData = req.app_data().unwrap();

    let conn = match appdata.get_db_connection(){
        Ok(connection) => connection,
        Err(_) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        },
    };

    let query = schema::courses::table.inner_join(schema::access::table.on(schema::courses::columns::id.eq(schema::access::columns::object_id)))
        .filter(schema::access::columns::user_id.eq(appdata.current_user.to_string()))
        .select((schema::courses::columns::id, schema::courses::columns::name, schema::courses::columns::description))
        .load::<models::QueryableCourse>(&*conn);

    match query {
        Ok(query_courses) => {
            let mut courses: Vec<models::Course> = Vec::new();
            for course in query_courses {
                let worksheets_query = schema::worksheets_in_courses::table
                    .filter(schema::worksheets_in_courses::columns::course_id.eq(&course.id))
                    .select((schema::worksheets_in_courses::columns::worksheet_id))
                    .load::<String>(&*conn);
                courses.push(models::Course {
                    id: course.id,
                    name: course.name,
                    description: course.description,
                    worksheets: worksheets_query.ok(),
                });
            }
            Box::new(Ok(HttpResponse::Ok().json(courses)).into_future())
        },
        Err(e) => {
            Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future())
        }
    }
}
#[post("")]
fn create_course(req: HttpRequest, json: web::Json<models::Course>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    Box::new(Ok(HttpResponse::NotImplemented().finish()).into_future())
}
#[get("/{id}")]
fn get_course(req: HttpRequest, id: web::Path<Uuid>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let appdata: &AppData = req.app_data().unwrap();

    let conn = match appdata.get_db_connection(){
        Ok(connection) => connection,
        Err(_) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        },
    };

    let query = schema::courses::table.find(format!("{}", id)).get_result::<models::QueryableCourse>(&*conn);

    match query {
        Ok(course) => {
            let worksheets_query = schema::worksheets_in_courses::table
                .filter(schema::worksheets_in_courses::columns::course_id.eq(format!("{}", id)))
                .select((schema::worksheets_in_courses::columns::worksheet_id))
                .load::<String>(&*conn);

            Box::new(Ok(HttpResponse::Ok().json(models::Course {
                id: course.id,
                name: course.name,
                description: course.description,
                worksheets: worksheets_query.ok(),
            })).into_future())
        },
        Err(e) => {
            match e {
                diesel::result::Error::NotFound => Box::new(Ok(HttpResponse::NotFound().finish()).into_future()),
                _ => Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future()),
            }
        }
    }
}
#[put("/{id}")]
fn update_course(req: HttpRequest, id: web::Path<Uuid>, json: web::Json<models::Course>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    Box::new(Ok(HttpResponse::NotImplemented().finish()).into_future())
}
#[delete("/{id}")]
fn delete_course(req: HttpRequest, id: web::Path<Uuid>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let appdata: &AppData = req.app_data().unwrap();

    let conn = match appdata.get_db_connection(){
        Ok(connection) => connection,
        Err(_) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        },
    };

    let query = diesel::delete(schema::courses::table.find(format!("{}", id.into_inner())))
        .execute(&*conn);
    match query {
        Ok(result) => {
            Box::new(Ok(HttpResponse::Ok().finish()).into_future())
        },
        Err(e) => {
            Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future())
        }
    }
}