use std::error::Error;
use std::fmt::{Display, Formatter, Debug, Result};
use std::path::PathBuf;

pub enum GeneratorError {
    DirCannotRead(PathBuf),
    PathIsNotDir(PathBuf),
    NameNotDetermined(PathBuf),
    NameNotValid(PathBuf),
    IgnoreError(ignore::Error),
    Unexpected,
}

impl Error for GeneratorError {

}

impl Display for GeneratorError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "")
    }
}

impl Debug for GeneratorError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "")
    }
}

impl From<ignore::Error> for GeneratorError {
    fn from(err: ignore::Error) -> Self {
        GeneratorError::IgnoreError(err)
    }
}