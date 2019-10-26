use std::fs::{self, DirEntry};
use std::path::PathBuf;
use std::sync::{Mutex, Arc, mpsc};
use std::thread;
use std::sync::mpsc::Sender;

use crate::strings::*;
use crate::wispha::{intermediate::*, core::*};

use ignore::{gitignore::{GitignoreBuilder, Gitignore}};

pub mod error;
use error::GeneratorError;

mod converter;

pub mod option;
use option::*;

pub type Result<T> = std::result::Result<T, GeneratorError>;

// treat `path` as root. `path` is absolute
pub fn generate(path: &'static PathBuf, options: GeneratorOptions) -> Result<()> {
    match &options.layer {
        GenerateLayer::Flat => {
            let ignored_files = get_ignored_files_from_root(path, &options.ignored_files)?;
            generate_entry_from_path_flat_and_concurrently(&path, &path, &ignored_files, &options, None)?;
        }
        GenerateLayer::Recursive => {
            let ignored_files = get_ignored_files_from_root(path, &options.ignored_files)?;
            generate_entry_from_path_recursively_and_concurrently(&path, &path, &ignored_files, &options, None)?;
        }
    }
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
fn generate_file_at_path_without_sub_and_sup(path: &PathBuf, options: &GeneratorOptions) -> Result<WisphaDirectEntry> {
    let mut wispha_entry = WisphaDirectEntry::default();

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

// `sup_entry_things` is `None` if this is the top call from user, otherwise it is called from the function
// top call from user will not returned until the entire tree is constructed and all `.wispha` file is written
// `path` and `root_dir` is absolute
fn generate_entry_from_path_recursively_and_concurrently(path: &'static PathBuf, root_dir: &'static PathBuf, ignored_files: &'static Gitignore, options: &'static GeneratorOptions, sup_entry_things: Option<(Arc<Mutex<WisphaDirectEntry>>, mpsc::Sender<bool>, mpsc::Sender<bool>)>) -> Result<()> {
    if path.is_dir() {
        let relative_path = PathBuf::from(path.file_name().unwrap())
            .join(PathBuf::from(&options.wispha_name));

        let link_entry = WisphaLinkEntry {
            entry_file_path: relative_path,
        };
        let (tx_global, rx_global_option) = if let Some((sup_entry, tx_local, tx_global)) = sup_entry_things {
            let locked_sup_entry     = sup_entry.lock().unwrap();
            let mut locked_sub_entries = locked_sup_entry.sub_entries.lock().unwrap();
            locked_sub_entries.push(Arc::new(Mutex::new(WisphaIntermediateEntry::Link(link_entry))));
            drop(locked_sub_entries);
            drop(locked_sup_entry);
            tx_local.send(true).or(Err(GeneratorError::Unexpected))?;
            drop(tx_local);
            (tx_global, None)
        } else {
            let (tx_global, rx_global) =mpsc::channel();
            (tx_global, Some(rx_global))
        };
        let direct_entry = Arc::new(Mutex::new(generate_file_at_path_without_sub_and_sup(path, &options)?));
        let (tx, rx) = mpsc::channel();
        let entries: Vec<std::io::Result<DirEntry>> = fs::read_dir(&path).or(Err(GeneratorError::DirCannotRead(path.clone())))?.collect();
        let entries_count = entries.len();
        for entry in entries {
            let cloned_wispha = Arc::clone(&direct_entry);
            thread::spawn(move || -> Result<()> {
                let entry = entry.or(Err(GeneratorError::Unexpected))?;
                if should_include_entry(&entry, ignored_files, options) {
                    generate_entry_from_path_recursively_and_concurrently(path, root_dir, ignored_files, options, Some((cloned_wispha, Sender::clone(&tx), Sender::clone(&tx_global))))?;
                }
                Ok(())
            });
        }
        drop(tx);
        for _ in rx { }
        let entry = direct_entry.into_inner().unwrap();
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
        let direct_entry = generate_file_at_path_without_sub_and_sup(path, &options)?;
        if let Some((sup_entry, tx_local, tx_global)) = sup_entry_things {
            let locked_sup_entry = sup_entry.lock().unwrap();
            let mut locked_sub_entries = locked_sup_entry.sub_entries.lock().unwrap();
            locked_sub_entries.push(Arc::new(Mutex::new(WisphaIntermediateEntry::Direct(direct_entry))));
            drop(locked_sub_entries);
            drop(locked_sup_entry);
            tx_local.send(true).or(Err(GeneratorError::Unexpected))?;
            drop(tx_local);
            tx_global.send(true).or(Err(GeneratorError::Unexpected))?;
            drop(tx_global);
        }
    }

    Ok(())
}

// `sup_entry_things` is `None` if this is the top call from user, otherwise it is called from the function
// top call from user will not returned until the entire tree is constructed and `.wispha` file is written
// `path` and `root_dir` is absolute
fn generate_entry_from_path_flat_and_concurrently(path: &'static PathBuf, root_dir: &'static PathBuf, ignored_files: &'static Gitignore, options: &'static GeneratorOptions, sup_entry_things: Option<(Arc<Mutex<WisphaIntermediateEntry>>, mpsc::Sender<bool>, mpsc::Sender<bool>)>) -> Result<()> {
    let intermediate_entry = Arc::new(Mutex::new(WisphaIntermediateEntry::Direct(generate_file_at_path_without_sub_and_sup(path, &options)?)));
    let (tx_global, rx_global_option) = if let Some((sup_entry, tx_local, tx_global)) = sup_entry_things {
        let mut locked_sup_entry = sup_entry.lock().unwrap();
        let mut locked_sub_entries = locked_sup_entry.get_direct_entry_mut().unwrap().sub_entries.lock().unwrap();
        locked_sub_entries.push(Arc::clone(&intermediate_entry));
        drop(locked_sub_entries);
        drop(locked_sup_entry);
        tx_local.send(true).or(Err(GeneratorError::Unexpected))?;
        drop(tx_local);
        (tx_global, None)
    } else {
        let (tx_global, rx_global) =mpsc::channel();
        (tx_global, Some(rx_global))
    };
    if path.is_dir() {
        let (tx, rx) = mpsc::channel();
        for entry in fs::read_dir(&path).or(Err(GeneratorError::DirCannotRead(path.clone())))? {
            let cloned_wispha = Arc::clone(&intermediate_entry);
            thread::spawn(move || -> Result<()> {
                let entry = entry.or(Err(GeneratorError::Unexpected))?;
                if should_include_entry(&entry, ignored_files, options) {
                    generate_entry_from_path_flat_and_concurrently(path, root_dir, ignored_files, options, Some((cloned_wispha, Sender::clone(&tx), Sender::clone(&tx_global))));
                }
                Ok(())
            });
        }
        drop(tx);
        for _ in rx { }
        tx_global.send(true).or(Err(GeneratorError::Unexpected))?;
        drop(tx_global);
        if let Some(rx_global) = rx_global_option {
            for _ in rx_global { }
            let intermediate_entry = intermediate_entry.into_inner().unwrap();
            let entry = intermediate_entry.get_direct_entry().unwrap();
            let absolute_path = path.join(PathBuf::from(&options.wispha_name));
            fs::write(&absolute_path, entry.to_file_string(0, root_dir)?)
                .or(Err(GeneratorError::FileCannotWrite(absolute_path.clone())))?;
            return Ok(())
        }
    } else {
        tx_global.send(true).or(Err(GeneratorError::Unexpected))?;
        drop(tx_global);
    }
    Ok(())
}
