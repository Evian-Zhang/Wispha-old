use std::path::PathBuf;
use std::fs;

use toml;
use serde::Deserialize;

pub mod error;
use error::ConfigError;

type Result<T> = std::result::Result<T, ConfigError>;

pub const CONFIG_FILE_NAME: &str = ".wispharc.toml";

#[derive(Deserialize)]
pub struct Config {
    pub generate: Option<GenerateConfig>,
    pub properties: Option<Vec<PropertyConfig>>,
}

#[derive(Deserialize, Clone)]
pub struct GenerateConfig {
    pub allow_hidden_files: Option<bool>,
}

#[derive(Deserialize, Clone)]
pub struct PropertyConfig {
    pub name: String,
    pub default_value: Option<String>,
}

impl Config {

}

pub fn read_configs_in_dir(dir: &PathBuf) -> Result<Option<Config>> {
    let mut path = dir.clone();
    path.push(CONFIG_FILE_NAME);
    read_configs_from_path(&path)
}

pub fn read_configs_from_path(path: &PathBuf) -> Result<Option<Config>> {
    match fs::read_to_string(path) {
        Ok(content) => {
            read_configs(content)
        },
        Err(_) => {
            Ok(None)
        }
    }
}

pub fn read_configs(content: String) -> Result<Option<Config>> {
    match toml::from_str::<Config>(&content) {
        Ok(config) => {
            Ok(Some(config))
        },
        Err(error) => {
            Err(ConfigError::DeserializeError(error))
        },
    }
}