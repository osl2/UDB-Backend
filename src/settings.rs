use config::{ConfigError, Config, File, Environment};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
}

impl Settings {
    pub fn new(config_file_path: &str) -> Result<Self, ConfigError> {
        let mut s = Config::new();
        s.merge(File::with_name(config_file_path))?;
        s.merge(Environment::with_prefix("udb"))?;
        s.try_into()
    }
}
