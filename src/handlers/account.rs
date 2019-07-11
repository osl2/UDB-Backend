use futures::future::{Future, IntoFuture};
use actix_web::{
    get, put, post, delete, Error, HttpRequest, HttpResponse
};

#[get("/account")]
fn get_account(req: HttpRequest) -> Box<Future<Item = HttpResponse, Error = Error>> {
    Box::new(Ok(HttpResponse::NotImplemented().finish()).into_future())
}
#[put("/account")]
fn update_account(req: HttpRequest) -> Box<Future<Item = HttpResponse, Error = Error>> {
    Box::new(Ok(HttpResponse::NotImplemented().finish()).into_future())
}
#[post("/account")]
fn create_account(req: HttpRequest) -> Box<Future<Item = HttpResponse, Error = Error>> {
    Box::new(Ok(HttpResponse::NotImplemented().finish()).into_future())
}
#[delete("/account")]
fn delete_account(req: HttpRequest) -> Box<Future<Item = HttpResponse, Error = Error>> {
    Box::new(Ok(HttpResponse::NotImplemented().finish()).into_future())
}
#[post("/account/login")]
fn login(req: HttpRequest) -> Box<Future<Item = HttpResponse, Error = Error>> {
    Box::new(Ok(HttpResponse::NotImplemented().finish()).into_future())
}
