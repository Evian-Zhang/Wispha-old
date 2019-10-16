use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter, Debug};
use std::path::PathBuf;
use std::rc::Rc;

use crate::parser::*;

#[derive(Debug)]
pub enum ParserError {
    UnrecognizedEntryFileType(Rc<WisphaToken>),
    FileCannotRead(PathBuf),
    UnexpectedToken(Rc<WisphaToken>, Option<Vec<(WisphaToken, Vec<WisphaExpectOption>)>>),
    EmptyBody(Rc<WisphaToken>),
    EnvNotFound,
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
        };
        write!(f, "{}", error_message)
    }
}