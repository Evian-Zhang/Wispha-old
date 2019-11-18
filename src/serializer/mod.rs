pub mod options;
pub mod error;

use options::{SerializerOptions, Language};
use error::SerializerError;

use crate::wispha::common::WisphaEntry;
use crate::strings::*;

use serde::ser::{Serialize, Serializer, SerializeMap};

use std::rc::Rc;
use std::cell::RefCell;

type Result<T> = std::result::Result<T, SerializerError>;

impl Serialize for WisphaEntry {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<<S as Serializer>::Ok, <S as Serializer>::Error>
        where
            S: Serializer
    {
        let mut wispha = serializer.serialize_map(None)?;
        wispha.serialize_entry(NAME_HEADER, &self.properties.name)?;
        wispha.serialize_entry(ABSOLUTE_PATH_HEADER, &self.properties.absolute_path.to_str().unwrap())?;
        wispha.serialize_entry(ENTRY_TYPE_HEADER, &self.properties.entry_type.to_str())?;
        if let Some(description) = &self.properties.description {
            wispha.serialize_entry(DESCRIPTION_HEADER, description)?;
        }
        for (header, body) in &self.properties.customized {
            wispha.serialize_entry(header, body)?;
        }
        if !self.sub_entries.borrow().is_empty() {
            wispha.serialize_entry(SUB_ENTRIES_HEADER, &self.sub_entries)?;
        }
        if !self.dependencies.borrow().is_empty() {
            wispha.serialize_entry(DEPENDENCY_HEADER, &self.dependencies)?;
        }
        wispha.end()
    }
}

pub fn serialize(entry: Rc<RefCell<WisphaEntry>>, options: SerializerOptions) -> Result<String> {
    match &options.language {
        Language::JSON => {
            convert_to_json(entry)
        },
        Language::TOML => {
            convert_to_toml(entry)
        }
    }
}

fn convert_to_json(entry: Rc<RefCell<WisphaEntry>>) -> Result<String> {
    serde_json::to_string(&entry).or(Err(SerializerError::SerializeError))
}

fn convert_to_toml(entry: Rc<RefCell<WisphaEntry>>) -> Result<String> {
    toml::to_string(&entry).or(Err(SerializerError::SerializeError))
}
