use futures::future::{Future, IntoFuture};
use uuid::Uuid;
use actix_web::{
    get, put, post, delete, web, Error, HttpRequest, HttpResponse, Scope
};

pub fn get_scope() -> Scope {
    web::scope("/courses")
    .service(get_courses)
    .service(create_course)
    .service(get_course)
    .service(update_course)
    .service(delete_course)
}

#[get("")]
fn get_courses(req: HttpRequest) -> Box<Future<Item = HttpResponse, Error = Error>> {
    Box::new(Ok(HttpResponse::NotImplemented().finish()).into_future())
}
#[post("")]
fn create_course(req: HttpRequest) -> Box<Future<Item = HttpResponse, Error = Error>> {
    Box::new(Ok(HttpResponse::NotImplemented().finish()).into_future())
}
#[get("/{id}")]
fn get_course(req: HttpRequest, id: web::Path<Uuid>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    Box::new(Ok(HttpResponse::NotImplemented().finish()).into_future())
}
#[put("/{id}")]
fn update_course(req: HttpRequest, id: web::Path<Uuid>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    Box::new(Ok(HttpResponse::NotImplemented().finish()).into_future())
}
#[delete("/{id}")]
fn delete_course(req: HttpRequest, id: web::Path<Uuid>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    Box::new(Ok(HttpResponse::NotImplemented().finish()).into_future())
}
