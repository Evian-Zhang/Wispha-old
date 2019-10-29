use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter, Debug};
use std::path::PathBuf;

use crate::helper::thread_pool::ThreadPoolError;

#[derive(Debug)]
pub enum GeneratorError {
    DirCannotRead(PathBuf),
    PathIsNotDir(PathBuf),
    NameNotDetermined(PathBuf),
    NameNotValid(PathBuf),
    IgnoreError(ignore::Error),
    FileCannotWrite(PathBuf),
    ThreadPoolError(ThreadPoolError),
    Unexpected,
}

impl Error for GeneratorError { }

impl Display for GeneratorError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use GeneratorError::*;
        let error_message = match &self {
            DirCannotRead(path) => {
                format!("Cannot read directory {}", path.to_str().unwrap())
            },
            PathIsNotDir(path) => {
                format!("Path {} is not a directory.", path.to_str().unwrap())
            },
            NameNotDetermined(path) => {
                format!("Cannot determine the entry name of {}.", path.to_str().unwrap())
            },
            NameNotValid(path) => {
                format!("Path {} contains invalid characters.", path.to_str().unwrap())
            },
            IgnoreError(ignore_error) => {
                format!("{}", deal_with_ignore_error(ignore_error))
            }
            Unexpected => {
                format!("Unexpected error. Please retry.")
            },
            FileCannotWrite(path) => {
                format!("Cannot write to file {}. Permission denied.", path.to_str().unwrap())
            },
            ThreadPoolError(error) => {
                format!("{}", error)
            }
        };
        write!(f, "{}", error_message)
    }
}

impl From<ignore::Error> for GeneratorError {
    fn from(err: ignore::Error) -> Self {
        GeneratorError::IgnoreError(err)
    }
}

impl From<ThreadPoolError> for GeneratorError {
    fn from(err: ThreadPoolError) -> Self {
        GeneratorError::ThreadPoolError(err)
    }
}

fn deal_with_ignore_error(ignore_error: &ignore::Error) -> String {
    match ignore_error {
        ignore::Error::Partial(errors) => {
            let mut error_messages = vec![];
            for error in errors {
                error_messages.push(deal_with_ignore_error(&error));
            }
            error_messages.join("\n")
        },
        ignore::Error::WithLineNumber { line, err } => {
            let error_message = format!("in line {} ", line);
            error_message + deal_with_ignore_error(&*err).as_str()
        },
        ignore::Error::WithPath { path, err } => {
            let error_message = format!("in the file {} ", path.to_str().unwrap());
            error_message + deal_with_ignore_error(&*err).as_str()
        },
        ignore::Error::WithDepth { depth, err } => {
            let error_message = format!("to the depth {} ", depth);
            error_message + deal_with_ignore_error(&*err).as_str()
        },
        ignore::Error::Loop { ancestor, child } => {
            format!("A dead loop occurred because of the {} in {}.",
                         child.to_str().unwrap(),
                         ancestor.to_str().unwrap())
        },
        ignore::Error::Io(_) => {
            format!("IO error. May be lack permission")
        },
        ignore::Error::Glob { glob, err } => {
            let default_value = "".to_string();
            let glob = glob.as_ref().unwrap_or(&default_value);
            format!("An error occurred when parsing {}, because {}", glob, err)
        },
        _ => {
            String::new()
        }
    }
}