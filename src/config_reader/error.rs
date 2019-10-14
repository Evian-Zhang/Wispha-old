use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter, Debug};
use std::path::PathBuf;
use toml;

#[derive(Debug)]
pub enum ConfigError {
    DeserializeError(toml::de::Error),
}

impl Error for ConfigError { }

impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use ConfigError::*;
        match &self {
            DeserializeError(toml_error) => {
                let error_message = format!("{}", toml_error);
                write!(f, "{}", error_message)
            },
        }
    }
}