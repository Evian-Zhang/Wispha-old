use std::error::Error;
use std::fmt::{Display, Formatter, Debug, Result};
use std::path::PathBuf;

pub enum ManipulatorError {
    PathNotEntry(PathBuf),
    PathNotExist,
    Unexpected,
}

impl Error for ManipulatorError {

}

impl Display for ManipulatorError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "")
    }
}

impl Debug for ManipulatorError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "")
    }
}