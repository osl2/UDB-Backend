use actix_web::{web, App, HttpServer};

use actix_web_jwt_middleware::{Algorithm, JwtAuthentication, JwtKey};

fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();

    HttpServer::new(|| {
        App::new().wrap(JwtAuthentication {
            key: JwtKey::Inline("".to_owned()),
            algorithm: Algorithm::HS512,
        })
    })
    .bind("127.0.0.1:8080")?
    .run()
}
