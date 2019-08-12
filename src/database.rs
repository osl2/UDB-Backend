use diesel::r2d2::{self, ConnectionManager};
use diesel::SqliteConnection;
use serde::Deserialize;

/// The String specifies a filepath or URI for the DB Connection
#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum DatabaseConnectionConfig {
    #[serde(rename = "sqlite")]
    SQLiteFile { file: String },
    #[serde(rename = "memory")]
    SQLiteInMemory,
}

impl DatabaseConnectionConfig {
    pub fn create_sqlite_connection_pool(&self) -> r2d2::Pool<ConnectionManager<SqliteConnection>> {
        match self {
            DatabaseConnectionConfig::SQLiteFile { file } => sqlite(&file.clone()),
            DatabaseConnectionConfig::SQLiteInMemory => sqlite(":memory:"),
        }
    }
}

fn sqlite(file: &str) -> r2d2::Pool<ConnectionManager<SqliteConnection>> {
    r2d2::Pool::builder()
        .max_size(15)
        .build(ConnectionManager::<SqliteConnection>::new(file))
        .expect("Failed to create database connection Pool.")
}
