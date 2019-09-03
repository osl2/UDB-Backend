use diesel::r2d2::ConnectionManager;
use serde::Deserialize;

#[cfg(any(all(feature = "sqlite", feature = "postgres"), all(not(feature = "sqlite"), not(feature = "postgres"))))]
compile_error!("features `sqlite` and `postgres` are mutually exclusive");


#[cfg(feature = "sqlite")]
pub type DatabaseConnection = diesel::SqliteConnection;

#[cfg(feature = "postgres")]
pub type DatabaseConnection = diesel::PgConnection;

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
}

pub enum DatabaseConnectionError {
    IncompatibleBuild,
    Diesel(diesel::ConnectionError),
    R2D2(r2d2::Error),
}

impl From<r2d2::Error> for DatabaseConnectionError {
    fn from(error: r2d2::Error) -> DatabaseConnectionError {
        DatabaseConnectionError::R2D2(error)
    }
}

impl From<diesel::ConnectionError> for DatabaseConnectionError {
    fn from(error: diesel::ConnectionError) -> DatabaseConnectionError {
        DatabaseConnectionError::Diesel(error)
    }
}

impl DatabaseConnectionConfig {
    #[cfg(any(feature = "sqlite", feature = "postgres"))]
    pub fn create_connection_pool(&self) -> Result<r2d2::Pool<ConnectionManager<DatabaseConnection>>, DatabaseConnectionError> {
    pub fn create_database(&self) -> Result<Database, DatabaseConnectionError> {
        match self {
            DatabaseConnectionConfig::SQLiteFile { file } => {
                if cfg!(feature = "sqlite") {
                    Ok(Database::new(r2d2::Pool::builder().max_size(15).build(
                        ConnectionManager::<DatabaseConnection>::new(&file.clone()),
                    )?))
                } else {
                    Err(DatabaseConnectionError::IncompatibleBuild)
                }
            }
            DatabaseConnectionConfig::SQLiteInMemory => {
                if cfg!(feature = "sqlite") {
                    Ok(Database::new(r2d2::Pool::builder().max_size(15).build(
                        ConnectionManager::<DatabaseConnection>::new(":memory:"),
                    )?))
                } else {
                    Err(DatabaseConnectionError::IncompatibleBuild)
                }
            }
            DatabaseConnectionConfig::Postgres { uri } => {
                if cfg!(feature = "postgres") {
                    Ok(Database::new(r2d2::Pool::builder().max_size(15).build(
                        ConnectionManager::<DatabaseConnection>::new(uri),
                    )?))
                } else {
                    Err(DatabaseConnectionError::IncompatibleBuild)
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct Database {
    pool: r2d2::Pool<ConnectionManager<DatabaseConnection>>,
}

impl Database {
    pub fn new(pool: r2d2::Pool<ConnectionManager<DatabaseConnection>>) -> Self {
        Self { pool }
    }
    pub fn get_connection(
        &self,
    ) -> Result<r2d2::PooledConnection<ConnectionManager<DatabaseConnection>>, r2d2::Error> {
        self.pool.get()
    }
}

#[derive(Debug)]
pub enum DatabaseError {
    R2D2(r2d2::Error),
    Diesel(diesel::result::Error),
    Other(OtherErrorKind),
}

impl From<r2d2::Error> for DatabaseError {
    fn from(error: r2d2::Error) -> Self {
        Self::R2D2(error)
    }
}

impl From<diesel::result::Error> for DatabaseError {
    fn from(error: diesel::result::Error) -> Self {
        Self::Diesel(error)
    }
}

#[derive(Debug)]
pub enum OtherErrorKind {
}
