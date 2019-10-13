use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter, Debug};

use crate::commandline::Generate;
use crate::config_reader::Config;

type Result<T> = std::result::Result<T, GeneratorOptionError>;

pub struct GeneratorOptions {
    pub layer: GenerateLayer,
}

pub enum GenerateLayer {
    Flat,
    Recursive,
}

impl GeneratorOptions {
    pub fn default() -> GeneratorOptions {
        GeneratorOptions {
            layer: GenerateLayer::Flat,
        }
    }

    pub fn from_commandline(generate: &Generate) -> Result<GeneratorOptions> {
        let mut result = GeneratorOptions::default();
        if generate.flat && generate.recursively {
            return Err(GeneratorOptionError::FlatAndRecursive);
        }
        if generate.recursively {
            result.layer = GenerateLayer::Recursive;
        }
        if generate.flat {
            result.layer = GenerateLayer::Flat;
        };
        Ok(result)
    }

    pub fn from_config(config: &Config) -> Result<GeneratorOptions> {

    }

    pub fn update(&mut self, options: GeneratorOptions) {

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