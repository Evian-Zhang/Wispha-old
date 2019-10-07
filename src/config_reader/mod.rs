use std::path::PathBuf;

use toml;
//use toml::de::Deserializer;
//use serde_derive::Deserialize;
use serde::de::Deserialize;

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
    toml::from_str(&content).or_else(|error| ConfigError::DeserializeError(error))?
}