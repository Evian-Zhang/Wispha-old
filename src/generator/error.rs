use std::error::Error;
use std::fmt::{Display, Formatter, Debug, Result};
use std::io;

pub enum GeneratorError {
    DirCannotRead,
    PathIsNotDir,
    NameNotDetermined,
    NameNotValid,
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