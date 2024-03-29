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
    pub dependency_path_bufs: Mutex<Vec<PathBuf>>,
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

        let sup_entry = Mutex::new(Weak::new());

        let sub_entries = Mutex::new(Vec::new());

        let dependency_path_bufs = Mutex::new(Vec::new());

        WisphaDirectEntry {
            properties,
            sup_entry,
            sub_entries,
            dependency_path_bufs,
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
    pub fn to_common<F>(&self, callback: &mut F) -> Option<Rc<RefCell<WisphaEntry>>>
        where   F: FnMut(Rc<RefCell<WisphaEntry>>)
    {
        use WisphaIntermediateEntry::*;
        match &self {
            Direct(direct_entry) => {
                let common = Rc::new(RefCell::new(WisphaEntry::default()));
                callback(Rc::clone(&common));
                common.borrow_mut().properties = direct_entry.properties.clone();
                let locked_sub_entries = direct_entry.sub_entries.lock().unwrap();
                for sub_entry in &*locked_sub_entries {
                    let locked_sub_entry = sub_entry.lock().unwrap();
                    if let Some(sub_entry) = locked_sub_entry.to_common(callback) {
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

// take victim's status, except for its sup_entry
pub fn take_properties(entry: Arc<Mutex<WisphaIntermediateEntry>>, victim: Arc<Mutex<WisphaIntermediateEntry>>) {
    let mut locked_entry = entry.lock().unwrap();
    if let Some(direct_entry) = locked_entry.get_direct_entry_mut() {
        let mut locked_victim = victim.lock().unwrap();
        if let Some(direct_victim) = locked_victim.get_direct_entry_mut() {
            direct_entry.properties = direct_victim.properties.clone();
//            let mut locked_entry_sub_entries = direct_entry.sub_entries.lock().unwrap();
//            let locked_victim_sub_entries = direct_victim.sub_entries.lock().unwrap();
//            for sub_entry in &*locked_victim_sub_entries {
//                let mut locked_sub_entry = sub_entry.lock().unwrap();
//                if let Some(direct_sub_entry) = locked_sub_entry.get_direct_entry_mut() {
//                    direct_sub_entry.sup_entry = Mutex::new(Arc::downgrade(&entry));
//                }
//                drop(locked_sub_entry);
//                locked_entry_sub_entries.push(Arc::clone(sub_entry));
//            }
//            drop(locked_victim_sub_entries);
//            drop(locked_entry_sub_entries);
        }
        drop(locked_victim);
    }
    drop(locked_entry);
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
        let locked_dependency_path_bufs = self.dependency_path_bufs.lock().unwrap();
        let dependency_path_bufs = Mutex::new(locked_dependency_path_bufs.iter().map(|path| path.clone()).collect());
        drop(locked_dependency_path_bufs);
        WisphaDirectEntry {
            properties: self.properties.clone(),
            sup_entry,
            sub_entries,
            dependency_path_bufs,
        }
    }
}