use futures::future::{Future, IntoFuture};
use uuid::Uuid;
use actix_web::{
    get, put, post, delete, web, Error, HttpRequest, HttpResponse, Scope
};
use diesel::prelude::*;
use crate::schema;
use crate::models;
use crate::AppData;

pub fn get_scope() -> Scope {
    web::scope("/worksheets")
    .service(get_worksheets)
    .service(create_worksheet)
    .service(get_worksheet)
    .service(update_worksheet)
    .service(delete_worksheet)
}

#[get("")]
fn get_worksheets(req: HttpRequest) -> Box<Future<Item = HttpResponse, Error = Error>> {
    Box::new(Ok(HttpResponse::NotImplemented().finish()).into_future())
}
#[post("")]
fn create_worksheet(req: HttpRequest) -> Box<Future<Item = HttpResponse, Error = Error>> {
    Box::new(Ok(HttpResponse::NotImplemented().finish()).into_future())
}
#[get("/{id}")]
fn get_worksheet(req: HttpRequest, id: web::Path<Uuid>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let appdata: &AppData = req.app_data().unwrap();

    let conn = match appdata.get_db_connection(){
        Ok(connection) => connection,
        Err(_) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        },
    };

    let query = schema::worksheets::table.find(format!("{}", id)).get_result::<models::QueryableWorksheet>(&*conn);

    match query {
        Ok(worksheet) => {
            let tasks_query = schema::tasks_in_worksheets::table
                .filter(schema::tasks_in_worksheets::columns::worksheet_id.eq(format!("{}", id)))
                .select((schema::tasks_in_worksheets::columns::task_id))
                .load::<String>(&*conn);

            Box::new(Ok(HttpResponse::Ok().json(models::Worksheet {
                id: worksheet.id,
                name: worksheet.name,
                is_online: worksheet.is_online,
                is_solution_online: worksheet.is_solution_online,
                tasks: tasks_query.ok(),
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
fn update_worksheet(req: HttpRequest, id: web::Path<Uuid>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    Box::new(Ok(HttpResponse::NotImplemented().finish()).into_future())
}
#[delete("/{id}")]
fn delete_worksheet(req: HttpRequest, id: web::Path<Uuid>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    Box::new(Ok(HttpResponse::NotImplemented().finish()).into_future())
}
