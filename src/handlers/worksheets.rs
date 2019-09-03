use crate::{
    models,
    util::{database, internal_server_error, user},
};
use actix_web::{web, Error, HttpRequest, HttpResponse, Scope};
use futures::future::{Future, IntoFuture};
use uuid::Uuid;

pub fn get_scope() -> Scope {
    web::scope("/worksheets")
        .service(
            web::resource("")
                .route(web::get().to_async(get_worksheets))
                .route(web::post().to_async(create_worksheet)),
        )
        .service(
            web::resource("/{id}")
                .route(web::get().to_async(get_worksheet))
                .route(web::put().to_async(update_worksheet))
                .route(web::delete().to_async(delete_worksheet)),
        )
}

fn get_worksheets(req: HttpRequest) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    match database(&req).get_worksheets(user(&req)) {
        Ok(result) => Box::new(Ok(HttpResponse::Ok().json(result)).into_future()),
        Err(e) => Box::new(
            internal_server_error(format!("Couldn't get worksheets: {:?}", e)).into_future(),
        ),
    }
}

fn create_worksheet(
    req: HttpRequest,
    json: web::Json<models::Worksheet>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    match database(&req).create_worksheet(user(&req), json.into_inner()) {
        Ok(id) => Box::new(Ok(HttpResponse::Created().body(id.to_string())).into_future()),
        Err(e) => Box::new(
            internal_server_error(format!("Couldn't create worksheet: {:?}", e)).into_future(),
        ),
    }
}

fn get_worksheet(
    req: HttpRequest,
    id: web::Path<Uuid>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    match database(&req).get_worksheet(id.into_inner()) {
        Ok(sheet) => Box::new(Ok(HttpResponse::Ok().json(sheet)).into_future()),
        Err(e) => match e {
            crate::database::DatabaseError::Diesel(diesel::result::Error::NotFound) => {
                Box::new(Ok(HttpResponse::NotFound().finish()).into_future())
            }
            e => Box::new(
                Ok(internal_server_error(format!(
                    "Couldn't load worksheet: {:?}",
                    e
                )))
                .into_future(),
            ),
        },
    }
}

fn update_worksheet(
    req: HttpRequest,
    id: web::Path<Uuid>,
    json: web::Json<models::Worksheet>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    match database(&req).update_worksheet(id.into_inner(), json.into_inner(), user(&req)) {
        Ok(_) => Box::new(Ok(HttpResponse::Ok().finish()).into_future()),
        Err(e) => {
            log::error!("Couldn't update worksheet: {:?}", e);
            Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future())
        }
    }
}

fn delete_worksheet(
    req: HttpRequest,
    id: web::Path<Uuid>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    match database(&req).delete_worksheet(id.into_inner(), user(&req)) {
        Ok(_) => Box::new(Ok(HttpResponse::Ok().finish()).into_future()),
        Err(e) => {
            log::error!("Couldn't delete worksheet: {:?}", e);
            Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future())
        }
    }
}
