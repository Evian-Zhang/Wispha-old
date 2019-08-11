use std::string::String;
use std::path::{Path, PathBuf};
use std::rc::{Rc, Weak};
use std::cell::RefCell;

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
    pub entry_file_path: Option<PathBuf>,
    pub sup_entry: RefCell<Weak<WisphaEntry>>,
    pub sub_entries: RefCell<Vec<Rc<WisphaEntry>>>,
}

//impl WisphaEntry {
//    pub fn new() -> WisphaEntry {
//        let properties = WisphaEntryProperties { entry_type: WisphaEntryType::File, name:
//        String::new(), description:
//        String::new(), absolute_path: PathBuf::new() };
//        let sub_entries: Vec<Box<WisphaEntry>> = Vec::new();
//        WisphaEntry { properties, entry_file_path: None, sup_entry: None, sub_entries }
//    }
//}

impl WisphaEntryType {
    pub fn to_str(&self) -> &'static str {
        match &self {
            WisphaEntryType::Directory => "Directory",
            WisphaEntryType::File => "File",
            WisphaEntryType::ProgramEntry => "Program entry",
        }
    }
}