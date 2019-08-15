use std::error::Error;
use std::fmt::{Display, Formatter, Debug, Result};

pub enum ParserError {
    AbsolutePathEmpty,
    NameEmpty,
    EntryFileTypeEmpty,
    UnrecognizedEntryFileType,
    InvalidPath,
    FileCannotRead,
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