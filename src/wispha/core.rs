use std::path::PathBuf;
use std::collections::HashMap;

use crate::strings::*;

#[derive(Copy, Clone)]
pub enum WisphaEntryType {
    Directory, // if the entry is a directory
    File, // if the entry is a file in a directory
    ProgramEntry, // if the entry is a programmatic stuff in a file
}

#[derive(Clone)]
pub struct WisphaEntryProperties {
    pub entry_type: WisphaEntryType,
    pub name: String,
    pub description: Option<String>, // the whitespace is not allowed at the begin and end
    pub absolute_path: PathBuf, // is absolute in memory, and starts with `$ROOT_DIR` when saved, can also be absolute or relative
    pub file_path: PathBuf, // the absolute path of the file where the entry is directly saved, i.e. not intermediate. Not saved in file
    pub customized: HashMap<String, String>,
}

impl WisphaEntryType {
    pub fn to_str(&self) -> &'static str {
        match &self {
            WisphaEntryType::Directory => DIRECTORY_TYPE,
            WisphaEntryType::File => FILE_TYPE,
            WisphaEntryType::ProgramEntry => PROGRAM_ENTRY_TYPE,
        }
    }

    pub fn from(string: String) -> Option<WisphaEntryType> {
        match string.as_str() {
            DIRECTORY_TYPE => Some(WisphaEntryType::Directory),
            FILE_TYPE => Some(WisphaEntryType::File),
            PROGRAM_ENTRY_TYPE => Some(WisphaEntryType::ProgramEntry),
            _ => None,
        }
    }
}