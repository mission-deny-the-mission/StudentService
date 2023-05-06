use serde::Deserialize;
use config::ConfigError;

#[derive(Deserialize)]
pub struct Address {
    pub host: String,
    pub port: u16,
}

#[derive(Deserialize)]
pub struct Config {
    pub server: Address,
    pub database_url: String,
    pub finance_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let mut config_instance = config::Config::new();
        config_instance.merge(config::Environment::new())?;
        config_instance.try_into()
    }
}