use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::path::{PathBuf, Component};
use std::env;

use crate::wispha::common::*;
use crate::strings::*;

pub mod error;
use error::ManipulatorError;

type Result<T> = std::result::Result<T, ManipulatorError>;

pub struct Manipulator {
    pub root: Rc<RefCell<WisphaFatEntry>>,
    pub current_entry: Rc<RefCell<WisphaEntry>>,
    pub entries: HashMap<PathBuf, Rc<RefCell<WisphaEntry>>>,
}

impl Manipulator {
    pub fn new(root: &Rc<RefCell<WisphaEntry>>, current_entry: &Rc<RefCell<WisphaEntry>>) -> Manipulator {
        let root = Rc::clone(root);
        let current_entry = Rc::clone(current_entry);
        let mut entries: HashMap<PathBuf, Rc<RefCell<WisphaEntry>>> = HashMap::new();
        push_into_entries(&root, &mut entries);
        Manipulator { root, current_entry, entries }
    }

    pub fn set_current_entry_to_local_path(&mut self, path: &PathBuf) -> Result<()> {
        let current_path = (*self.current_entry).borrow()
            .properties
            .absolute_path.clone();
        if let Some(target_entry) = self.entries.get(&actual_path(&path, &current_path)?) {
            self.current_entry = Rc::clone(target_entry);
            return Ok(());
        } else {
            return Err(ManipulatorError::PathNotEntry(path.clone()));
        }
    }

    pub fn set_current_entry_to_path(&mut self, path: &PathBuf) -> Result<()> {
        if path.is_absolute() {
            return Err(ManipulatorError::AbsolutePathNotSupported);
        }

        self.current_entry = if path.starts_with(ROOT_DIR) {
            let remain_path = path.strip_prefix(ROOT_DIR).unwrap().to_path_buf();
            let used_path = PathBuf::from(ROOT_DIR);
            find_entry(Rc::clone(&self.root), remain_path, used_path)?
        } else {
            find_entry(Rc::clone(&self.current_entry), path.clone(), PathBuf::new())?
        };
        Ok(())
    }

    pub fn current_path(&self) -> PathBuf {
        let raw = (*self.current_entry)
            .borrow()
            .properties
            .absolute_path.clone();
        let root_dir = PathBuf::from(env::var(ROOT_DIR_VAR).unwrap());
        if raw.starts_with(&root_dir) {
            PathBuf::from(ROOT_DIR).join(raw.strip_prefix(root_dir).unwrap().to_path_buf())
        } else {
            PathBuf::from(ROOT_DIR).join(raw)
        }
    }

    pub fn current_list(&self) -> String {
        let mut names: Vec<String> = Vec::new();
        for sub_entry in &*(*self.current_entry).borrow().sub_entries.borrow() {
            let sub_entry = Rc::clone(sub_entry);
            let sub_entry = (*sub_entry).borrow();
            names.push(sub_entry.properties.name.clone());
        }

        names.join("\n")
    }

    pub fn list_of_local_path(&self, path: &PathBuf) -> Result<String> {
        let current_path = (*self.current_entry)
            .borrow()
            .properties
            .absolute_path.clone();
        let actual_path = actual_path(path, &current_path)?;
        match self.entries.get(&actual_path) {
            Some(entry) => {
                let mut names: Vec<String> = Vec::new();
                let entry = Rc::clone(entry);
                for sub_entry in &*(*entry).borrow().sub_entries.borrow() {
                    let sub_entry = Rc::clone(sub_entry);
                    let sub_entry = (*sub_entry).borrow();
                    names.push(sub_entry.properties.name.clone());
                }

                Ok(names.join("\n"))
            },

            None => {
                Err(ManipulatorError::PathNotEntry(path.clone()))
            }
        }
    }

    pub fn list_of_path(&self, path: &PathBuf) -> Result<String> {
        let entry = if path.starts_with(ROOT_DIR) {
            let remain_path = path.strip_prefix(ROOT_DIR).unwrap().to_path_buf();
            let used_path = PathBuf::from(ROOT_DIR);
            find_entry(Rc::clone(&self.root), remain_path, used_path)?
        } else {
            find_entry(Rc::clone(&self.current_entry), path.clone(), PathBuf::new())?
        };
        let mut names: Vec<String> = Vec::new();
        for sub_entry in &*(*entry).borrow().sub_entries.borrow() {
            let sub_entry = Rc::clone(sub_entry);
            let sub_entry = (*sub_entry).borrow();
            names.push(sub_entry.properties.name.clone());
        }

        Ok(names.join("\n"))
    }

    pub fn info_of_property(&self, property: &String) -> Result<String> {
        match property.as_str() {
            ABSOLUTE_PATH_HEADER => {
                return Ok(self.current_entry.borrow().properties.absolute_path.to_str().unwrap().to_string());
            },
            NAME_HEADER => {
                return Ok(self.current_entry.borrow().properties.name.clone());

            },
            ENTRY_TYPE_HEADER => {
                return Ok(self.current_entry.borrow().properties.entry_type.to_str().to_string());

            },
            DESCRIPTION_HEADER => {
                if let Some(description) = &self.current_entry.borrow().properties.description {
                    return Ok(description.clone());
                }
            },
            _ => {
                if let Some(value) = self.current_entry.borrow().properties.customized.get(property) {
                    return Ok(value.clone());
                }
            }
        }
        Err(ManipulatorError::PropertyNotFound)
    }
}

fn push_into_entries(entry: &Rc<RefCell<WisphaEntry>>, entries: &mut HashMap<PathBuf, Rc<RefCell<WisphaEntry>>>) {
    let entry = Rc::clone(entry);
    entries.insert((*entry).borrow().properties.absolute_path.clone(), Rc::clone(&entry));
    for sub_entry in &*(*entry).borrow().sub_entries.borrow() {
        push_into_entries(sub_entry, entries);
    }
}

fn actual_path(raw: &PathBuf, current_dir: &PathBuf) -> Result<PathBuf> {
    if raw.is_absolute() {
        return Ok(raw.clone());
    }

    if raw.starts_with(ROOT_DIR) {
        let root_dir = PathBuf::from(env::var(ROOT_DIR_VAR).unwrap());
        let relative_path = raw.strip_prefix(ROOT_DIR).unwrap().to_path_buf();
        return Ok(root_dir.join(relative_path));
    }

    Ok(current_dir.join(&raw).canonicalize().or(Err(ManipulatorError::PathNotExist))?)
}

fn find_entry(current_entry: Rc<RefCell<WisphaEntry>>, remain_path: PathBuf, used_path: PathBuf) -> Result<Rc<RefCell<WisphaEntry>>> {
    let path = remain_path.clone();
    if let Some(component) = path.components().next() {
        match component {
            Component::CurDir => {
                let remain_path = remain_path.strip_prefix(".").unwrap().to_path_buf();
                let mut used_path = used_path.clone();
                used_path.push(".");
                return find_entry(current_entry, remain_path, used_path);
            }
            Component::ParentDir => {
                if let Some(next_entry) = current_entry.borrow()
                    .sup_entry.borrow()
                    .upgrade() {
                    let remain_path = remain_path.strip_prefix("..").unwrap().to_path_buf();
                    let mut used_path = used_path.clone();
                    used_path.push("..");
                    return find_entry(next_entry, remain_path, used_path);
                } else {
                    return Err(ManipulatorError::BeyondDomain);
                }
            },
            Component::Normal(name) => {
                let name = name.to_str().unwrap();
                for sub_entry in &*current_entry.borrow()
                    .sub_entries.borrow() {
                    if sub_entry.borrow().properties.name == name.to_string() {
                        let remain_path = remain_path.strip_prefix(name).unwrap().to_path_buf();
                        let mut used_path = used_path.clone();
                        used_path.push(name);
                        return find_entry(Rc::clone(sub_entry), remain_path, used_path);
                    }
                }
                let mut used_path = used_path.clone();
                used_path.push(name);
                return Err(ManipulatorError::EntryNotFound(used_path));
            },
            _ => {
                return Err(ManipulatorError::Unexpected);
            }
        }
    } else {
        return Ok(current_entry);
    }
}
