use std::string::String;
use std::path::{Path, PathBuf};
use std::rc::{Rc, Weak};
use std::cell::RefCell;

mod error;
use error::WisphaError;
use crate::wispha::WisphaEntryType::Directory;

type Result<T> = std::result::Result<T, WisphaError>;

const DEFAULT_ENTRY_TYPE: WisphaEntryType = WisphaEntryType::File;
const DEFAULT_NAME: &str = "default name";
const DEFAULT_DESCRIPTION: &str = "default description";
const DEFAULT_PATH: &str = "default path";
const DEFAULT_FILE_PATH: &str = "default file path";

const DEFAULT_FILE_NAME_STR: &str = "LOOKME.wispha";

pub const LINE_SEPARATOR: &str = "\n";

pub const BEGIN_MARK: &str = "+";

const DIRECTORY_TYPE: &str = "directory";
const FILE_TYPE: &str = "file";
const PROGRAM_ENTRY_TYPE: &str = "program entry";

pub const ABSOLUTE_PATH_HEADER: &str = "absolute path";
pub const NAME_HEADER: &str = "name";
pub const ENTRY_TYPE_HEADER: &str = "entry type";
pub const DESCRIPTION_HEADER: &str = "description";

pub const ENTRY_FILE_PATH_HEADER: &str = "entry file path";
pub const SUB_ENTRIES_HEADER: &str = "subentry";

pub const ROOT_DIR: &str = "$ROOT_DIR";
pub const ROOT_DIR_VAR: &str = "WISPHA_ROOT_DIR";

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
    pub file_path: PathBuf,
}

pub struct WisphaIntermediateEntry {
    pub entry_file_path: PathBuf,
}

pub enum WisphaFatEntry {
    Intermediate(WisphaIntermediateEntry),
    Immediate(WisphaEntry),
}

pub struct WisphaEntry {
    pub properties:  WisphaEntryProperties,
    pub sup_entry: RefCell<Weak<RefCell<WisphaFatEntry>>>,
    pub sub_entries: RefCell<Vec<Rc<RefCell<WisphaFatEntry>>>>,
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

impl Copy for WisphaEntryType { }

impl Clone for WisphaEntryType {
    fn clone(&self) -> WisphaEntryType {
        *self
    }
}

impl Clone for WisphaEntryProperties {
    fn clone(&self) -> Self {
        WisphaEntryProperties {
            entry_type: self.entry_type,
            name: self.name.clone(),
            description: self.description.clone(),
            absolute_path: self.absolute_path.clone(),
            file_path:self.file_path.clone(),
        }
    }
}

impl Clone for WisphaIntermediateEntry {
    fn clone(&self) -> Self {
        WisphaIntermediateEntry {
            entry_file_path: self.entry_file_path.clone(),
        }
    }
}

impl WisphaFatEntry {
    pub fn get_immediate_entry(&self) -> Option<&WisphaEntry> {
        if let WisphaFatEntry::Immediate(entry) = &self {
            return Some(entry);
        }
        None
    }

    pub fn get_immediate_entry_mut(&mut self) -> Option<&mut WisphaEntry> {
        if let WisphaFatEntry::Immediate(entry) = self {
            return Some(entry);
        }
        None
    }

    pub fn get_intermediate_entry(&self) -> Option<&WisphaIntermediateEntry> {
        if let WisphaFatEntry::Intermediate(entry) = &self {
            return Some(entry);
        }
        None
    }

    pub fn get_intermediate_entry_mut(&mut self) -> Option<&mut WisphaIntermediateEntry> {
        if let WisphaFatEntry::Intermediate(entry) = self {
            return Some(entry);
        }
        None
    }
}

impl Clone for WisphaFatEntry {
    fn clone(&self) -> Self {
        match &self {
            WisphaFatEntry::Intermediate(entry) => WisphaFatEntry::Intermediate(entry.clone()),
            WisphaFatEntry::Immediate(entry) => WisphaFatEntry::Immediate(entry.clone()),
        }
    }
}

impl WisphaEntry {
    pub fn default() -> WisphaEntry {
        let properties = WisphaEntryProperties {
            entry_type: DEFAULT_ENTRY_TYPE,
            name: String::from(DEFAULT_NAME),
            description: String::from(DEFAULT_DESCRIPTION),
            absolute_path: PathBuf::from(DEFAULT_PATH),
            file_path: PathBuf::from(DEFAULT_FILE_PATH),
        };

        let sup_entry: RefCell<Weak<RefCell<WisphaFatEntry>>> = RefCell::new(Weak::new());

        let sub_entries: RefCell<Vec<Rc<RefCell<WisphaFatEntry>>>> = RefCell::new(Vec::new());

        WisphaEntry {
            properties,
            sup_entry,
            sub_entries
        }
    }
}

impl Clone for WisphaEntry {
    fn clone(&self) -> Self {
        WisphaEntry {
            properties: self.properties.clone(),
            sup_entry: self.sup_entry.clone(),
            sub_entries: self.sub_entries.clone(),
        }
    }
}