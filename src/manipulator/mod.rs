use std::cell::{RefCell, Ref};
use std::collections::HashMap;
use std::rc::{Weak, Rc};
use std::pin::Pin;
use std::path::PathBuf;
use std::env;

use crate::wispha::{self, WisphaEntryType, WisphaEntry, WisphaFatEntry, WisphaIntermediateEntry, WisphaEntryProperties};

pub mod error;
use error::ManipulatorError;
use std::borrow::Borrow;

type Result<T> = std::result::Result<T, ManipulatorError>;

pub struct Manipulator {
    pub root: Rc<RefCell<WisphaFatEntry>>,
    pub current_entry: Rc<RefCell<WisphaFatEntry>>,
    pub entries: HashMap<PathBuf, Rc<RefCell<WisphaFatEntry>>>,
}

impl Manipulator {
    pub fn new(root: &Rc<RefCell<WisphaFatEntry>>, current_entry: &Rc<RefCell<WisphaFatEntry>>) -> Manipulator {
        let root = Rc::clone(root);
        let current_entry = Rc::clone(current_entry);
        let mut entries: HashMap<PathBuf, Rc<RefCell<WisphaFatEntry>>> = HashMap::new();
        push_into_entries(&root, &mut entries);
        Manipulator { root, current_entry, entries }
    }

    pub fn set_current_entry_to_path(&mut self, path: &PathBuf) -> Result<()> {
        let current_path = (*self.current_entry).borrow()
            .get_immediate_entry().unwrap()
            .properties
            .absolute_path.clone();
        if let Some(target_entry) = self.entries.get(&actual_path(&path, &current_path)?) {
            self.current_entry = Rc::clone(target_entry);
            return Ok(());
        } else {
            return Err(ManipulatorError::PathNoEntry(path.clone()));
        }
    }

    pub fn goto_sup(&mut self) {
        let mut has_super_entry = true;
        let super_entry = (*self.current_entry).borrow()
            .get_immediate_entry().unwrap()
            .sup_entry.borrow()
            .upgrade().unwrap_or_else(|| {
            has_super_entry = false;
            Rc::new(RefCell::new(WisphaFatEntry::Immediate(WisphaEntry::default())))
        });
        if has_super_entry {
            let super_entry_file_path = (*super_entry).borrow()
                .get_immediate_entry().unwrap()
                .properties
                .file_path
                .clone();
            let super_a_entry = self.entries.get(&super_entry_file_path)
                .unwrap();
            self.current_entry = Rc::clone(super_a_entry);
        }
    }

    pub fn current_path(&self) -> PathBuf {
        let raw = (*self.current_entry)
            .borrow()
            .get_immediate_entry().unwrap()
            .properties
            .absolute_path.clone();
        let root_dir = PathBuf::from(env::var(wispha::ROOT_DIR_VAR).unwrap());
        PathBuf::from(wispha::ROOT_DIR).join(raw.strip_prefix(root_dir).unwrap().to_path_buf())
    }

    pub fn current_list(&self) -> String {
        let mut names: Vec<String> = Vec::new();
        for sub_entry in &*(*self.current_entry).borrow().get_immediate_entry().unwrap().sub_entries.borrow() {
            let sub_entry = Rc::clone(sub_entry);
            let sub_entry = (*sub_entry).borrow();
            let sub_entry = sub_entry.get_immediate_entry().unwrap();
            names.push(sub_entry.properties.name.clone());
        }

        names.join("\n")
    }
}

fn push_into_entries(entry: &Rc<RefCell<WisphaFatEntry>>, entries: &mut HashMap<PathBuf, Rc<RefCell<WisphaFatEntry>>>) {
    let entry = Rc::clone(entry);
    entries.insert((*entry).borrow().get_immediate_entry().unwrap().properties.absolute_path.clone(), Rc::clone(&entry));
    for sub_entry in &*(*entry).borrow().get_immediate_entry().unwrap().sub_entries.borrow() {
        push_into_entries(sub_entry, entries);
    }
}

fn actual_path(raw: &PathBuf, current_dir: &PathBuf) -> Result<PathBuf> {
    if raw.is_absolute() {
        return Ok(raw.clone());
    }

    if raw.starts_with(wispha::ROOT_DIR) {
        let root_dir = PathBuf::from(env::var(wispha::ROOT_DIR_VAR).unwrap());
        let relative_path = raw.strip_prefix(wispha::ROOT_DIR).unwrap().to_path_buf();
        return Ok(root_dir.join(relative_path));
    }

    Ok(current_dir.join(&raw).canonicalize().or(Err(ManipulatorError::PathNotExist))?)
}
