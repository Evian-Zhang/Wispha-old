use std::fs::{self, DirEntry};
use std::path::PathBuf;
use std::sync::{Mutex, Arc, mpsc};
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
use std::io::{stdout, Write};

pub type Result<T> = std::result::Result<T, GeneratorError>;

// treat `path` as root. `path` is absolute
pub fn generate(path: PathBuf, options: GeneratorOptions) -> Result<()> {
    let thread_pool = Arc::new(Mutex::new(ThreadPool::new(options.threads)?));
    match &options.layer {
        GenerateLayer::Flat => {
            let ignored_files = get_ignored_files_from_root(path.clone(), options.ignored_files.clone())?;
            generate_entry_from_path_flat_and_concurrently(Arc::new(path.clone()), Arc::new(path.clone()), Arc::new(ignored_files), Arc::new(options), thread_pool)?;
        }
        GenerateLayer::Recursive => {
            let ignored_files = get_ignored_files_from_root(path.clone(), options.ignored_files.clone())?;
            generate_entry_from_path_recursively_and_concurrently(Arc::new(path.clone()), Arc::new(path.clone()), Arc::new(ignored_files), Arc::new(options), thread_pool)?;
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

// Only called once in a parsing process
// Will not returned until the entire tree is constructed and all `.wispha` files are written
// `path` and `root_dir` is absolute
fn generate_entry_from_path_recursively_and_concurrently(path: Arc<PathBuf>,
                                                         root_dir: Arc<PathBuf>,
                                                         ignored_files: Arc<Gitignore>,
                                                         options: Arc<GeneratorOptions>,
                                                         thread_pool: Arc<Mutex<ThreadPool>>) -> Result<()> {
    if path.is_dir() {
        let (tx_global, rx_global) = mpsc::channel(); // `tx_global` will be moved into sub routine
        let cloned_path = Arc::clone(&path);
        let cloned_root_dir = Arc::clone(&root_dir);
        let cloned_ignored_files = Arc::clone(&ignored_files);
        let cloned_options = Arc::clone(&options);
        let cloned_thread_pool = Arc::clone(&thread_pool);
        let cloned_tx_global = Sender::clone(&tx_global);
        thread_pool.lock().unwrap().execute(move || {
            let result = generate_entry_from_path_recursively_and_concurrently_sub_routine(cloned_path, cloned_root_dir, cloned_ignored_files, cloned_options, cloned_tx_global, cloned_thread_pool);
            tx_global.send(result).unwrap();
        });
        let mut counter = 0;
        for result in rx_global {
            result?;
            counter += 1;
            print!("\rRecording {} files.", counter);
            stdout().flush().unwrap();
        }
        println!();
        Ok(())
    } else {
        Err(GeneratorError::PathIsNotDir((*path).clone()))
    }
}

// called by `generate_entry_from_path_recursively_and_concurrently` and itself.
fn generate_entry_from_path_recursively_and_concurrently_sub_routine(path: Arc<PathBuf>,
                                                                     root_dir: Arc<PathBuf>,
                                                                     ignored_files: Arc<Gitignore>,
                                                                     options: Arc<GeneratorOptions>,
                                                                     tx_global: Sender<Result<()>>,
                                                                     thread_pool: Arc<Mutex<ThreadPool>>) -> Result<()> {
    let direct_entry = Arc::new(Mutex::new(generate_file_at_path_without_sub_and_sup(Arc::clone(&path), Arc::clone(&options))?));

    // this function is designed to be called with `path` as directory
    let entries = fs::read_dir(&*path).or(Err(GeneratorError::DirCannotRead((*path).clone())))?;
    for entry in entries {
        let entry = entry.or(Err(GeneratorError::Unexpected))?;
        let cloned_ignored_files = Arc::clone(&ignored_files);
        let cloned_options = Arc::clone(&options);
        let cloned_path = Arc::new(entry.path().clone());
        let cloned_root_dir = Arc::clone(&root_dir);
        let cloned_thread_pool = Arc::clone(&thread_pool);
        let cloned_tx_global = Sender::clone(&tx_global);
        if should_include_entry(&entry, Arc::clone(&cloned_ignored_files), Arc::clone(&cloned_options)) {
            if entry.path().is_dir() {
                let link_entry = WisphaLinkEntry {
                    entry_file_path: entry.path().clone().join(PathBuf::from(&options.wispha_name)),
                };
                direct_entry.lock().unwrap().sub_entries.lock().unwrap().push(Arc::new(Mutex::new(WisphaIntermediateEntry::Link(link_entry))));
                thread_pool.lock().unwrap().execute(move || {
                    let tx_global = cloned_tx_global;
                    let result = generate_entry_from_path_recursively_and_concurrently_sub_routine(cloned_path, cloned_root_dir, cloned_ignored_files, cloned_options, Sender::clone(&tx_global), cloned_thread_pool);
                    tx_global.send(result).unwrap();
                });
            } else {
                let sub_entry = generate_file_at_path_without_sub_and_sup(Arc::new(entry.path()), Arc::clone(&options))?;
                direct_entry.lock().unwrap().sub_entries.lock().unwrap().push(Arc::new(Mutex::new(WisphaIntermediateEntry::Direct(sub_entry))));
                tx_global.send(Ok(())).unwrap();
            }
        }
    }
    let locked_entry = direct_entry.lock().unwrap();
    let absolute_path = path.join(PathBuf::from(&options.wispha_name));
    fs::write(&absolute_path, locked_entry.to_file_string(0, &root_dir)?)
        .or(Err(GeneratorError::FileCannotWrite(absolute_path.clone())))?;
    drop(locked_entry);
    Ok(())
}

// Only called once in a parsing process
// Will not returned until the entire tree is constructed and `.wispha` files are written
// `path` and `root_dir` is absolute
fn generate_entry_from_path_flat_and_concurrently(path: Arc<PathBuf>,
                                                  root_dir: Arc<PathBuf>,
                                                  ignored_files: Arc<Gitignore>,
                                                  options: Arc<GeneratorOptions>,
                                                  thread_pool: Arc<Mutex<ThreadPool>>) -> Result<()> {
    if path.is_dir() {
        let this_entry = Arc::new(Mutex::new(WisphaIntermediateEntry::Direct(generate_file_at_path_without_sub_and_sup(Arc::clone(&path), Arc::clone(&options))?)));
        let (tx_global, rx_global) = mpsc::channel();
        let cloned_wispha = Arc::clone(&this_entry);
        let cloned_ignored_files = Arc::clone(&ignored_files);
        let cloned_options = Arc::clone(&options);
        let cloned_path = Arc::clone(&path);
        let cloned_root_dir = Arc::clone(&root_dir);
        let cloned_tx_global = Sender::clone(&tx_global);
        let cloned_thread_pool = Arc::clone(&thread_pool);
        thread_pool.lock().unwrap().execute(move || {
            let tx_global = cloned_tx_global;
            let result = generate_entry_from_path_flat_and_concurrently_sub_routine(cloned_path, cloned_root_dir, cloned_ignored_files, cloned_options, cloned_wispha, Sender::clone(&tx_global), cloned_thread_pool);
            tx_global.send(result).unwrap();
        });
        drop(tx_global);
        let mut counter = 0;
        for result in rx_global {
            result?;
            counter += 1;
            print!("\rRecording {} files.", counter);
            stdout().flush().unwrap();
        }
        println!();
        let absolute_path = this_entry.lock().unwrap().get_direct_entry().unwrap().properties.absolute_path.join(&options.wispha_name);
        fs::write(&absolute_path, this_entry.lock().unwrap().get_direct_entry().unwrap().to_file_string(0, &root_dir)?).or(Err(GeneratorError::FileCannotWrite(absolute_path.clone())))?;
        Ok(())
    } else {
        Err(GeneratorError::PathIsNotDir((*path).clone()))
    }
}

// called by `generate_entry_from_path_flat_and_concurrently` and itself.
fn generate_entry_from_path_flat_and_concurrently_sub_routine(path: Arc<PathBuf>,
                                                              root_dir: Arc<PathBuf>,
                                                              ignored_files: Arc<Gitignore>,
                                                              options: Arc<GeneratorOptions>,
                                                              this_entry: Arc<Mutex<WisphaIntermediateEntry>>,
                                                              tx_global: mpsc::Sender<Result<()>>,
                                                              thread_pool: Arc<Mutex<ThreadPool>>) -> Result<()> {
    let entries = fs::read_dir(&*path).or(Err(GeneratorError::DirCannotRead((*path).clone())))?;
    for entry in entries {
        let entry = entry.or(Err(GeneratorError::Unexpected))?;
        let cloned_wispha = Arc::clone(&this_entry);
        let cloned_ignored_files = Arc::clone(&ignored_files);
        let cloned_options = Arc::clone(&options);
        let cloned_path = Arc::new(entry.path().clone());
        let cloned_root_dir = Arc::clone(&root_dir);
        let cloned_tx_global = Sender::clone(&tx_global);
        let cloned_thread_pool = Arc::clone(&thread_pool);
        if should_include_entry(&entry, Arc::clone(&cloned_ignored_files), Arc::clone(&cloned_options)) {
            if entry.path().is_dir() {
                let sub_entry = Arc::new(Mutex::new(WisphaIntermediateEntry::Direct(generate_file_at_path_without_sub_and_sup(Arc::clone(&cloned_path), Arc::clone(&options))?)));
                this_entry.lock().unwrap().get_direct_entry_mut().unwrap().sub_entries.lock().unwrap().push(Arc::clone(&sub_entry));
                thread_pool.lock().unwrap().execute(move || {
                    let tx_global = cloned_tx_global;
                    let result = generate_entry_from_path_flat_and_concurrently_sub_routine(cloned_path, cloned_root_dir, cloned_ignored_files, cloned_options, cloned_wispha, Sender::clone(&tx_global), cloned_thread_pool);
                    tx_global.send(result).unwrap();
                });
            } else {
                let sub_entry = generate_file_at_path_without_sub_and_sup(Arc::clone(&path), Arc::clone(&options))?;
                this_entry.lock().unwrap().get_direct_entry_mut().unwrap().sub_entries.lock().unwrap().push(Arc::new(Mutex::new(WisphaIntermediateEntry::Direct(sub_entry))));
                tx_global.send(Ok(())).unwrap();
            }
        }
    }
    Ok(())
}
