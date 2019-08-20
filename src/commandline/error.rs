use std::error::Error;
use std::fmt::{Display, Formatter, Debug, Result};

pub enum CommandlineError {
    Unexpected,
}

impl Error for CommandlineError {

}

impl Display for CommandlineError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "")
    }
}

impl Debug for CommandlineError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "")
    }
}