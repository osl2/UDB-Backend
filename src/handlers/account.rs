use actix_web::{delete, get, post, put, web, Error, HttpRequest, HttpResponse, Scope};
use futures::future::{Future, IntoFuture};

pub fn get_scope(auth: actix_web_jwt_middleware::JwtAuthentication) -> Scope {
    web::scope("/account")
        .service(get_account)
        .service(update_account)
        .service(create_account)
        .service(delete_account)
        .service(login)
}

#[get("")]
fn get_account(req: HttpRequest) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    Box::new(Ok(HttpResponse::NotImplemented().finish()).into_future())
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
