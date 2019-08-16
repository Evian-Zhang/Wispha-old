use std::error::Error;
use std::fmt::{Display, Formatter, Debug, Result};

pub enum MainError {
    PathInvalid,
    Unexpect,
}

impl Error for MainError {

}

impl Display for MainError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "")
    }
}

impl Debug for MainError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "")
    }
}