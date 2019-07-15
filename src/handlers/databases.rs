use futures::future::{Future, IntoFuture};
use uuid::Uuid;
use actix_web::{
    get, put, post, delete, web, Error, HttpRequest, HttpResponse
};
use crate::AppData;
use crate::schema;
use crate::models;
use diesel::prelude::*;

#[get("/databases")]
pub fn get_databases(req: HttpRequest) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let appdata: &AppData = req.app_data().unwrap();

    let conn = match appdata.get_db_connection(){
        Ok(connection) => connection,
        Err(_) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        },
    };

    let query = schema::databases::table.inner_join(schema::access::table.on(schema::databases::columns::id.eq(schema::access::columns::object_id)))
        .filter(schema::access::columns::user_id.eq("549b60cd-9b88-467b-9b1e-b15c68114c96"))
        .select((schema::databases::columns::id, schema::databases::columns::name, schema::databases::columns::content))
        .load::<models::Database>(&*conn);

    match query {
        Ok(result) => {
            Box::new(Ok(HttpResponse::Ok().json(result)).into_future())
        },
        Err(e) => {
            Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future())
        }
    }
}
#[post("/databases")]
pub fn create_database(req: HttpRequest) -> Box<Future<Item = HttpResponse, Error = Error>> {
    Box::new(Ok(HttpResponse::NotImplemented().finish()).into_future())
}
#[get("/databases/{id}")]
pub fn get_database(req: HttpRequest, id: web::Path<Uuid>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let appdata: &AppData = req.app_data().unwrap();

    let conn = match appdata.get_db_connection(){
        Ok(connection) => connection,
        Err(_) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        },
    };

    let query = schema::databases::table.find(format!("{}", id)).get_result::<models::Database>(&*conn);

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
#[put("/databases/{id}")]
pub fn update_database(req: HttpRequest, id: web::Path<Uuid>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    Box::new(Ok(HttpResponse::NotImplemented().finish()).into_future())
}
#[delete("/databases/{id}")]
pub fn delete_database(req: HttpRequest, id: web::Path<Uuid>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    Box::new(Ok(HttpResponse::NotImplemented().finish()).into_future())
}
