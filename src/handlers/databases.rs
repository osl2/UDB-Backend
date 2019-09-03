use crate::{
    models,
    util::{database, internal_server_error, user},
};
use actix_web::{web, Error, FromRequest, HttpRequest, HttpResponse, Scope};

use futures::future::{Future, IntoFuture};
use uuid::Uuid;

pub fn get_scope() -> Scope {
    let json_config = web::Json::<models::Database>::configure(|cfg| {
        cfg.limit(4 * (1024usize.pow(2))) //4MB limit
    });
    web::scope("/databases")
        .service(
            web::resource("")
                .data(json_config.clone())
                .route(web::get().to_async(get_databases))
                .route(web::post().to_async(create_database)),
        )
        .service(
            web::resource("/{id}")
                .data(json_config.clone())
                .route(web::get().to_async(get_database))
                .route(web::put().to_async(update_database))
                .route(web::delete().to_async(delete_database)),
        )
}

pub fn get_databases(req: HttpRequest) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    match database(&req).get_databases(user(&req)) {
        Ok(result) => Box::new(Ok(HttpResponse::Ok().json(result)).into_future()),
        Err(e) => Box::new(
            Ok(internal_server_error(format!(
                "Couldn't load database: {:?}",
                e
            )))
            .into_future(),
        ),
    }
}

pub fn create_database(
    req: HttpRequest,
    json: web::Json<models::Database>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    match database(&req).create_database(user(&req), json.into_inner()) {
        Ok(id) => Box::new(Ok(HttpResponse::Created().body(id.to_string())).into_future()),
        Err(e) => Box::new(
            Ok(internal_server_error(format!(
                "Couldn't create database: {:?}",
                e
            )))
            .into_future(),
        ),
    }
}

pub fn get_database(
    req: HttpRequest,
    id: web::Path<Uuid>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    match database(&req).get_database(id.into_inner()) {
        Ok(result) => Box::new(Ok(HttpResponse::Ok().json(result)).into_future()),
        Err(e) => match e {
            crate::database::DatabaseError::Diesel(diesel::result::Error::NotFound) => {
                Box::new(Ok(HttpResponse::NotFound().finish()).into_future())
            }
            e => Box::new(
                Ok(internal_server_error(format!(
                    "Couldn't load database: {:?}",
                    e
                )))
                .into_future(),
            ),
        },
    }
}

pub fn update_database(
    req: HttpRequest,
    id: web::Path<Uuid>,
    json: web::Json<models::Database>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    match database(&req).update_database(id.into_inner(), json.into_inner()) {
        Ok(_) => Box::new(Ok(HttpResponse::Ok().finish()).into_future()),
        Err(e) => match e {
            crate::database::DatabaseError::Diesel(diesel::result::Error::NotFound) => {
                Box::new(Ok(HttpResponse::NotFound().finish()).into_future())
            }
            e => Box::new(
                Ok(internal_server_error(format!(
                    "Couldn't update database: {:?}",
                    e
                )))
                .into_future(),
            ),
        },
    }
}

pub fn delete_database(
    req: HttpRequest,
    id: web::Path<Uuid>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    match database(&req).delete_database(id.into_inner()) {
        Ok(_) => Box::new(Ok(HttpResponse::Ok().finish()).into_future()),
        Err(e) => match e {
            crate::database::DatabaseError::Diesel(diesel::result::Error::NotFound) => {
                Box::new(Ok(HttpResponse::NotFound().finish()).into_future())
            }
            e => Box::new(
                Ok(internal_server_error(format!(
                    "Couldn't delete database: {:?}",
                    e
                )))
                .into_future(),
            ),
        },
    }
}
