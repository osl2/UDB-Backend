use serde::Deserialize;
// use diesel::{PgConnection, SqliteConnection, MysqlConnection, Connection, ConnectionResult};

/// The String specifies a filepath or URI for the DB Connection
#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum DatabaseConnectionConfig {
    #[serde(rename = "sqlite")]
    SQLiteFile { file: String },
    #[serde(rename = "memory")]
    SQLiteInMemory,
    #[serde(rename = "postgres")]
    Postgres{ uri: String },
    #[serde(rename = "mysql")]
    MySQL{ uri: String },
}

// impl DatabaseConnectionConfig {
//     pub fn establish_connection(&mut self) -> ConnectionResult<Connection> {
//         match self {
//             DatabaseConnectionConfig::SQLiteFile { file } => {
//                 SqliteConnection::establish(file)
//             },
//             DatabaseConnectionConfig::SQLiteInMemory => {
//                 SqliteConnection::establish(":memory:")
//             },
//             DatabaseConnectionConfig::Postgres { uri } => {
//                 PgConnection::establish(uri)
//             },
//             DatabaseConnectionConfig::MySQL { uri } => {
//                 MysqlConnection::establish(uri)
//             },
//         }
//     }
// }