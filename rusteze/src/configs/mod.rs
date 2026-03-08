use config::{Config, Environment, File, ConfigError};
use serde::Deserialize;

// AppConfig struct to hold the application configuration.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
}

// Provide default values for AppConfig.
impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            host: "localhost".to_string(),
            port: 8080,
        }
    }
}

// load_config function to read configuration from file and
// environment variables.
pub fn load_config() -> Result<AppConfig, ConfigError> {
    let config = Config::builder()
        .add_source(File::with_name("config").required(false))
        .add_source(Environment::with_prefix("RUSTEZE").separator("__"))
        .build()?;

    config.try_deserialize()
}
