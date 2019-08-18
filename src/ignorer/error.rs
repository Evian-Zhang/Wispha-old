use std::error::Error;
use std::fmt::{Display, Formatter, Debug, Result};

pub enum IgnoreError {
    Syntax,
    Unexpect,
}

impl Error for IgnoreError {

}

impl Display for IgnoreError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "")
    }
}

impl Debug for IgnoreError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "")
    }
}
