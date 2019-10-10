use std::path::PathBuf;

use toml;
use serde::Deserialize;

mod error;
use error::ConfigError;

type Result<T> = std::result::Result<T, ConfigError>;

#[derive(Deserialize)]
pub struct Config {
    pub properties: Vec<PropertyConfig>,
}

#[derive(Deserialize)]
pub struct PropertyConfig {
    pub name: String,
    pub default_value: Option<String>,
}

pub fn read_configs(content: String) -> Result<Config> {
    match toml::from_str::<Config>(&content) {
        Ok(config) => {
            Ok(config)
        },
        Err(error) => {
            Err(ConfigError::DeserializeError(error))
        },
    }
}