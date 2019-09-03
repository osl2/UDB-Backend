use crate::{
    models,
    util::{database, internal_server_error, user},
};
use actix_web::{web, Error, HttpRequest, HttpResponse, Scope};

use futures::future::{Future, IntoFuture};
use uuid::Uuid;

pub fn get_scope() -> Scope {
    web::scope("/tasks")
        .service(
            web::resource("")
                .route(web::get().to_async(get_tasks))
                .route(web::post().to_async(create_task)),
        )
        .service(
            web::resource("/{id}")
                .route(web::get().to_async(get_task))
                .route(web::put().to_async(update_task))
                .route(web::delete().to_async(delete_task)),
        )
}

fn get_tasks(req: HttpRequest) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    match database(&req).get_tasks(user(&req)) {
        Ok(tasks) => Box::new(Ok(HttpResponse::Ok().json(tasks)).into_future()),
        Err(e) => {
            Box::new(internal_server_error(format!("Couldn't get tasks: {:?}", e)).into_future())
        }
    }
}

fn create_task(
    req: HttpRequest,
    json: web::Json<models::Task>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    match database(&req).create_task(user(&req), json.into_inner()) {
        Ok(id) => Box::new(Ok(HttpResponse::Created().body(id.to_string())).into_future()),
        Err(e) => Box::new(
            Ok(internal_server_error(format!(
                "Couldn't create task: {:?}",
                e
            )))
            .into_future(),
        ),
    }
}

fn get_task(
    req: HttpRequest,
    id: web::Path<Uuid>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    match database(&req).get_task(id.into_inner()) {
        Ok(task) => Box::new(Ok(HttpResponse::Ok().json(task)).into_future()),
        Err(e) => match e {
            crate::database::DatabaseError::Diesel(diesel::result::Error::NotFound) => {
                Box::new(Ok(HttpResponse::NotFound().finish()).into_future())
            }
            e => Box::new(
                Ok(internal_server_error(format!(
                    "Couldn't load task: {:?}",
                    e
                )))
                .into_future(),
            ),
        },
    }
}

fn update_task(
    req: HttpRequest,
    id: web::Path<Uuid>,
    json: web::Json<models::Task>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    match database(&req).update_task(id.into_inner(), json.into_inner(), user(&req)) {
        Ok(_) => Box::new(Ok(HttpResponse::Ok().finish()).into_future()),
        Err(error) => match error {
            crate::database::DatabaseError::Diesel(diesel::result::Error::NotFound) => {
                Box::new(Ok(HttpResponse::NotFound().finish()).into_future())
            }
            error => Box::new(
                Ok(internal_server_error(format!(
                    "Couldn't update task: {:?}",
                    error
                )))
                .into_future(),
            ),
        },
    }
}

fn delete_task(
    req: HttpRequest,
    id: web::Path<Uuid>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    match database(&req).delete_task(id.into_inner(), user(&req)) {
        Ok(_) => Box::new(Ok(HttpResponse::Ok().finish()).into_future()),
        Err(error) => match error {
            crate::database::DatabaseError::Diesel(diesel::result::Error::NotFound) => {
                Box::new(Ok(HttpResponse::NotFound().finish()).into_future())
            }
            error => Box::new(
                Ok(internal_server_error(format!(
                    "Couldn't delete task: {:?}",
                    error
                )))
                .into_future(),
            ),
        },
    }
}
