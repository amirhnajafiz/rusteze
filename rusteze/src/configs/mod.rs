use config::{Config, Environment, File, ConfigError};
use serde::Deserialize;

// Prefix for environment variables.
const PREFIX: &str = "RUSTEZE";

// AppConfig struct to hold the application configuration.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub log_level: String,
    pub data_dir: String,
    pub metrics_port: u16,
    pub snapshot_interval: u64,
}

// Provide default values for AppConfig.
impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            host: "127.0.0.1".to_string(),
            log_level: "info".to_string(),
            data_dir: "./data".to_string(),
            port: 8080,
            metrics_port: 9090,
            snapshot_interval: 300, // 5 minutes
        }
    }
}

// load_config function to read configuration from file and
// environment variables.
pub fn load_config(path: &str) -> Result<AppConfig, ConfigError> {
    let config = Config::builder()
        .add_source(File::with_name(path).required(false))
        .add_source(Environment::with_prefix(PREFIX).separator("__"))
        .build()?;

    config.try_deserialize()
}
