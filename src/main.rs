#[macro_use]
extern crate diesel;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use diesel::r2d2::{self, ConnectionManager};
use diesel::SqliteConnection;
use log::error;
use regex::Regex;
use uuid::Uuid;

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
    db_connection_pool: Option<r2d2::Pool<ConnectionManager<SqliteConnection>>>,
    current_user: Uuid,
}

impl AppData {
    pub fn from_configuration(config: settings::Settings) -> Self {
        let pool = config.db_connection.create_sqlite_connection_pool();
        Self {
            settings: config,
            db_connection_pool: pool,
            current_user: Uuid::parse_str("549b60cd-9b88-467b-9b1e-b15c68114c96").unwrap(), // test user
        }
    }

    pub fn get_db_connection(
        &self,
    ) -> Result<r2d2::PooledConnection<ConnectionManager<SqliteConnection>>, ()> {
        match &self.db_connection_pool {
            Some(pool) => match pool.get() {
                Ok(connection) => Ok(connection),
                Err(e) => Err(()),
            },
            None => Err(()),
        }
    }

    pub fn get_user(&self) -> Uuid {
        self.current_user
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
            .wrap(actix_web::middleware::Logger::default())
            .wrap(actix_web_prom::PrometheusMetrics::new("api", "/metrics"))
            .wrap(middlewares::upload_filter::UploadFilter { filter: false })
            .wrap(JwtAuthentication {
                key: JwtKey::Inline(jwt_key.clone()),
                algorithm: Algorithm::HS512,
                except: Regex::new(
                    r"(?P<uuid>[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})(/verify)?$",
                )
                .unwrap(),
            })
            .wrap(Cors::default())
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
