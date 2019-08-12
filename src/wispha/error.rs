use std::error::Error;
use std::fmt::{Display, Formatter, Debug, Result};

pub enum WisphaError {
    Unexpect,
}

impl Error for WisphaError {

}

impl Display for WisphaError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "")
    }
}

impl Debug for WisphaError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "")
    }
}