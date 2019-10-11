use std::error::Error;
use std::fmt::{Display, Formatter, Debug, Result};
use std::path::PathBuf;
use toml;

pub enum ConfigError {
    DeserializeError(toml::de::Error),
}

impl Error for ConfigError {

}

impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "")
    }
}

impl Debug for ConfigError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "")
    }
}