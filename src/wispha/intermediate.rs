use std::path::PathBuf;
use std::sync::{Mutex, Weak, Arc};
use std::collections::HashMap;

use crate::wispha::{core::*, common::*};
use crate::strings::*;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Clone)]
pub enum WisphaIntermediateEntry {
    Link(WisphaLinkEntry),
    Direct(WisphaDirectEntry),
}

#[derive(Clone)]
pub struct WisphaLinkEntry {
    pub entry_file_path: PathBuf,
}

pub struct WisphaDirectEntry {
    pub properties:  WisphaEntryProperties,
    pub sup_entry: Mutex<Weak<Mutex<WisphaIntermediateEntry>>>,
    pub sub_entries: Mutex<Vec<Arc<Mutex<WisphaIntermediateEntry>>>>,
}

impl WisphaLinkEntry {
    pub fn default() -> WisphaLinkEntry {
        WisphaLinkEntry {
            entry_file_path: PathBuf::new(),
        }
    }
}

impl WisphaDirectEntry {
    pub fn default() -> WisphaDirectEntry {
        let properties = WisphaEntryProperties {
            entry_type: DEFAULT_ENTRY_TYPE,
            name: String::from(DEFAULT_NAME),
            description: None,
            absolute_path: PathBuf::from(DEFAULT_PATH),
            file_path: PathBuf::from(DEFAULT_FILE_PATH),
            customized: HashMap::new(),
        };

        let sup_entry: Mutex<Weak<Mutex<WisphaIntermediateEntry>>> = Mutex::new(Weak::new());

        let sub_entries: Mutex<Vec<Arc<Mutex<WisphaIntermediateEntry>>>> = Mutex::new(Vec::new());

        WisphaDirectEntry {
            properties,
            sup_entry,
            sub_entries
        }
    }
}

impl WisphaIntermediateEntry {
    pub fn get_type(&self) -> WisphaIntermediateEntry {
        use WisphaIntermediateEntry::*;
        match &self {
            Direct(_) => {
                Direct(WisphaDirectEntry::default())
            },
            Link(_) => {
                Link(WisphaLinkEntry::default())
            }
        }
    }
    pub fn get_direct_entry(&self) -> Option<&WisphaDirectEntry> {
        if let WisphaIntermediateEntry::Direct(entry) = &self {
            return Some(entry);
        }
        None
    }

    pub fn get_direct_entry_mut(&mut self) -> Option<&mut WisphaDirectEntry> {
        if let WisphaIntermediateEntry::Direct(entry) = self {
            return Some(entry);
        }
        None
    }

    pub fn get_link_entry(&self) -> Option<&WisphaLinkEntry> {
        if let WisphaIntermediateEntry::Link(entry) = &self {
            return Some(entry);
        }
        None
    }

    pub fn get_link_entry_mut(&mut self) -> Option<&mut WisphaLinkEntry> {
        if let WisphaIntermediateEntry::Link(entry) = self {
            return Some(entry);
        }
        None
    }

    // must be used from the top, i.e., the `sup_entry` is RefCell::new(Weak::new())
    pub fn to_common(&self) -> Option<Rc<RefCell<WisphaEntry>>> {
        use WisphaIntermediateEntry::*;
        match &self {
            Direct(direct_entry) => {
                let mut common = Rc::new(RefCell::new(WisphaEntry::default()));
                common.borrow_mut().properties = direct_entry.properties.clone();
                let locked_sub_entries = direct_entry.sub_entries.lock().unwrap();
                for sub_entry in &*locked_sub_entries {
                    let locked_sub_entry = sub_entry.lock().unwrap();
                    if let Some(sub_entry) = locked_sub_entry.to_common() {
                        sub_entry.borrow_mut().sup_entry = RefCell::new(Rc::downgrade(&common));
                        common.borrow_mut().sub_entries.borrow_mut().push(Rc::clone(&sub_entry));
                    } else {
                        return None;
                    }
                    drop(locked_sub_entry);
                }
                drop(locked_sub_entries);
                Some(common)
            },
            Link(_) => {
                None
            }
        }
    }
}

impl Clone for WisphaDirectEntry {
    fn clone(&self) -> Self {
        let locked_sup_entry = self.sup_entry.lock().unwrap();
        let sup_entry = if let Some(sup_entry) = locked_sup_entry.upgrade() {
            Mutex::new(Arc::downgrade(&sup_entry))
        } else {
            Mutex::new(Weak::new())
        };
        drop(locked_sup_entry);
        let locked_sub_entries = self.sub_entries.lock().unwrap();
        let sub_entries = Mutex::new(locked_sub_entries.iter().map(|sub_entry| {Arc::clone(sub_entry)}).collect());
        drop(locked_sub_entries);
        WisphaDirectEntry {
            properties: self.properties.clone(),
            sup_entry,
            sub_entries,
        }
    }
}