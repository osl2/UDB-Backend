use futures::future::{Future, IntoFuture};
use uuid::Uuid;
use actix_web::{
    get, put, post, delete, web, Error, HttpRequest, HttpResponse, Scope
};
use diesel::prelude::*;
use crate::AppData;
use crate::schema;
use crate::models;

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
    let appdata: &AppData = req.app_data().unwrap();

    let conn = match appdata.get_db_connection(){
        Ok(connection) => connection,
        Err(_) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        },
    };

    let query = schema::subtasks::table.find(format!("{}", ids.1)).get_result::<models::Subtask>(&*conn);

    match query {
        Ok(result) => {
            Box::new(Ok(HttpResponse::Ok().json(result)).into_future())
        },
        Err(e) => {
            match e {
                diesel::result::Error::NotFound => Box::new(Ok(HttpResponse::NotFound().finish()).into_future()),
                _ => Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future()),
            }
        }
    }
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
