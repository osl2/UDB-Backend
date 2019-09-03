#![warn(unused_extern_crates)]
use actix_cors::Cors;
use actix_web::{http::Method, web, App, HttpServer};
use log::error;
use regex::Regex;

use actix_web_jwt_middleware::{Algorithm, JwtAuthentication, JwtKey};

#[macro_use]
extern crate diesel_migrations;

pub use upowdb_models::{models, schema};

mod alias_generator;
mod cli;
mod database;
mod handlers;
mod logging;
mod middlewares;
mod settings;
mod solution_compare;

#[derive(Clone)]
struct AppData {
    settings: settings::Settings,
    database: database::Database,
}

impl AppData {
    pub fn from_configuration(
        config: settings::Settings,
    ) -> Result<Self, database::DatabaseConnectionError> {
        let database = config.db_connection.create_database()?;
        Ok(Self {
            settings: config,
            database,
        })
    }
}

#[cfg(all(feature = "postgres", not(feature = "sqlite")))]
embed_migrations!("migrations/postgresql");
#[cfg(all(feature = "sqlite", not(feature = "postgres")))]
embed_migrations!("migrations/sqlite");

#[cfg(any(feature = "sqlite", feature = "postgres"))]
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

    let appstate = match AppData::from_configuration(configuration.clone()) {
        Ok(appstate) => appstate,
        Err(error) => {
            match error {
                database::DatabaseConnectionError::IncompatibleBuild => {
                    error!("The database you configured is not compatible with this build.")
                }
                database::DatabaseConnectionError::Diesel(error) => error!(
                    "Something went wrong connecting to the database: {:?}",
                    error
                ),
                database::DatabaseConnectionError::R2D2(error) => error!(
                    "Something went wrong creating the connection pool: {:?}",
                    error
                ),
            };
            return;
        }
    };

    match embedded_migrations::run(&appstate.database.get_connection().unwrap()) {
        Ok(_) => {},
        Err(error) => {
            error!("Couldn't run database migrations: {:?}", error);
            return;
        }
    }
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
                database: appstate.database.clone()
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
            .wrap(Cors::default())
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
        Err(e) => error!("Something went wrong starting the runtime: {:?}", e),
    }
}
