use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter, Debug};

use crate::commandline::Generate;
use crate::config_reader::{Config, PropertyConfig};

type Result<T> = std::result::Result<T, GeneratorOptionError>;

pub struct GeneratorOptions {
    pub layer: GenerateLayer,
    pub properties: Vec<PropertyConfig>,
}

pub enum GenerateLayer {
    Flat,
    Recursive,
}

impl GeneratorOptions {
    pub fn default() -> GeneratorOptions {
        GeneratorOptions {
            layer: GenerateLayer::Flat,
            properties: vec![],
        }
    }

    pub fn update_from_commandline(&mut self, generate: &Generate) -> Result<()> {
        self.validate_commandline(generate)?;
        if generate.recursively {
            self.layer = GenerateLayer::Recursive;
        }
        if generate.flat {
            self.layer = GenerateLayer::Flat;
        };
        Ok(())
    }

    pub fn update_from_config(&mut self, config: &Config) -> Result<()> {
        if let Some(properties) = &config.properties {
            self.properties = properties.clone();
        }
        Ok(())
    }

    fn validate_commandline(&self, generate: &generate) -> Result<()> {
        if generate.flat && generate.recursively {
            return Err(GeneratorOptionError::FlatAndRecursive);
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum GeneratorOptionError {
    FlatAndRecursive,
}

impl Error for GeneratorOptionError { }

impl Display for GeneratorOptionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use GeneratorOptionError::*;
        match &self {
            FlatAndRecursive => {
                write!(f, "Cannot specify flat and recursively at same time.")
            },
        }
    }
}