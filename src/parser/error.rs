use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter, Debug};
use std::path::PathBuf;

use crate::parser::parser_struct::{WisphaToken, WisphaExpectOption};
use crate::helper::thread_pool::ThreadPoolError;

#[derive(Debug)]
pub enum ParserError {
    UnrecognizedEntryFileType(WisphaToken),
    FileCannotRead(PathBuf),
    UnexpectedToken(WisphaToken, Option<Vec<(WisphaToken, Vec<WisphaExpectOption>)>>),
    EmptyBody(WisphaToken),
    EnvNotFound,
    ThreadPoolError(ThreadPoolError),
    DependencyNotFound(PathBuf),
    Unexpected,
}

impl Error for ParserError { }

impl Display for ParserError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use ParserError::*;
        let error_message = match &self {
            UnrecognizedEntryFileType(token) => {
                format!("In file {}, line {}:\nUnrecognized entry file type {}.",
                             token.raw_token().file_path.to_str().unwrap(),
                             token.raw_token().line_number,
                             token.raw_token().content.clone())
            },
            FileCannotRead(path) => {
                format!("Cannot read file {}.", path.to_str().unwrap())
            },
            UnexpectedToken(token, _) => {
                format!("In file {}, line {}:\nUnexpected token {}",
                             token.raw_token().file_path.to_str().unwrap(),
                             token.raw_token().line_number,
                             token.raw_token().content.clone())
            },
            EmptyBody(token) => {
                format!("In file {}, line {}:\nProperty {} has empty body.",
                             token.raw_token().file_path.to_str().unwrap(),
                             token.raw_token().line_number,
                             token.raw_token().content.clone())
            },
            EnvNotFound => {
                format!("Cannot determine the environment variable.")
            },
            ThreadPoolError(error) => {
                format!("{}", error)
            },
            DependencyNotFound(path) => {
                format!("Cannot find dependency at path {}", path.to_str().unwrap())
            },
            Unexpected => {
                format!("Unexpected error. Please retry.")
            },
        };
        write!(f, "{}", error_message)
    }
}

impl From<ThreadPoolError> for ParserError {
    fn from(err: ThreadPoolError) -> Self {
        ParserError::ThreadPoolError(err)
    }
}