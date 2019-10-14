use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter, Debug};
use std::path::PathBuf;

#[derive(Debug)]
pub enum GeneratorError {
    DirCannotRead(PathBuf),
    PathIsNotDir(PathBuf),
    NameNotDetermined(PathBuf),
    NameNotValid(PathBuf),
    IgnoreError(ignore::Error),
    FileCannotWrite(PathBuf),
    Unexpected,
}

impl Error for GeneratorError { }

impl Display for GeneratorError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use GeneratorError::*;
        match &self {
            DirCannotRead(path) => {
                let error_message = String::from("Cannot read directory {}.", path.to_str().unwrap());
            },
            PathIsNotDir(path) => {
                let error_message = String::from("Path {} is not a directory.", path.to_str().unwrap());
            },
            NameNotDetermined(path) => {
                let error_message = String::from("Cannot determine the entry name of {}.", path.to_str().unwrap());
            },
            NameNotValid(path) => {
                let error_message = String::from("Path {} contains invalid characters.", path.to_str().unwrap());
            },
            IgnoreError(ignore_error) => {
                deal_with_ignore_error(&ignore_error);
            }
            Unexpected => {
                let error_message = String::from("Unexpected error. Please retry.");
            },
            FileCannotWrite(path) => {
                let error_message = String::from("Cannot write to file {}. Permission denied.", path.to_str().unwrap());
            }
        }
    }
}

impl From<ignore::Error> for GeneratorError {
    fn from(err: ignore::Error) -> Self {
        GeneratorError::IgnoreError(err)
    }
}