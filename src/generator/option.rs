use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter, Debug};

use crate::commandline::Generate;
use crate::config_reader::{Config, PropertyConfig};
use crate::strings::*;

type Result<T> = std::result::Result<T, GeneratorOptionError>;

#[derive(Clone)]
pub struct GeneratorOptions {
    pub layer: GenerateLayer,
    pub allow_hidden_files: bool,
    pub properties: Vec<PropertyConfig>,
    pub ignored_files: Vec<String>,
    pub wispha_name: String,
}

#[derive(Clone, Copy)]
pub enum GenerateLayer {
    Flat,
    Recursive,
}

impl GeneratorOptions {
    pub fn default() -> GeneratorOptions {
        GeneratorOptions {
            layer: GenerateLayer::Flat,
            allow_hidden_files: false,
            properties: vec![],
            ignored_files: vec![],
            wispha_name: DEFAULT_FILE_NAME_STR.to_string(),
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
        if generate.all {
            self.allow_hidden_files = true;
        }
        Ok(())
    }

    pub fn update_from_config(&mut self, config: &Config) -> Result<()> {
        if let Some(generate_config) = &config.generate {
            if let Some(allow_hidden_file) = generate_config.allow_hidden_files {
                self.allow_hidden_files = allow_hidden_file;
            }
            if let Some(ignored_files) = &generate_config.ignored_files {
                self.ignored_files = ignored_files.clone();
            }
            if let Some(wispha_name) = &generate_config.wispha_name {
                self.wispha_name = wispha_name.clone();
            }
        }
        if let Some(properties) = &config.properties {
            self.properties = properties.clone();
        }
        Ok(())
    }

    fn validate_commandline(&self, generate: &Generate) -> Result<()> {
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