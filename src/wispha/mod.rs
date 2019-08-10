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

pub struct WisphaEntry<'a> {
    pub properties:  WisphaEntryProperties,
    pub entry_file_path: Option<PathBuf>,
    pub sup_entry: Option<& 'a WisphaEntry<'a>>,
    pub sub_entries: Vec<Box<WisphaEntry<'a>>>,
}

impl<'a> WisphaEntry<'a> {
    pub fn new() -> WisphaEntry<'a> {
        let properties = WisphaEntryProperties { entry_type: WisphaEntryType::File, name:
        String::new(), description:
        String::new(), absolute_path: PathBuf::new() };
        let sub_entries: Vec<Box<WisphaEntry>> = Vec::new();
        WisphaEntry { properties, entry_file_path: None, sup_entry: None, sub_entries }
    }
}