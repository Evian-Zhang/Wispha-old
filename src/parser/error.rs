use std::error::Error;
use std::fmt::{Display, Formatter, Debug, Result};
use std::path::PathBuf;
use std::rc::Rc;

use crate::wispha::WisphaEntryType;
use crate::parser::*;

pub struct ParserErrorInfo {
    pub path: PathBuf,
    pub property: Option<String>,
}

pub enum ParserError {
    UnrecognizedEntryFileType(Rc<WisphaToken>),
    FileCannotRead(PathBuf),
    UnexpectedToken(Rc<WisphaToken>, Option<Vec<(WisphaToken, Vec<WisphaExpectOption>)>>),
    EmptyBody(Rc<WisphaToken>),
    EnvNotFound,
}

impl Error for ParserError {

}

impl Display for ParserError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "")
    }
}

impl Debug for ParserError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "")
    }
}