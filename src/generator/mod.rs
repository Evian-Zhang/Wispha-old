use std::fs::{self, DirEntry};
use std::path::PathBuf;
use std::sync::{Mutex, Arc, mpsc};
use std::thread;
use std::sync::mpsc::Sender;

use crate::strings::*;
use crate::wispha::{intermediate::*, core::*};
use crate::helper::thread_pool::ThreadPool;

use ignore::{gitignore::{GitignoreBuilder, Gitignore}};

pub mod error;
use error::GeneratorError;

mod converter;

pub mod option;
use option::*;

pub type Result<T> = std::result::Result<T, GeneratorError>;

// treat `path` as root. `path` is absolute
pub fn generate(path: PathBuf, options: GeneratorOptions) -> Result<()> {
    let thread_pool = Arc::new(Mutex::new(ThreadPool::new(options.threads)));
    match &options.layer {
        GenerateLayer::Flat => {
            let ignored_files = get_ignored_files_from_root(path.clone(), options.ignored_files.clone())?;
            generate_entry_from_path_flat_and_concurrently(Arc::new(path.clone()), Arc::new(path.clone()), Arc::new(ignored_files), Arc::new(options), None, thread_pool)?;
        }
        GenerateLayer::Recursive => {
            let ignored_files = get_ignored_files_from_root(path.clone(), options.ignored_files.clone())?;
            generate_entry_from_path_recursively_and_concurrently(Arc::new(path.clone()), Arc::new(path.clone()), Arc::new(ignored_files), Arc::new(options), None, thread_pool)?;
        }
    }
    Ok(())
}

// read ignored patterns from GeneratorOptions to form a Gitignore instance
fn get_ignored_files_from_root(root_dir: PathBuf, ignored_files: Vec<String>) -> Result<Gitignore> {
    let mut ignore_builder = GitignoreBuilder::new(root_dir);
    for ignored_file in ignored_files {
        ignore_builder.add_line(None, &ignored_file).or_else(|error| Err(GeneratorError::IgnoreError(error)))?;
    }
    let wispha_ignore = ignore_builder.build().or_else(|error| Err(GeneratorError::IgnoreError(error)))?;
    Ok(wispha_ignore)
}

// `path` is absolute
fn generate_file_at_path_without_sub_and_sup(path: Arc<PathBuf>, options: Arc<GeneratorOptions>) -> Result<WisphaDirectEntry> {
    let mut wispha_entry = WisphaDirectEntry::default();

    wispha_entry.properties.name = path.file_name().ok_or(GeneratorError::NameNotDetermined((*path).clone()))?
        .to_str().ok_or(GeneratorError::NameNotValid((*path).clone()))?
        .to_string();

    wispha_entry.properties.absolute_path = (*path).clone();

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

fn should_include_entry(entry: &DirEntry, wispha_ignore: Arc<Gitignore>, options: Arc<GeneratorOptions>) -> bool {
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
fn generate_entry_from_path_recursively_and_concurrently(path: Arc<PathBuf>, root_dir: Arc<PathBuf>, ignored_files: Arc<Gitignore>, options: Arc<GeneratorOptions>, sup_entry_things: Option<(Arc<Mutex<WisphaDirectEntry>>, mpsc::Sender<bool>, mpsc::Sender<bool>)>, thread_pool: Arc<Mutex<ThreadPool>>) -> Result<()> {
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
        let direct_entry = Arc::new(Mutex::new(generate_file_at_path_without_sub_and_sup(Arc::clone(&path), Arc::clone(&options))?));
        let (tx, rx) = mpsc::channel();
        let entries: Vec<std::io::Result<DirEntry>> = fs::read_dir(&*path).or(Err(GeneratorError::DirCannotRead((*path).clone())))?.collect();
        for entry in entries {
            let entry = entry.or(Err(GeneratorError::Unexpected))?;
            let cloned_wispha = Arc::clone(&direct_entry);
            let cloned_ignored_files = Arc::clone(&ignored_files);
            let cloned_options = Arc::clone(&options);
            let cloned_path = Arc::new(entry.path().clone());
            let cloned_root_dir = Arc::clone(&root_dir);
            let cloned_tx = Sender::clone(&tx);
            let cloned_tx_global = Sender::clone(&tx_global);
            let cloned_thread_pool = Arc::clone(&thread_pool);
            if entry.path().is_dir() {
                thread_pool.lock().unwrap().execute(move || {
                    if should_include_entry(&entry, Arc::clone(&cloned_ignored_files), Arc::clone(&cloned_options)) {
                        generate_entry_from_path_recursively_and_concurrently(cloned_path, cloned_root_dir, cloned_ignored_files, cloned_options, Some((cloned_wispha, cloned_tx, cloned_tx_global)), cloned_thread_pool);
                    }
                });
            } else {
                if should_include_entry(&entry, Arc::clone(&cloned_ignored_files), Arc::clone(&cloned_options)) {
                    generate_entry_from_path_recursively_and_concurrently(cloned_path, cloned_root_dir, cloned_ignored_files, cloned_options, Some((cloned_wispha, cloned_tx, cloned_tx_global)), cloned_thread_pool)?;
                }
            }
        }
        drop(tx);
        for _ in rx { }
        let locked_entry = direct_entry.lock().unwrap();
        let absolute_path = path.join(PathBuf::from(&options.wispha_name));
        fs::write(&absolute_path, locked_entry.to_file_string(0, &root_dir)?)
            .or(Err(GeneratorError::FileCannotWrite(absolute_path.clone())))?;
        drop(locked_entry);
        tx_global.send(true).or(Err(GeneratorError::Unexpected))?;
        drop(tx_global);
        if let Some(rx_global) = rx_global_option {
            for _ in rx_global { }
            return Ok(())
        }
    } else {
        let direct_entry = generate_file_at_path_without_sub_and_sup(Arc::clone(&path), Arc::clone(&options))?;
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
fn generate_entry_from_path_flat_and_concurrently(path: Arc<PathBuf>, root_dir: Arc<PathBuf>, ignored_files: Arc<Gitignore>, options: Arc<GeneratorOptions>, sup_entry_things: Option<(Arc<Mutex<WisphaIntermediateEntry>>, mpsc::Sender<bool>, mpsc::Sender<bool>)>, thread_pool: Arc<Mutex<ThreadPool>>) -> Result<()> {
    let intermediate_entry = Arc::new(Mutex::new(WisphaIntermediateEntry::Direct(generate_file_at_path_without_sub_and_sup(Arc::clone(&path), Arc::clone(&options))?)));
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
        for entry in fs::read_dir(&*path).or(Err(GeneratorError::DirCannotRead((*path).clone())))? {
            let cloned_wispha = Arc::clone(&intermediate_entry);
            let cloned_ignored_files = Arc::clone(&ignored_files);
            let cloned_options = Arc::clone(&options);
            let cloned_path = Arc::new(entry.path().clone());
            let cloned_root_dir = Arc::clone(&root_dir);
            let cloned_tx = Sender::clone(&tx);
            let cloned_tx_global = Sender::clone(&tx_global);
            let cloned_thread_pool = Arc::clone(&thread_pool);
            thread_pool.lock().unwrap().execute(move || {
                let entry = entry.unwrap();
                if should_include_entry(&entry, Arc::clone(&cloned_ignored_files), Arc::clone(&cloned_options)) {
                    generate_entry_from_path_flat_and_concurrently(cloned_path, cloned_root_dir, cloned_ignored_files, cloned_options, Some((cloned_wispha, cloned_tx, cloned_tx_global)), cloned_thread_pool);
                }
            });
        }
        drop(tx);
        for _ in rx { }
        tx_global.send(true).or(Err(GeneratorError::Unexpected))?;
        drop(tx_global);
        if let Some(rx_global) = rx_global_option {
            for _ in rx_global { }
            let locked_intermediate_entry = intermediate_entry.lock().unwrap();
            let entry = locked_intermediate_entry.get_direct_entry().unwrap();
            let absolute_path = path.join(PathBuf::from(&options.wispha_name));
            fs::write(&absolute_path, entry.to_file_string(0, &root_dir)?)
                .or(Err(GeneratorError::FileCannotWrite(absolute_path.clone())))?;
            drop(locked_intermediate_entry);
            return Ok(())
        }
    } else {
        tx_global.send(true).or(Err(GeneratorError::Unexpected))?;
        drop(tx_global);
    }
    Ok(())
}
