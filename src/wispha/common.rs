use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::wispha::core::*;
use crate::strings::*;

#[derive(Clone)]
pub struct WisphaEntry {
    pub properties:  WisphaEntryProperties,
    pub sup_entry: RefCell<Weak<RefCell<WisphaEntry>>>, // for the root node, `*sup_entry` is Weak::new()
    pub sub_entries: RefCell<Vec<Rc<RefCell<WisphaEntry>>>>,
    pub dependencies: RefCell<Vec<Weak<RefCell<WisphaEntry>>>>,
    pub dependency_path_bufs: RefCell<Vec<PathBuf>>,
}

impl WisphaEntry {
    pub fn default() -> WisphaEntry {
        let properties = WisphaEntryProperties {
            entry_type: DEFAULT_ENTRY_TYPE,
            name: String::from(DEFAULT_NAME),
            description: None,
            absolute_path: PathBuf::from(DEFAULT_PATH),
            file_path: PathBuf::from(DEFAULT_FILE_PATH),
            customized: HashMap::new(),
        };

        let sup_entry = RefCell::new(Weak::new());

        let sub_entries = RefCell::new(Vec::new());

        let dependencies = RefCell::new(Vec::new());

        let dependency_path_bufs = RefCell::new(Vec::new());

        WisphaEntry {
            properties,
            sup_entry,
            sub_entries,
            dependencies,
            dependency_path_bufs,
        }
    }
}