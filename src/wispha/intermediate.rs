use std::path::PathBuf;
use std::sync::{Mutex, Weak, Arc};
use std::collections::HashMap;

use crate::wispha::core::*;
use crate::strings::*;

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