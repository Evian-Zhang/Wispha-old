use std::env;
use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};
use std::io;
use std::rc::{Rc, Weak};
use std::cell::{RefCell, Ref};
use std::ops::Add;
use crate::wispha::{self, WisphaEntry, WisphaEntryProperties, WisphaEntryType, WisphaFatEntry, WisphaIntermediateEntry};
use ignore::{Walk, WalkBuilder};

pub mod error;
use error::GeneratorError;
mod converter;
pub mod option;
use option::*;

pub type Result<T> = std::result::Result<T, GeneratorError>;

// treat `path` as root. `path` is absolute
pub fn generate(path: &PathBuf, options: GeneratorOptions) -> Result<()> {
    let root = match options.layer {
        GenerateLayer::Flat => {
            generate_file_at_path_flat(&path, &path, &get_ignored_files_from_root(path)?)?
        },
        GenerateLayer::Recursive => {
            generate_file_at_path_recursively(&path, &path, &get_ignored_files_from_root(path)?)?
        },
    };
    let root_path = path.join(PathBuf::from(&wispha::DEFAULT_FILE_NAME_STR));
    fs::write(&root_path, &root.to_file_string(0, &path)?)
        .or(Err(GeneratorError::FileCannotWrite(root_path.clone())))?;
    Ok(())
}

// traverse from `root_dir` to find .wisphaignore file. `root_dir` is absolute. If there is none, return `Ok(Vec::new())`
fn get_ignored_files_from_root(root_dir: &PathBuf) -> Result<Vec<PathBuf>> {
    let walk = WalkBuilder::new(root_dir)
        .standard_filters(false)
        .parents(true)
        .add_custom_ignore_filename(wispha::IGNORE_FILE_NAME_STR)
        .build();

    let mut ignored_files: Vec<PathBuf> = Vec::new();
    for dir_entry in walk {
        let dir_entry = dir_entry?;
        ignored_files.push(dir_entry.path().to_path_buf());
    }
    Ok(ignored_files)
}

// `path` is absolute
fn generate_file_at_path_without_sub_and_sup(path: &PathBuf) -> Result<WisphaEntry> {
    let mut wispha_entry = WisphaEntry::default();

    wispha_entry.properties.name = path.file_name().ok_or(GeneratorError::NameNotDetermined(path.clone()))?
        .to_str().ok_or(GeneratorError::NameNotValid(path.clone()))?
        .to_string();

    wispha_entry.properties.absolute_path = path.clone();

    wispha_entry.properties.entry_type = match path.is_dir() {
        true => WisphaEntryType::Directory,
        false => WisphaEntryType::File,
    };

    Ok(wispha_entry)
}

// `path` and `root_dir` are absolute. Returned `WisphaEntry` has no `sup_entry`. Generated intermediate entry's path is relative. Write all sub_entry to disk
fn generate_file_at_path_recursively(path: &PathBuf, root_dir: &PathBuf, ignored_files: &Vec<PathBuf>) -> Result<WisphaEntry> {
    let mut wispha_entry = generate_file_at_path_without_sub_and_sup(path)?;
    if path.is_dir() {
        for entry in fs::read_dir(&path).or(Err(GeneratorError::DirCannotRead(path.clone())))? {
            let entry = entry.or(Err(GeneratorError::Unexpected))?;
            if ignored_files.contains(&entry.path()) {
                let mut sub_entry = generate_file_at_path_recursively(&entry.path(), root_dir, ignored_files)?;
                if (&entry.path()).is_dir() {
                    let absolute_path = sub_entry.properties.absolute_path
                        .join(PathBuf::from(&wispha::DEFAULT_FILE_NAME_STR));
                    fs::write(&absolute_path, &sub_entry.to_file_string(0, root_dir)?)
                        .or(Err(GeneratorError::FileCannotWrite(absolute_path.clone())))?;

                    let relative_path = PathBuf::from(&sub_entry.properties.name)
                        .join(PathBuf::from(&wispha::DEFAULT_FILE_NAME_STR));

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

// `path` and `root_dir` are absolute. Returned `WisphaEntry` has no `sup_entry`. Not write sub_entry to disk
fn generate_file_at_path_flat(path: &PathBuf, root_dir: &PathBuf, ignored_files: &Vec<PathBuf>) -> Result<WisphaEntry> {
    let mut wispha_entry = generate_file_at_path_without_sub_and_sup(path)?;
    if path.is_dir() {
        for entry in fs::read_dir(&path).or(Err(GeneratorError::DirCannotRead(path.clone())))? {
            let entry = entry.or(Err(GeneratorError::Unexpected))?;
            if ignored_files.contains(&entry.path()) {
                let mut sub_entry = generate_file_at_path_flat(&entry.path(), root_dir, ignored_files)?;
                wispha_entry.sub_entries.borrow_mut()
                    .push(Rc::new(RefCell::new(WisphaFatEntry::Immediate(sub_entry))));
            }
        }
    }
    Ok(wispha_entry)
}
