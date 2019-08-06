use diesel::r2d2::{self, ConnectionManager};
use diesel::{Connection, PgConnection, SqliteConnection};
use serde::Deserialize;

/// The String specifies a filepath or URI for the DB Connection
#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum DatabaseConnectionConfig {
    #[serde(rename = "sqlite")]
    SQLiteFile { file: String },
    #[serde(rename = "memory")]
    SQLiteInMemory,
    #[serde(rename = "postgres")]
    Postgres { uri: String },
    #[serde(rename = "mysql")]
    MySQL { uri: String },
}

impl DatabaseConnectionConfig {
    pub fn create_sqlite_connection_pool(
        &self,
    ) -> Option<r2d2::Pool<ConnectionManager<SqliteConnection>>> {
        match self {
            DatabaseConnectionConfig::SQLiteFile { file } => Some(
                r2d2::Pool::builder()
                    .build(ConnectionManager::<SqliteConnection>::new(file.clone()))
                    .expect("Failed to create database connection Pool."),
            ),
            _ => None,
        }
    }
}
