use crate::{
    models,
    util::{database, internal_server_error, user},
};
use actix_web::{web, Error, HttpRequest, HttpResponse, Scope};
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
    match database(&req).get_courses(user(&req)) {
        Ok(courses) => Box::new(Ok(HttpResponse::Ok().json(courses)).into_future()),
        Err(e) => Box::new(
            Ok(internal_server_error(format!(
                "Couldn't get courses: {:?}",
                e
            )))
            .into_future(),
        ),
    }
}

fn create_course(
    req: HttpRequest,
    json: web::Json<models::Course>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    match database(&req).create_course(json.into_inner(), user(&req)) {
        Ok(course_id) => {
            Box::new(Ok(HttpResponse::Created().body(course_id.to_string())).into_future())
        }
        Err(e) => Box::new(
            Ok(internal_server_error(format!(
                "Could not create course: {:?}",
                e
            )))
            .into_future(),
        ),
    }
}

fn get_course(
    req: HttpRequest,
    id: web::Path<Uuid>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    match database(&req).get_course(id.into_inner()) {
        Ok(course) => Box::new(Ok(HttpResponse::Ok().json(course)).into_future()),
        Err(e) => match e {
            crate::database::DatabaseError::Diesel(diesel::result::Error::NotFound) => {
                Box::new(Ok(HttpResponse::NotFound().finish()).into_future())
            }
            e => Box::new(
                Ok(internal_server_error(format!(
                    "Couldn't get course: {:?}",
                    e
                )))
                .into_future(),
            ),
        },
    }
}

fn update_course(
    req: HttpRequest,
    id: web::Path<Uuid>,
    json: web::Json<models::Course>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    match database(&req).update_course(id.into_inner(), json.into_inner(), user(&req)) {
        Ok(_) => Box::new(Ok(HttpResponse::Ok().finish()).into_future()),
        Err(e) => match e {
            crate::database::DatabaseError::Diesel(diesel::result::Error::NotFound) => {
                Box::new(Ok(HttpResponse::NotFound().finish()).into_future())
            }
            e => Box::new(
                Ok(internal_server_error(format!(
                    "Couldn't update course: {:?}",
                    e
                )))
                .into_future(),
            ),
        },
    }
}

fn delete_course(
    req: HttpRequest,
    id: web::Path<Uuid>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    match database(&req).delete_course(id.into_inner(), user(&req)) {
        Ok(_) => Box::new(Ok(HttpResponse::Ok().finish()).into_future()),
        Err(e) => match e {
            crate::database::DatabaseError::Diesel(diesel::result::Error::NotFound) => {
                Box::new(Ok(HttpResponse::NotFound().finish()).into_future())
            }
            e => Box::new(
                Ok(internal_server_error(format!(
                    "Couldn't delete course: {:?}",
                    e
                )))
                .into_future(),
            ),
        },
    }
}
