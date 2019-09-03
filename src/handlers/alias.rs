use crate::models;
use crate::util::{database, internal_server_error};
use actix_web::{web, Error, HttpRequest, HttpResponse, Scope};
use futures::future::{Future, IntoFuture};
use uuid::Uuid;

pub fn get_scope() -> Scope {
    web::scope("/alias")
        .service(web::resource("").route(web::post().to_async(create_alias)))
        .service(web::resource("/{id}").route(web::get().to_async(get_alias)))
        .service(web::resource("/uuid/{alias}").route(web::get().to_async(get_uuid)))
}

fn create_alias(
    req: HttpRequest,
    json: web::Json<models::AliasRequest>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    match database(&req).create_alias(json.into_inner()) {
        Ok(alias) => Box::new(Ok(HttpResponse::Created().body(alias)).into_future()),
        Err(e) => match e {
            crate::database::DatabaseError::Diesel(e) => Box::new(
                Ok(internal_server_error(format!(
                    "Couldn't create new alias: {:?}",
                    e
                )))
                .into_future(),
            ),
            crate::database::DatabaseError::R2D2(e) => Box::new(
                Ok(internal_server_error(format!(
                    "Couldn't create new alias: {:?}",
                    e
                )))
                .into_future(),
            ),
            crate::database::DatabaseError::Other(
                crate::database::OtherErrorKind::NoFreeAliases,
            ) => Box::new(
                Ok(HttpResponse::InternalServerError().body("Couldn't find a free alias."))
                    .into_future(),
            ),
        },
    }
}

fn get_alias(
    req: HttpRequest,
    id: web::Path<Uuid>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    match database(&req).get_alias_by_uuid(id.into_inner()) {
        Ok(result) => Box::new(Ok(HttpResponse::Ok().json(result)).into_future()),
        Err(error) => match error {
            crate::database::DatabaseError::Diesel(diesel::result::Error::NotFound) => {
                Box::new(Ok(HttpResponse::NotFound().finish()).into_future())
            }
            error => Box::new(
                Ok(internal_server_error(format!(
                    "Couldn't get alias: {:?}",
                    error
                )))
                .into_future(),
            ),
        },
    }
}

fn get_uuid(
    req: HttpRequest,
    alias: web::Path<String>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    match database(&req).get_uuid_by_alias(alias.into_inner()) {
        Ok(result) => Box::new(Ok(HttpResponse::Ok().json(result)).into_future()),
        Err(error) => match error {
            crate::database::DatabaseError::Diesel(diesel::result::Error::NotFound) => {
                Box::new(Ok(HttpResponse::NotFound().finish()).into_future())
            }
            error => Box::new(
                Ok(internal_server_error(format!(
                    "Couldn't get uuid: {:?}",
                    error
                )))
                .into_future(),
            ),
        },
    }
}
