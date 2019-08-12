#[macro_use]
extern crate diesel;

use actix_cors::Cors;
use actix_web::{http::Method, web, App, HttpServer};
use log::error;
use regex::Regex;

use actix_web_jwt_middleware::{Algorithm, JwtAuthentication, JwtKey};

mod alias_generator;
mod cli;
mod database;
mod handlers;
mod logging;
mod middlewares;
mod models;
mod schema;
mod settings;
mod solution_compare;


#[derive(Clone)]
struct AppData {
    settings: settings::Settings,
}

impl AppData {
    pub fn from_configuration(config: settings::Settings) -> Self {
        Self { settings: config }
    }
}

fn main() {
    let cli_matches = cli::setup_cli();
    logging::setup_logging(match cli_matches.occurrences_of("v") {
        0 => log::LevelFilter::Error,
        1 => log::LevelFilter::Warn,
        2 => log::LevelFilter::Info,
        3 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    });
    let configuration =
        settings::Settings::new(cli_matches.value_of("config").unwrap_or("config.toml")).unwrap();
    let sys = actix::System::new("udb-backend");

    let appstate = AppData::from_configuration(configuration.clone());

    let jwt_key = configuration.jwt_key.clone();
    let mut server = HttpServer::new(move || {
        App::new()
            .data(appstate.clone())
            .wrap(middlewares::upload_filter::UploadFilter { filter: false })
            .wrap(middlewares::ownership::OwnershipChecker{})
            .wrap(JwtAuthentication {
                key: JwtKey::Inline(jwt_key.clone()),
                algorithm: Algorithm::HS512,
                except: vec![(
                    Regex::new(
                        r"(?P<uuid>[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})$",
                    )
                    .unwrap(),
                    vec![Method::GET],
                ),(
                    Regex::new(
                        r"/subtasks/(?P<uuid>[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})/verify$",
                    )
                    .unwrap(),
                    vec![Method::POST],
                ),(
                    #[allow(clippy::trivial_regex)]
                    Regex::new(
                        r"/account",
                    )
                    .unwrap(),
                    vec![Method::GET, Method::POST, Method::PUT, Method::DELETE],
                ),(
                    #[allow(clippy::trivial_regex)]
                    Regex::new(
                        r"/alias",
                    )
                    .unwrap(),
                    vec![Method::GET],
                )],
            })
            .wrap(middlewares::db_connection::DatabaseConnection {
                pool: appstate.clone().settings.db_connection.create_sqlite_connection_pool(),
            })
            .wrap({
                let cors = Cors::new();
                let cors = if let Some(host) = appstate.clone().settings.allowed_frontend {
                    cors.allowed_origin(&host)
                } else {
                    cors
                };
                cors
                    .allowed_methods(&[Method::GET, Method::POST, Method::PUT, Method::DELETE])
                    .supports_credentials()
                    .max_age(3600)
            })
            .wrap(actix_web::middleware::Logger::default())
            .wrap(actix_web_prom::PrometheusMetrics::new("api", "/metrics"))
            .service(web::resource("/health").to(|| actix_web::HttpResponse::Ok().finish()))
            .service(
                web::scope("/api/v1")
                    .service(handlers::account::get_scope())
                    .service(handlers::courses::get_scope())
                    .service(handlers::databases::get_scope())
                    .service(handlers::worksheets::get_scope())
                    .service(handlers::tasks::get_scope())
                    .service(handlers::subtasks::get_scope())
                    .service(handlers::alias::get_scope()),
            )
    });
    for addr in configuration.listen_addr {
        server = match server.bind(addr) {
            Ok(server) => server,
            Err(e) => {
                error!("Couldn't bind to {} because of {}", addr, e);
                return;
            }
        };
    }
    server.start();

    match sys.run() {
        Ok(_) => (),
        Err(e) => error!("Something went wrong starting the runtime: {}", e),
    }
}
