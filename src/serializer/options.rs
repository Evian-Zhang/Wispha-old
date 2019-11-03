use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter, Debug};

use crate::strings::*;
use crate::commandline::Convert;

type Result<T> = std::result::Result<T, SerializerOptionError>;

#[derive(Clone)]
pub struct SerializerOptions {
    pub language: Language,
}

impl SerializerOptions {
    pub fn default() -> SerializerOptions {
        SerializerOptions {
            language: DEFAULT_SERIALIZE_LANGUAGE,
        }
    }

    pub fn update_from_commandline(&mut self, convert: &Convert) -> Result<()> {
        if let Some(language_str) = &convert.language {
            self.language = Language::from(language_str)?;
        }
        Ok(())
    }
}

#[derive(Clone)]
pub enum Language {
    JSON,
    TOML,
}

impl Language {
    pub fn from(language_str: &str) -> Result<Language> {
        use Language::*;
        match language_str {
            "JSON" => Ok(JSON),
            "TOML" => Ok(TOML),
            _ => Err(SerializerOptionError::LanguageNotSupport(language_str.to_string()))
        }
    }

    pub fn to_extension(&self) -> String {
        use Language::*;
        match &self {
            JSON => String::from("json"),
            TOML => String::from("toml"),
        }
    }
}

#[derive(Debug)]
pub enum SerializerOptionError {
    CurrentDirNotDetermined,
    LanguageNotSupport(String),
}

impl Error for SerializerOptionError { }

impl Display for SerializerOptionError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use SerializerOptionError::*;
        let message = match &self {
            CurrentDirNotDetermined => {
                format!("Can")
            },
            LanguageNotSupport(language) => {
                format!("{} is not supported.", language)
            },
        };
        write!(f, "{}", message)
    }
}