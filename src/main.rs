#[macro_use]
extern crate diesel;

use actix_web::{web, App, HttpServer};
use log::error;
use diesel::r2d2::{self, ConnectionManager};
use diesel::SqliteConnection;

mod settings;
mod cli;
mod logging;
mod database;
mod handlers;
mod models;
mod schema;

#[derive(Clone)]
struct AppData {
    settings: settings::Settings,
    db_connection_pool: Option<r2d2::Pool<ConnectionManager<SqliteConnection>>>,
}

impl AppData {
    pub fn from_configuration(config: settings::Settings) -> Self {
        let pool = config.db_connection.create_sqlite_connection_pool();
        Self {
            settings: config,
            db_connection_pool: pool,
        }
    }

    pub fn get_db_connection(&self) -> Result<r2d2::PooledConnection<ConnectionManager<SqliteConnection>>, ()> {
        match &self.db_connection_pool {
            Some(pool) => {
                match pool.get() {
                    Ok(connection) => { Ok(connection) },
                    Err(e) => Err(())
                }
            },
            None => Err(())
        }
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
    let configuration = settings::Settings::new(cli_matches.value_of("config").unwrap_or("config.toml")).unwrap();
    let sys = actix::System::new("udb-backend");
    let prometheus = actix_web_prom::PrometheusMetrics::new("api", "/metrics");

    let appstate = AppData::from_configuration(configuration.clone());
    let mut server = HttpServer::new(move || {
        App::new()
            .data(appstate.clone())
            .wrap(actix_web::middleware::Logger::default())
            .wrap(prometheus.clone())
            .service(web::resource("/health").to(|| actix_web::HttpResponse::Ok().finish()))
            .service(
                web::scope("/api/v1")
                .service(handlers::account::get_scope())
                .service(handlers::courses::get_scope())
                .service(handlers::databases::get_scope())
                .service(handlers::worksheets::get_scope())
                .service(handlers::tasks::get_scope())
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
