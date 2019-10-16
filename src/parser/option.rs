use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter, Debug};

use crate::config_reader::{Config, PropertyConfig};

type Result<T> = std::result::Result<T, ParserOptionError>;

pub struct ParserOptions {
    pub properties: Vec<PropertyConfig>,
}

impl ParserOptions {
    pub fn default() -> ParserOptions {
        ParserOptions {
            properties: vec![],
        }
    }

    pub fn update_from_config(&mut self, config: &Config) -> Result<()> {
        if let Some(properties) = &config.properties {
            self.properties = properties.clone();
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum ParserOptionError {

}

impl Error for ParserOptionError { }

impl Display for ParserOptionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use ParserOptionError::*;
//        match &self {
//
//        }
        write!(f, "")
    }
}