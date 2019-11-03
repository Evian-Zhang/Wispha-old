use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter, Debug};

#[derive(Debug)]
pub enum SerializerError {
    SerializeError,
}

impl Error for SerializerError { }

impl Display for SerializerError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use SerializerError::*;
        let message = match &self {
            SerializeError => {
                format!("An unexpected error occurred when serializing.")
            }
        };
        write!(f, "{}", message)
    }
}
