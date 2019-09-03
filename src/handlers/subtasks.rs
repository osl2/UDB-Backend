use crate::{
    models,
    solution_compare::compare_solutions,
    util::{database, internal_server_error, user},
};
use actix_web::{web, Error, HttpRequest, HttpResponse, Scope};

use futures::future::{Future, IntoFuture};
use uuid::Uuid;

pub fn get_scope() -> Scope {
    web::scope("/subtasks")
        .service(
            web::resource("")
                .route(web::get().to_async(get_subtasks))
                .route(web::post().to_async(create_subtask)),
        )
        .service(
            web::resource("/{id}")
                .route(web::get().to_async(get_subtask))
                .route(web::put().to_async(update_subtask))
                .route(web::delete().to_async(delete_subtask)),
        )
        .service(web::resource("/{id}/verify").route(web::post().to_async(verify_subtask_solution)))
}

fn get_subtasks(req: HttpRequest) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    match database(&req).get_subtasks(user(&req)) {
        Ok(result) => Box::new(Ok(HttpResponse::Ok().json(result)).into_future()),
        Err(e) => Box::new(
            Ok(internal_server_error(format!(
                "Couldn't get subtasks: {:?}",
                e
            )))
            .into_future(),
        ),
    }
}
fn create_subtask(
    req: HttpRequest,
    json: web::Json<models::Subtask>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    match database(&req).create_subtask(json.into_inner(), user(&req)) {
        Ok(result) => Box::new(Ok(HttpResponse::Created().body(result.to_string())).into_future()),
        Err(e) => Box::new(
            Ok(internal_server_error(format!(
                "Couldn't create subtask: {:?}",
                e
            )))
            .into_future(),
        ),
    }
}
fn get_subtask(
    req: HttpRequest,
    id: web::Path<Uuid>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    match database(&req).get_subtask(id.into_inner()) {
        Ok(result) => Box::new(Ok(HttpResponse::Ok().json(result)).into_future()),
        Err(error) => match error {
            crate::database::DatabaseError::Diesel(diesel::result::Error::NotFound) => {
                Box::new(Ok(HttpResponse::NotFound().finish()).into_future())
            }
            error => Box::new(
                Ok(internal_server_error(format!(
                    "Couldn't get subtask: {:?}",
                    error
                )))
                .into_future(),
            ),
        },
    }
}
fn update_subtask(
    req: HttpRequest,
    id: web::Path<Uuid>,
    json: web::Json<models::Subtask>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    match database(&req).update_subtask(json.into_inner(), id.into_inner()) {
        Ok(_) => Box::new(Ok(HttpResponse::Ok().finish()).into_future()),
        Err(error) => match error {
            crate::database::DatabaseError::Diesel(diesel::result::Error::NotFound) => {
                Box::new(Ok(HttpResponse::NotFound().finish()).into_future())
            }
            error => Box::new(
                Ok(internal_server_error(format!(
                    "Couldn't update subtask: {:?}",
                    error
                )))
                .into_future(),
            ),
        },
    }
}
fn delete_subtask(
    req: HttpRequest,
    id: web::Path<Uuid>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    match database(&req).delete_subtask(id.into_inner(), user(&req)) {
        Ok(_) => Box::new(Ok(HttpResponse::Ok().finish()).into_future()),
        Err(error) => match error {
            crate::database::DatabaseError::Diesel(diesel::result::Error::NotFound) => {
                Box::new(Ok(HttpResponse::NotFound().finish()).into_future())
            }
            error => Box::new(
                Ok(internal_server_error(format!(
                    "Couldn't delete subtask: {:?}",
                    error
                )))
                .into_future(),
            ),
        },
    }
}
fn verify_subtask_solution(
    req: HttpRequest,
    id: web::Path<Uuid>,
    json: web::Json<models::Solution>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    // get student solution
    let student_solution = json.into_inner();

    // get teacher solution
    match database(&req).get_subtask(id.into_inner()) {
        Ok(subtask) => {
            if !subtask.is_solution_verifiable || !subtask.is_solution_visible {
                // this subtask does not have a public solution
                return Box::new(Ok(HttpResponse::NotFound().finish()).into_future());
            }

            let result = compare_solutions(student_solution, subtask);

            Box::new(Ok(HttpResponse::Ok().json(result)).into_future())
        }
        Err(error) => match error {
            crate::database::DatabaseError::Diesel(diesel::result::Error::NotFound) => {
                Box::new(Ok(HttpResponse::NotFound().finish()).into_future())
            }
            error => Box::new(
                Ok(internal_server_error(format!(
                    "Couldn't get subtask: {:?}",
                    error
                )))
                .into_future(),
            ),
        },
    }
}
