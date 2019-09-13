pub fn internal_server_error(message: String) -> actix_web::HttpResponse {
    log::error!("{}", message);
    if cfg!(debug_assertions) {
        actix_web::HttpResponse::InternalServerError().body(message)
    } else {
        actix_web::HttpResponse::InternalServerError().finish()
    }
}

pub fn database(req: &actix_web::HttpRequest) -> crate::database::Database {
    req.app_data::<crate::AppData>().unwrap().database.clone()
}

pub fn user(req: &actix_web::HttpRequest) -> uuid::Uuid {
    *req.extensions().get::<uuid::Uuid>().unwrap()
}
