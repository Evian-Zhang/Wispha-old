use std::path::PathBuf;
use std::fs;

use toml;
use serde::Deserialize;

pub mod error;
use error::ConfigError;

type Result<T> = std::result::Result<T, ConfigError>;

use crate::strings::*;

#[derive(Deserialize)]
pub struct Config {
    pub generate: Option<GenerateConfig>,
    pub properties: Option<Vec<PropertyConfig>>,
}

#[derive(Deserialize, Clone)]
pub struct GenerateConfig {
    pub allow_hidden_files: Option<bool>,
    pub ignored_files: Option<Vec<String>>,
}

#[derive(Deserialize, Clone)]
pub struct PropertyConfig {
    pub name: String,
    pub default_value: Option<String>,
}

pub fn read_configs_in_dir(dir: &PathBuf) -> Result<Option<Config>> {
    let path = dir.join(CONFIG_FILE_NAME);
    read_configs_from_path(&path)
}

// If there is none or can't open, return Ok(None); If the toml deserialize fails, return Err
pub fn read_configs_from_path(path: &PathBuf) -> Result<Option<Config>> {
    match fs::read_to_string(path) {
        Ok(content) => {
            Ok(Some(read_configs(content)?))
        },
        Err(_) => {
            Ok(None)
        }
    }
}

pub fn read_configs(content: String) -> Result<Config> {
    toml::from_str::<Config>(&content).or_else(|error| Err(ConfigError::DeserializeError(error)))
}