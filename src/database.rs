use diesel::r2d2::{self, ConnectionManager};
use diesel::PgConnection;
use serde::Deserialize;

/// The String specifies a filepath or URI for the DB Connection
#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum DatabaseConnectionConfig {
    #[serde(rename = "pg")]
    PgFile { file: String },
    #[serde(rename = "memory")]
    PgInMemory,
}

impl DatabaseConnectionConfig {
    pub fn create_pg_connection_pool(&self) -> r2d2::Pool<ConnectionManager<PgConnection>> {
        match self {
            DatabaseConnectionConfig::PgFile { file } => pg(&file.clone()),
            DatabaseConnectionConfig::PgInMemory => pg(":memory:"),
        }
    }
}

fn pg(file: &str) -> r2d2::Pool<ConnectionManager<PgConnection>> {
    r2d2::Pool::builder()
        .max_size(20)
        .build(ConnectionManager::<PgConnection>::new(file))
        .expect("Failed to create database connection Pool.")
}
