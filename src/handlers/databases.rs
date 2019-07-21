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
    web::scope("/databases")
    .service(get_databases)
    .service(create_database)
    .service(get_database)
    .service(update_database)
    .service(delete_database)
}

const CURRENT_USER: &str = "549b60cd-9b88-467b-9b1e-b15c68114c96";  // TODO: user authentication

#[get("")]
pub fn get_databases(req: HttpRequest) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let appdata: &AppData = req.app_data().unwrap();

    let conn = match appdata.get_db_connection(){
        Ok(connection) => connection,
        Err(_) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        },
    };

    let query = schema::databases::table.inner_join(schema::access::table.on(schema::databases::columns::id.eq(schema::access::columns::object_id)))
        .filter(schema::access::columns::user_id.eq(CURRENT_USER))
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

#[post("")]
pub fn create_database(req: HttpRequest, json: web::Json<models::Database>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let appdata: &AppData = req.app_data().unwrap();

    let conn = match appdata.get_db_connection(){
        Ok(connection) => connection,
        Err(_) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        },
    };

    // create database object
    let mut new_database = json.into_inner();
    let id = Uuid::new_v4();
    new_database.id = id.to_string();

    // insert access for user
    match diesel::insert_into(schema::access::table)
        .values(models::Access{ user_id: CURRENT_USER.to_string(), object_id: id.to_string() })
        .execute(&*conn) {
        Ok(result) => {}
        Err(e) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        }
    }

    // insert database object
    match diesel::insert_into(schema::databases::table).values(new_database).execute(&*conn) {
        Ok(result) => {
            Box::new(Ok(HttpResponse::Ok().json(id)).into_future())
        }
        Err(e) => {
            Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future())
        }
    }
}

#[get("/{id}")]
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

#[put("/{id}")]
pub fn update_database(req: HttpRequest, id: web::Path<Uuid>, json: web::Json<models::Database>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let appdata: &AppData = req.app_data().unwrap();

    let conn = match appdata.get_db_connection(){
        Ok(connection) => connection,
        Err(_) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        },
    };

    let query = diesel::update(schema::databases::table.find(format!("{}", id)))
        .set(json.into_inner())
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

#[delete("/{id}")]
pub fn delete_database(req: HttpRequest, id: web::Path<Uuid>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let appdata: &AppData = req.app_data().unwrap();

    let conn = match appdata.get_db_connection(){
        Ok(connection) => connection,
        Err(_) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        },
    };

    let query = diesel::delete(schema::databases::table.find(format!("{}", id.into_inner())))
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