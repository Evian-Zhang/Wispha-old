use std::error::Error;
use std::fmt::{Display, Formatter, Debug, Result};

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
    pub fn from_commandline(generate: &Generate) -> Result<GeneratorOptions> {

    }

    pub fn from_config(config: &Config) -> Result<GeneratorOptions> {

    }

    pub fn update(&mut self, options: GeneratorOptions) {

    }
}

#[derive(Debug)]
pub enum GeneratorOptionError {

}

impl Error for GeneratorOptionError {

}

impl Display for GeneratorOptionError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "")
    }
}