use crate::database::DatabaseConnectionConfig;
use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub(crate) open_registration: Option<bool>,
    pub(crate) jwt_key: String,
    pub(crate) http_timeout: Option<u64>,
    pub(crate) listen_addr: Vec<std::net::SocketAddr>,
    pub(crate) trusted_proxies: Option<Vec<std::net::IpAddr>>,
    pub(crate) db_connection: DatabaseConnectionConfig,
    pub(crate) allowed_frontend: Option<String>,
}

impl Settings {
    pub fn new(config_file_path: &str) -> Result<Self, ConfigError> {
        let mut s = Config::new();
        s.merge(File::with_name(config_file_path))?;
        s.merge(Environment::with_prefix("udb"))?;
        s.try_into()
    }
}
