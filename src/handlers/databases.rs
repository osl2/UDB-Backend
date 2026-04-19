use crate::models;
use crate::schema;
use actix_web::{web, HttpRequest, HttpResponse, Responder, Scope};
use actix_web::HttpMessage;
use std::cell::RefCell;
use uuid::Uuid;

use diesel::{
    r2d2::{self, ConnectionManager},
    Connection, ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl, SqliteConnection,
};

pub fn get_scope() -> Scope {
    web::scope("/databases")
        .service(
            web::resource("")
                .app_data(web::JsonConfig::default().limit(4194304))
                .route(web::get().to(get_databases))
                .route(web::post().to(create_database)),
        )
        .service(
            web::resource("/{id}")
                .app_data(web::JsonConfig::default().limit(4194304))
                .route(web::get().to(get_database))
                .route(web::put().to(update_database))
                .route(web::delete().to(delete_database)),
        )
}

pub async fn get_databases(req: HttpRequest) -> impl Responder {
    let extensions = req.extensions();
    let conn_cell = extensions
        .get::<RefCell<r2d2::PooledConnection<ConnectionManager<SqliteConnection>>>>()
        .unwrap();
    let mut conn = conn_cell.borrow_mut();
    let sub = extensions
        .get::<actix_web_jwt_middleware::AuthenticationData>()
        .unwrap()
        .claims
        .sub
        .clone()
        .unwrap();

    match schema::databases::table
        .inner_join(
            schema::access::table
                .on(schema::databases::columns::id.eq(schema::access::columns::object_id)),
        )
        .filter(schema::access::columns::user_id.eq(sub))
        .select((
            schema::databases::columns::id,
            schema::databases::columns::name,
            schema::databases::columns::content,
        ))
        .load::<models::Database>(&mut *conn)
    {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(e) => {
            log::error!("Couldn't load database: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn create_database(
    req: HttpRequest,
    json: web::Json<models::Database>,
) -> impl Responder {
    let extensions = req.extensions();
    let conn_cell = extensions
        .get::<RefCell<r2d2::PooledConnection<ConnectionManager<SqliteConnection>>>>()
        .unwrap();
    let mut conn = conn_cell.borrow_mut();
    let sub = extensions
        .get::<actix_web_jwt_middleware::AuthenticationData>()
        .unwrap()
        .claims
        .sub
        .clone()
        .unwrap();

    match conn.transaction::<Uuid, diesel::result::Error, _>(|conn| {
        let mut new_database = json.into_inner();
        let id = Uuid::new_v4();
        new_database.id = id.to_string();

        diesel::insert_into(schema::access::table)
            .values(models::Access {
                user_id: sub,
                object_id: id.to_string(),
            })
            .execute(conn)?;

        diesel::insert_into(schema::databases::table)
            .values(new_database)
            .execute(conn)?;

        Ok(id)
    }) {
        Ok(id) => HttpResponse::Ok().body(id.to_string()),
        Err(e) => {
            log::error!("Couldn't create database: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_database(
    req: HttpRequest,
    id: web::Path<Uuid>,
) -> impl Responder {
    let extensions = req.extensions();
    let conn_cell = extensions
        .get::<RefCell<r2d2::PooledConnection<ConnectionManager<SqliteConnection>>>>()
        .unwrap();
    let mut conn = conn_cell.borrow_mut();
    let uuid = id.into_inner();

    match schema::databases::table
        .find(format!("{}", uuid))
        .get_result::<models::Database>(&mut *conn)
    {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(e) => match e {
            diesel::result::Error::NotFound => HttpResponse::NotFound().finish(),
            e => {
                log::error!("Couldn't load database: {}", e);
                HttpResponse::InternalServerError().finish()
            }
        },
    }
}

pub async fn update_database(
    req: HttpRequest,
    id: web::Path<Uuid>,
    json: web::Json<models::Database>,
) -> impl Responder {
    let extensions = req.extensions();
    let conn_cell = extensions
        .get::<RefCell<r2d2::PooledConnection<ConnectionManager<SqliteConnection>>>>()
        .unwrap();
    let mut conn = conn_cell.borrow_mut();
    let uuid = id.into_inner();

    match diesel::update(schema::databases::table.find(format!("{}", uuid)))
        .set(json.into_inner())
        .execute(&mut *conn)
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            log::error!("Couldn't update database: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn delete_database(
    req: HttpRequest,
    id: web::Path<Uuid>,
) -> impl Responder {
    let extensions = req.extensions();
    let conn_cell = extensions
        .get::<RefCell<r2d2::PooledConnection<ConnectionManager<SqliteConnection>>>>()
        .unwrap();
    let mut conn = conn_cell.borrow_mut();
    let uuid = id.into_inner();

    match diesel::delete(schema::databases::table.find(format!("{}", uuid)))
        .execute(&mut *conn)
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            log::error!("Couldn't delete database: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
