use actix_web::{delete, post, put, web, Error, HttpRequest, HttpResponse, Scope};
use futures::future::{Future, IntoFuture};
use crate::{schema, models, AppData};
use diesel::prelude::*;

pub fn get_scope(auth: actix_web_jwt_middleware::JwtAuthentication) -> Scope {
    web::scope("/account")
        .service(web::resource("").route(web::get().to_async(get_account)))
        .service(update_account)
        .service(create_account)
        .service(delete_account)
        .service(login)
}

fn get_account(req: HttpRequest) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let appdata: &AppData = req.app_data().unwrap();

    let unauthorized = Box::new(Ok(HttpResponse::Unauthorized().finish()).into_future());
    //TODO: This is awful. We should use a typed alternative instead of a json Value
    let id = match req.extensions().get::<actix_web_jwt_middleware::AuthenticationData>() {
        Some(auth) => match &auth.0 {
            serde_json::Value::Object(map) => match map.get("sub") {
                Some(subject) => match subject {
                    serde_json::Value::String(subject) => {
                        uuid::Uuid::parse_str(&subject).unwrap()
                    },
                    _ => {
                        return unauthorized;
                    }
                },
                _ => {
                    return unauthorized;
                }
            },
            _ => {
                return unauthorized;
            }
        },
        None => {
            return unauthorized;
        }
    };

    let conn = match appdata.get_db_connection() {
        Ok(connection) => connection,
        Err(_) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        }
    };

    let query = schema::users::table
        .find(format!("{}", id))
        .get_result::<models::User>(&*conn);

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
#[put("")]
fn update_account(req: HttpRequest) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    Box::new(Ok(HttpResponse::NotImplemented().finish()).into_future())
}
#[post("")]
fn create_account(req: HttpRequest) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    Box::new(Ok(HttpResponse::NotImplemented().finish()).into_future())
}
#[delete("")]
fn delete_account(req: HttpRequest) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    Box::new(Ok(HttpResponse::NotImplemented().finish()).into_future())
}
#[post("/login")]
fn login(req: HttpRequest) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {

    Box::new(Ok(HttpResponse::NotImplemented().finish()).into_future())
}
