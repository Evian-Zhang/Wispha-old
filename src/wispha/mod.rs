use std::string::String;
use std::path::{Path, PathBuf};
use std::rc::{Rc, Weak};
use std::cell::RefCell;

mod error;
use error::WisphaError;
use crate::wispha::WisphaEntryType::Directory;

type Result<T> = std::result::Result<T, WisphaError>;

static DEFAULT_ENTRY_TYPE: WisphaEntryType = WisphaEntryType::File;
static DEFAULT_NAME: &str = "default name";
static DEFAULT_DESCRIPTION: &str = "default description";
static DEFAULT_PATH: &str = "default path";

static DEFAULT_FILE_NAME_STR: &str = "LOOKME.wispha";

pub static LINE_SEPARATOR: &str = "\n";

pub static BEGIN_MARK: &str = "+";

pub static ABSOLUTE_PATH_HEADER: &str = "absolute path";
pub static NAME_HEADER: &str = "name";
pub static ENTRY_TYPE_HEADER: &str = "entry type";
pub static DESCRIPTION_HEADER: &str = "description";

pub static ENTRY_FILE_PATH_HEADER: &str = "entry file path";
pub static SUB_ENTRIES_HEADER: &str = "subentry";

pub static ROOT_DIR: &str = "$ROOT_DIR";

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

impl WisphaEntryType {
    pub fn to_str(&self) -> &'static str {
        match &self {
            WisphaEntryType::Directory => "Directory",
            WisphaEntryType::File => "File",
            WisphaEntryType::ProgramEntry => "Program entry",
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
        };

        let entry_file_path = None;

        let sup_entry: RefCell<Weak<WisphaEntry>> = RefCell::new(Weak::new());

        let sub_entries: RefCell<Vec<Rc<WisphaEntry>>> = RefCell::new(Vec::new());

        WisphaEntry {
            properties,
            entry_file_path,
            sup_entry,
            sub_entries
        }
    }
}

impl Clone for WisphaEntry {
    fn clone(&self) -> Self {
        WisphaEntry {
            properties: self.properties.clone(),
            entry_file_path: self.entry_file_path.clone(),
            sup_entry: self.sup_entry.clone(),
            sub_entries: self.sub_entries.clone(),
        }
    }
}