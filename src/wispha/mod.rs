use std::string::String;
use std::path::PathBuf;
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::collections::HashMap;

use crate::strings::*;

pub mod core;
pub mod intermediate;
pub mod common;

//#[derive(Copy, Clone)]
//pub enum WisphaEntryType {
//    Directory, // if the entry is a directory
//    File, // if the entry is a file in a directory
//    ProgramEntry, // if the entry is a programmatic stuff in a file
//}
//
//#[derive(Clone)]
//pub struct WisphaEntryProperties {
//    pub entry_type: WisphaEntryType,
//    pub name: String,
//    pub description: Option<String>, // the whitespace is not allowed at the begin and end
//    pub absolute_path: PathBuf, // is absolute in memory, and starts with `$ROOT_DIR` when saved, can also be absolute or relative
//    pub file_path: PathBuf, // the absolute path of the file where the entry is directly saved, i.e. not intermediate. Not saved in file
//    pub customized: HashMap<String, String>,
//}
//
//// like soft link
//#[derive(Clone)]
//pub struct WisphaIntermediateEntry {
//    pub entry_file_path: PathBuf, // tells where to find the actual file. The path can be absolute, relative or start with `$ROOT_DIR`
//}
//
//// the structure used as tree node
//#[derive(Clone)]
//pub enum WisphaFatEntry {
//    Intermediate(WisphaIntermediateEntry),
//    Immediate(WisphaEntry),
//}
//
//pub struct WisphaEntry {
//    pub properties:  WisphaEntryProperties,
//    pub sup_entry: RefCell<Weak<RefCell<WisphaFatEntry>>>, // for the root node, `*sup_entry` is Weak::new()
//    pub sub_entries: RefCell<Vec<Rc<RefCell<WisphaFatEntry>>>>,
//}
//
//impl WisphaEntryType {
//    pub fn to_str(&self) -> &'static str {
//        match &self {
//            WisphaEntryType::Directory => DIRECTORY_TYPE,
//            WisphaEntryType::File => FILE_TYPE,
//            WisphaEntryType::ProgramEntry => PROGRAM_ENTRY_TYPE,
//        }
//    }
//
//    pub fn from(string: String) -> Option<WisphaEntryType> {
//        match string.as_str() {
//            DIRECTORY_TYPE => Some(WisphaEntryType::Directory),
//            FILE_TYPE => Some(WisphaEntryType::File),
//            PROGRAM_ENTRY_TYPE => Some(WisphaEntryType::ProgramEntry),
//            _ => None,
//        }
//    }
//}
//
//impl WisphaFatEntry {
//    pub fn get_immediate_entry(&self) -> Option<&WisphaEntry> {
//        if let WisphaFatEntry::Immediate(entry) = &self {
//            return Some(entry);
//        }
//        None
//    }
//
//    pub fn get_immediate_entry_mut(&mut self) -> Option<&mut WisphaEntry> {
//        if let WisphaFatEntry::Immediate(entry) = self {
//            return Some(entry);
//        }
//        None
//    }
//
//    pub fn get_intermediate_entry(&self) -> Option<&WisphaIntermediateEntry> {
//        if let WisphaFatEntry::Intermediate(entry) = &self {
//            return Some(entry);
//        }
//        None
//    }
//
//    pub fn get_intermediate_entry_mut(&mut self) -> Option<&mut WisphaIntermediateEntry> {
//        if let WisphaFatEntry::Intermediate(entry) = self {
//            return Some(entry);
//        }
//        None
//    }
//}
//
//impl WisphaEntry {
//    pub fn default() -> WisphaEntry {
//        let properties = WisphaEntryProperties {
//            entry_type: DEFAULT_ENTRY_TYPE,
//            name: String::from(DEFAULT_NAME),
//            description: None,
//            absolute_path: PathBuf::from(DEFAULT_PATH),
//            file_path: PathBuf::from(DEFAULT_FILE_PATH),
//            customized: HashMap::new(),
//        };
//
//        let sup_entry: RefCell<Weak<RefCell<WisphaFatEntry>>> = RefCell::new(Weak::new());
//
//        let sub_entries: RefCell<Vec<Rc<RefCell<WisphaFatEntry>>>> = RefCell::new(Vec::new());
//
//        WisphaEntry {
//            properties,
//            sup_entry,
//            sub_entries
//        }
//    }
//}
//
//impl Clone for WisphaEntry {
//    fn clone(&self) -> Self {
//        let cloned = WisphaEntry {
//            properties: self.properties.clone(),
//            sup_entry: self.sup_entry.clone(),
//            sub_entries: RefCell::new(Vec::new()),
//        };
//        for sub_entry in &*self.sub_entries.borrow() {
//            cloned.sub_entries.borrow_mut().push(Rc::new((**sub_entry).clone()));
//        }
//        cloned
//    }
//}