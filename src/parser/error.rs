use std::error::Error;
use std::fmt::{Display, Formatter, Debug, Result};
use std::path::PathBuf;

use crate::wispha::WisphaEntryType;

pub struct ParserErrorInfo {
    pub path: PathBuf,
    pub property: Option<String>,
}

pub enum ParserError {
    AbsolutePathEmpty(ParserErrorInfo),
    NameEmpty(ParserErrorInfo),
    EntryFileTypeEmpty(ParserErrorInfo),
    UnrecognizedEntryFileType(ParserErrorInfo, String),
    InvalidPath(ParserErrorInfo, PathBuf),
    FileCannotRead(PathBuf),
    DirectoryNotDetermined(PathBuf),
    Unexpected,
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