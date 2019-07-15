use futures::future::{Future, IntoFuture};
use uuid::Uuid;
use actix_web::{
    get, put, post, delete, web, Error, HttpRequest, HttpResponse
};

#[get("/tasks/{task_id}/subtasks")]
pub fn get_subtasks(req: HttpRequest, task_id: web::Path<Uuid>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    Box::new(Ok(HttpResponse::NotImplemented().finish()).into_future())
}
#[post("/tasks/{task_id}/subtasks")]
pub fn create_subtask(req: HttpRequest, task_id: web::Path<Uuid>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    Box::new(Ok(HttpResponse::NotImplemented().finish()).into_future())
}
#[get("/tasks/{task_id}/subtasks/{subtask_id}")]
pub fn get_subtask(req: HttpRequest, ids: web::Path<(Uuid, Uuid)>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    Box::new(Ok(HttpResponse::NotImplemented().finish()).into_future())
}
#[put("/tasks/{task_id}/subtasks/{subtask_id}")]
pub fn update_subtask(req: HttpRequest, ids: web::Path<(Uuid, Uuid)>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    Box::new(Ok(HttpResponse::NotImplemented().finish()).into_future())
}
#[delete("/tasks/{task_id}/subtasks/{subtask_id}")]
pub fn delete_subtask(req: HttpRequest, ids: web::Path<(Uuid, Uuid)>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    Box::new(Ok(HttpResponse::NotImplemented().finish()).into_future())
}
#[post("/tasks/{task_id}/subtasks/{subtask_id}")]
pub fn verify_subtask_solution(req: HttpRequest, ids: web::Path<(Uuid, Uuid)>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    Box::new(Ok(HttpResponse::NotImplemented().finish()).into_future())
}
