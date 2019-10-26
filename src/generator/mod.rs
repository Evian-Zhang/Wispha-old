use std::fs::{self, DirEntry};
use std::path::PathBuf;
use std::rc::Rc;
use std::cell::RefCell;

use crate::wispha::{WisphaEntry, WisphaEntryType, WisphaFatEntry, WisphaIntermediateEntry};
use crate::strings::*;

use ignore::{gitignore::{GitignoreBuilder, Gitignore}};

pub mod error;

use error::GeneratorError;

mod converter;

pub mod option;

use option::*;
use std::sync::{Mutex, Arc, mpsc};
use std::thread;
use std::sync::mpsc::Sender;

pub type Result<T> = std::result::Result<T, GeneratorError>;

// treat `path` as root. `path` is absolute
pub fn generate(path: &PathBuf, options: GeneratorOptions) -> Result<()> {
    let root = match &options.layer {
        GenerateLayer::Flat => {
            generate_file_at_path_flat(&path, &path, &get_ignored_files_from_root(path, &options.ignored_files)?, &options)?
        }
        GenerateLayer::Recursive => {
            generate_file_at_path_recursively(&path, &path, &get_ignored_files_from_root(path, &options.ignored_files)?, &options)?
        }
    };
    let root_path = path.join(PathBuf::from(&options.wispha_name));
    fs::write(&root_path, &root.to_file_string(0, &path)?)
        .or(Err(GeneratorError::FileCannotWrite(root_path.clone())))?;
    Ok(())
}

// read ignored patterns from GeneratorOptions to form a Gitignore instance
fn get_ignored_files_from_root(root_dir: &PathBuf, ignored_files: &Vec<String>) -> Result<Gitignore> {
    let mut ignore_builder = GitignoreBuilder::new(root_dir);
    for ignored_file in ignored_files {
        ignore_builder.add_line(None, ignored_file).or_else(|error| Err(GeneratorError::IgnoreError(error)))?;
    }
    let wispha_ignore = ignore_builder.build().or_else(|error| Err(GeneratorError::IgnoreError(error)))?;
    Ok(wispha_ignore)
}

// `path` is absolute
fn generate_file_at_path_without_sub_and_sup(path: &PathBuf, options: &GeneratorOptions) -> Result<WisphaEntry> {
    let mut wispha_entry = WisphaEntry::default();

    wispha_entry.properties.name = path.file_name().ok_or(GeneratorError::NameNotDetermined(path.clone()))?
        .to_str().ok_or(GeneratorError::NameNotValid(path.clone()))?
        .to_string();

    wispha_entry.properties.absolute_path = path.clone();

    wispha_entry.properties.entry_type = match path.is_dir() {
        true => WisphaEntryType::Directory,
        false => WisphaEntryType::File,
    };

    let properties = &options.properties;
    for property in properties {
        if let Some(default_value) = &property.default_value {
            wispha_entry.properties.customized.insert(property.name.clone(), default_value.clone());
        }
    }

    Ok(wispha_entry)
}

fn should_include_entry(entry: &DirEntry, wispha_ignore: &Gitignore, options: &GeneratorOptions) -> bool {
    if wispha_ignore.matched(&entry.path(), entry.path().is_dir()).is_ignore() {
        return false;
    }

    if entry.file_name().to_str().map(|s| s.starts_with(".")).unwrap_or(false) {
        return options.allow_hidden_files;
    }

    true
}

// `path` and `root_dir` are absolute. Returned `WisphaEntry` has no `sup_entry`. Generated intermediate entry's path is relative. Write all sub_entry to disk
// if `path` is not allowed, all its entries will not be considered
fn generate_file_at_path_recursively(path: &PathBuf, root_dir: &PathBuf, ignored_files: &Gitignore, options: &GeneratorOptions) -> Result<WisphaEntry> {
    let wispha_entry = generate_file_at_path_without_sub_and_sup(path, &options)?;
    if path.is_dir() {
        for entry in fs::read_dir(&path).or(Err(GeneratorError::DirCannotRead(path.clone())))? {
            let entry = entry.or(Err(GeneratorError::Unexpected))?;
            if should_include_entry(&entry, ignored_files, options) {
                let sub_entry = generate_file_at_path_recursively(&entry.path(), root_dir, ignored_files, &options)?;
                if (&entry.path()).is_dir() {
                    let absolute_path = sub_entry.properties.absolute_path
                        .join(PathBuf::from(&options.wispha_name));
                    fs::write(&absolute_path, &sub_entry.to_file_string(0, root_dir)?)
                        .or(Err(GeneratorError::FileCannotWrite(absolute_path.clone())))?;

                    let relative_path = PathBuf::from(&sub_entry.properties.name)
                        .join(PathBuf::from(&options.wispha_name));

                    let intermediate_entry = WisphaIntermediateEntry {
                        entry_file_path: relative_path,
                    };

                    wispha_entry.sub_entries.borrow_mut()
                        .push(Rc::new(RefCell::new(WisphaFatEntry::Intermediate(intermediate_entry))));
                } else {
                    wispha_entry.sub_entries.borrow_mut()
                        .push(Rc::new(RefCell::new(WisphaFatEntry::Immediate(sub_entry))));
                }
            }
        }
    }
    Ok(wispha_entry)
}

fn generate_entry_from_path_concurrently(path: &PathBuf, root_dir: &PathBuf, ignored_files: &Gitignore, options: &GeneratorOptions, sup_entry_things: Option<(Arc<Mutex<WisphaEntry>>, mpsc::Sender<bool>, mpsc::Sender<bool>)>) -> Result<()> {
//    let wispha_entry = generate_file_at_path_without_sub_and_sup(path, &options)?;
//    if let Some(sup_entry) = sup_entry {
//        let locked_sup_entry = sup_entry.lock().unwrap();
//        if path.is_dir() {
//            let relative_path = PathBuf::from(path.file_name().unwrap())
//                .join(PathBuf::from(&options.wispha_name));
//
//            let intermediate_entry = WisphaIntermediateEntry {
//                entry_file_path: relative_path,
//            };
//            *locked_sup_entry.sub_entries.borrow_mut().push(Rc::new(RefCell::new(WisphaFatEntry::Intermediate(intermediate_entry))));
//        } else {
//            *locked_sup_entry.sub_entries.borrow_mut().push(Rc::new(RefCell::new(WisphaFatEntry::Immediate(wispha_entry))));
//        }
//        drop(locked_sup_entry);
//    }
    if path.is_dir() {
        let relative_path = PathBuf::from(path.file_name().unwrap())
            .join(PathBuf::from(&options.wispha_name));

        let intermediate_entry = WisphaIntermediateEntry {
            entry_file_path: relative_path,
        };
        let (tx_global, rx_global_option) = if let Some((sup_entry, tx_local, tx_global)) = sup_entry_things {
            let locked_sup_entry = sup_entry.lock().unwrap();
            locked_sup_entry.sub_entries.borrow_mut().push(Rc::new(RefCell::new(WisphaFatEntry::Intermediate(intermediate_entry))));
            drop(locked_sup_entry);
            tx_local.send(true).or(Err(GeneratorError::Unexpected))?;
            drop(tx_local);
            (tx_global, None)
        } else {
            let (tx_global, rx_global) =mpsc::channel();
            (tx_global, Some(rx_global))
        };
        let wispha_entry = Arc::new(Mutex::new(generate_file_at_path_without_sub_and_sup(path, &options)?));
        let (tx, rx) = mpsc::channel();
        let entries: Vec<std::io::Result<DirEntry>> = fs::read_dir(&path).or(Err(GeneratorError::DirCannotRead(path.clone())))?.collect();
        let entries_count = entries.len();
        for entry in entries {
            let cloned_wispha = Arc::clone(&wispha_entry);
            thread::spawn(move || -> Result<()> {
                let entry = entry.or(Err(GeneratorError::Unexpected))?;
                if should_include_entry(&entry, ignored_files, options) {
                        generate_entry_from_path_concurrently(path, root_dir, ignored_files, options, Some((cloned_wispha, Sender::clone(&tx), Sender::clone(&tx_global))))?;
                }
                Ok(())
            });
        }
        drop(tx);
        for _ in rx { }
        let entry = wispha_entry.into_inner().unwrap();
        let absolute_path = path.join(PathBuf::from(&options.wispha_name));
        fs::write(&absolute_path, entry.to_file_string(0, root_dir)?)
            .or(Err(GeneratorError::FileCannotWrite(absolute_path.clone())))?;
        tx_global.send(true).or(Err(GeneratorError::Unexpected))?;
        drop(tx_global);
        if let Some(rx_global) = rx_global_option {
            for _ in rx_global { }
            return Ok(())
        }
    } else {
        let wispha_entry = generate_file_at_path_without_sub_and_sup(path, &options)?;
        if let Some((sup_entry, tx_local, tx_global)) = sup_entry_things {
            let locked_sup_entry = sup_entry.lock().unwrap();
            locked_sup_entry.sub_entries.borrow_mut().push(Rc::new(RefCell::new(WisphaFatEntry::Immediate(wispha_entry))));
            drop(locked_sup_entry);
            tx_local.send(true).or(Err(GeneratorError::Unexpected))?;
            drop(tx_local);
            tx_global.send(true).or(Err(GeneratorError::Unexpected))?;
            drop(tx_global);
        }
    }

    Ok(())
}

// `path` and `root_dir` are absolute. Returned `WisphaEntry` has no `sup_entry`. Not write sub_entry to disk
fn generate_file_at_path_flat(path: &PathBuf, root_dir: &PathBuf, ignored_files: &Gitignore, options: &GeneratorOptions) -> Result<WisphaEntry> {
    let wispha_entry = generate_file_at_path_without_sub_and_sup(path, &options)?;
    if path.is_dir() {
        for entry in fs::read_dir(&path).or(Err(GeneratorError::DirCannotRead(path.clone())))? {
            let entry = entry.or(Err(GeneratorError::Unexpected))?;
            if should_include_entry(&entry, ignored_files, options) {
                let sub_entry = generate_file_at_path_flat(&entry.path(), root_dir, ignored_files, &options)?;
                wispha_entry.sub_entries.borrow_mut()
                    .push(Rc::new(RefCell::new(WisphaFatEntry::Immediate(sub_entry))));
            }
        }
    }
    Ok(wispha_entry)
}
