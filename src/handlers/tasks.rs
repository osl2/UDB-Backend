use futures::future::{Future, IntoFuture};
use uuid::Uuid;
use actix_web::{
    get, put, post, delete, web, Error, HttpRequest, HttpResponse, Scope
};
use diesel::prelude::*;
use crate::schema;
use crate::models;
use crate::AppData;
use crate::handlers::subtasks;
use std::io::SeekFrom::Start;

pub fn get_scope() -> Scope {
    web::scope("/tasks")
    .service(get_tasks)
    .service(create_task)
    .service(get_task)
    .service(update_task)
    .service(delete_task)
    .service(subtasks::get_scope())
}

#[get("")]
fn get_tasks(req: HttpRequest) -> Box<Future<Item = HttpResponse, Error = Error>> {
    Box::new(Ok(HttpResponse::NotImplemented().finish()).into_future())
}
#[post("")]
fn create_task(req: HttpRequest) -> Box<Future<Item = HttpResponse, Error = Error>> {
    Box::new(Ok(HttpResponse::NotImplemented().finish()).into_future())
}
#[get("/{id}")]
fn get_task(req: HttpRequest, id: web::Path<Uuid>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let appdata: &AppData = req.app_data().unwrap();

    let conn = match appdata.get_db_connection(){
        Ok(connection) => connection,
        Err(_) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        },
    };

    let query = schema::tasks::table.find(format!("{}", id)).get_result::<models::QueryableTask>(&*conn);

    match query {
        Ok(task) => {

            let subtasks_query = schema::subtasks_in_tasks::table
                .filter(schema::subtasks_in_tasks::columns::task_id.eq(format!("{}", id)))
                .select((schema::subtasks_in_tasks::columns::subtask_id))
                .load::<String>(&*conn);

            Box::new(Ok(HttpResponse::Ok().json(models::Task {
                id: task.id,
                database: task.database,
                subtasks: subtasks_query.ok(),
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
fn update_task(req: HttpRequest, id: web::Path<Uuid>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    Box::new(Ok(HttpResponse::NotImplemented().finish()).into_future())
}
#[delete("/{id}")]
fn delete_task(req: HttpRequest, id: web::Path<Uuid>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    Box::new(Ok(HttpResponse::NotImplemented().finish()).into_future())
}
