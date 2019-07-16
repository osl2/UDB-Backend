use futures::future::{Future, IntoFuture};
use uuid::Uuid;
use actix_web::{
    get, put, post, delete, web, Error, HttpRequest, HttpResponse, Scope
};

use log::debug;

pub fn get_scope() -> Scope {
    web::scope("/{task_id}/subtasks")
    .service(get_subtasks)
    .service(create_subtask)
    .service(get_subtask)
    .service(update_subtask)
    .service(delete_subtask)
}

#[get("")]
fn get_subtasks(req: HttpRequest, task_id: web::Path<Uuid>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    Box::new(Ok(HttpResponse::NotImplemented().finish()).into_future())
}
#[post("")]
fn create_subtask(req: HttpRequest, task_id: web::Path<Uuid>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    Box::new(Ok(HttpResponse::NotImplemented().finish()).into_future())
}
#[get("/{subtask_id}")]
fn get_subtask(req: HttpRequest, ids: web::Path<(Uuid, Uuid)>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    debug!("Received request: {:?}", req);
    debug!("Has task ID {} and subtask ID {}", ids.0, ids.1);
    Box::new(Ok(HttpResponse::NotImplemented().finish()).into_future())
}
#[put("/{subtask_id}")]
fn update_subtask(req: HttpRequest, ids: web::Path<(Uuid, Uuid)>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    Box::new(Ok(HttpResponse::NotImplemented().finish()).into_future())
}
#[delete("/{subtask_id}")]
fn delete_subtask(req: HttpRequest, ids: web::Path<(Uuid, Uuid)>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    Box::new(Ok(HttpResponse::NotImplemented().finish()).into_future())
}
#[post("/{subtask_id}")]
fn verify_subtask_solution(req: HttpRequest, ids: web::Path<(Uuid, Uuid)>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    Box::new(Ok(HttpResponse::NotImplemented().finish()).into_future())
}
