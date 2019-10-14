use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter, Debug};
use std::path::PathBuf;

#[derive(Debug)]
pub enum ManipulatorError {
    PathNotEntry(PathBuf),
    PathNotExist,
    AbsolutePathNotSupported,
    BeyondDomain,
    EntryNotFound(PathBuf),
    Unexpected,
}

impl Error for ManipulatorError { }

impl Display for ManipulatorError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use ManipulatorError::*;
        let error_message = match &self {
            PathNotEntry(path) => {
                format!("Cannot find entry in {}", path.to_str().unwrap())
            },
            PathNotExist => {
                format!("Path not exist!")
            },
            AbsolutePathNotSupported => {
                format!("Don't support absolute path.")
            },
            BeyondDomain => {
                format!("Path is beyond wispha domain.")
            },
            EntryNotFound(path) => {
                format!("Cannot find entry in {}", path.to_str().unwrap())
            },
            Unexpected => {
                format!("Unexpected error.")
            },
        };
        write!(f, "{}", error_message)
    }
}