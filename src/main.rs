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

                .service(handlers::account::get_account)
                .service(handlers::account::update_account)
                .service(handlers::account::create_account)
                .service(handlers::account::delete_account)
                .service(handlers::account::login)

                .service(handlers::courses::get_courses)
                .service(handlers::courses::create_course)
                .service(handlers::courses::get_course)
                .service(handlers::courses::update_course)
                .service(handlers::courses::delete_course)

                .service(handlers::databases::get_databases)
                .service(handlers::databases::create_database)
                .service(handlers::databases::get_database)
                .service(handlers::databases::update_database)
                .service(handlers::databases::delete_database)

                .service(handlers::worksheets::get_worksheets)
                .service(handlers::worksheets::create_worksheet)
                .service(handlers::worksheets::get_worksheet)
                .service(handlers::worksheets::update_worksheet)
                .service(handlers::worksheets::delete_worksheet)

                .service(handlers::tasks::get_tasks)
                .service(handlers::tasks::create_task)
                .service(handlers::tasks::get_task)
                .service(handlers::tasks::update_task)
                .service(handlers::tasks::delete_task)

                .service(handlers::subtasks::get_subtasks)
                .service(handlers::subtasks::create_subtask)
                .service(handlers::subtasks::get_subtask)
                .service(handlers::subtasks::update_subtask)
                .service(handlers::subtasks::delete_subtask)
                .service(handlers::subtasks::verify_subtask_solution)
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
