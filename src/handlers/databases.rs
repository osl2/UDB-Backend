use crate::models;
use crate::schema;
use crate::AppData;
use actix_web::{web, Error, HttpRequest, HttpResponse, Scope};
use diesel::prelude::*;
use futures::future::{Future, IntoFuture};
use uuid::Uuid;

pub fn get_scope() -> Scope {
    web::scope("/databases")
        .service(
            web::resource("")
                .route(web::get().to_async(get_databases))
                .route(web::post().to_async(create_database)),
        )
        .service(
            web::resource("/{id}")
                .route(web::get().to_async(get_database))
                .route(web::put().to_async(update_database))
                .route(web::delete().to_async(delete_database)),
        )
}

pub fn get_databases(req: HttpRequest) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let appdata: &AppData = req.app_data().unwrap();

    let conn = match appdata.get_db_connection() {
        Ok(connection) => connection,
        Err(_) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        }
    };

    let query = schema::databases::table
        .inner_join(
            schema::access::table
                .on(schema::databases::columns::id.eq(schema::access::columns::object_id)),
        )
        .filter(schema::access::columns::user_id.eq(appdata.current_user.to_string()))
        .select((
            schema::databases::columns::id,
            schema::databases::columns::name,
            schema::databases::columns::content,
        ))
        .load::<models::Database>(&*conn);

    match query {
        Ok(result) => Box::new(Ok(HttpResponse::Ok().json(result)).into_future()),
        Err(e) => Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future()),
    }
}

pub fn create_database(
    req: HttpRequest,
    json: web::Json<models::Database>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let appdata: &AppData = req.app_data().unwrap();

    let conn = match appdata.get_db_connection() {
        Ok(connection) => connection,
        Err(_) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        }
    };

    // create database object
    let mut new_database = json.into_inner();
    let id = Uuid::new_v4();
    new_database.id = id.to_string();

    // insert access for user
    match diesel::insert_into(schema::access::table)
        .values(models::Access {
            user_id: appdata.current_user.to_string(),
            object_id: id.to_string(),
        })
        .execute(&*conn)
    {
        Ok(result) => {}
        Err(e) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        }
    }

    // insert database object
    match diesel::insert_into(schema::databases::table)
        .values(new_database)
        .execute(&*conn)
    {
        Ok(result) => Box::new(Ok(HttpResponse::Ok().json(id)).into_future()),
        Err(e) => Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future()),
    }
}

pub fn get_database(
    req: HttpRequest,
    id: web::Path<Uuid>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let appdata: &AppData = req.app_data().unwrap();

    let conn = match appdata.get_db_connection() {
        Ok(connection) => connection,
        Err(_) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        }
    };

    let query = schema::databases::table
        .find(format!("{}", id))
        .get_result::<models::Database>(&*conn);

    match query {
        Ok(result) => Box::new(Ok(HttpResponse::Ok().json(result)).into_future()),
        Err(e) => match e {
            diesel::result::Error::NotFound => {
                Box::new(Ok(HttpResponse::NotFound().finish()).into_future())
            }
            _ => Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future()),
        },
    }
}

pub fn update_database(
    req: HttpRequest,
    id: web::Path<Uuid>,
    json: web::Json<models::Database>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let appdata: &AppData = req.app_data().unwrap();

    let conn = match appdata.get_db_connection() {
        Ok(connection) => connection,
        Err(_) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        }
    };

    let query = diesel::update(schema::databases::table.find(format!("{}", id)))
        .set(json.into_inner())
        .execute(&*conn);

    match query {
        Ok(result) => Box::new(Ok(HttpResponse::Ok().finish()).into_future()),
        Err(e) => Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future()),
    }
}

pub fn delete_database(
    req: HttpRequest,
    id: web::Path<Uuid>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let appdata: &AppData = req.app_data().unwrap();

    let conn = match appdata.get_db_connection() {
        Ok(connection) => connection,
        Err(_) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        }
    };

    let query = diesel::delete(schema::databases::table.find(format!("{}", id.into_inner())))
        .execute(&*conn);
    match query {
        Ok(result) => Box::new(Ok(HttpResponse::Ok().finish()).into_future()),
        Err(e) => Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future()),
    }
}
