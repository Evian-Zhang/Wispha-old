use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter, Debug};

#[derive(Debug)]
pub enum StatorError {
    IgnoreError(ignore::Error),
}

impl Error for StatorError { }

impl Display for StatorError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use StatorError::*;
        let message = match &self {
            IgnoreError(error) => {
                format!("{}", deal_with_ignore_error(error))
            }
        };
        write!(f, "{}", message)
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