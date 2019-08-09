use std::string::String;
use std::path::{Path, PathBuf};

pub enum WisphaEntryType {
    Directory,
    File,
    ProgramEntry,
}

pub struct WisphaEntryProperties {
    pub entry_type: WisphaEntryType,
    pub name: String,
    pub description: String,
    pub absolute_path: PathBuf,
}

pub struct WisphaEntry {
    pub properties:  WisphaEntryProperties,
    pub sub_entries: Vec<Box<WisphaEntry>>,
}

impl WisphaEntry {
    pub fn new() -> WisphaEntry {
        let properties = WisphaEntryProperties { entry_type: WisphaEntryType::File, name:
        String::new(), description:
        String::new(), absolute_path: PathBuf::new() };
        let sub_entries: Vec<Box<WisphaEntry>> = Vec::new();
        WisphaEntry { properties, sub_entries }
    }
}