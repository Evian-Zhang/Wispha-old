use std::env;
use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};
use std::io;
use std::rc::{Rc, Weak};
use std::cell::{RefCell, Ref};
use std::ops::Add;
use crate::wispha::{self, WisphaEntry, WisphaEntryProperties, WisphaEntryType, WisphaFatEntry, WisphaIntermediateEntry};
use crate::generator::error::GeneratorError;
use ignore::{Walk, WalkBuilder};

pub mod error;
mod converter;

pub type Result<T> = std::result::Result<T, GeneratorError>;

// treat `path` as root. `path` is absolute
pub fn generate(path: &PathBuf) -> Result<()> {
    let root = generate_file_at_path(&path, &path, &get_ignored_files_from_root(path)?)?;
    fs::write(&path.join(PathBuf::from(&wispha::DEFAULT_FILE_NAME_STR)), &root.to_file_string(0, &path)?)
        .or(Err(GeneratorError::Unexpected))?;
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

// `path` and `root_dir` are absolute. Returned `WisphaEntry` has no `sup_entry`. Generated intermediate entry's path is relative
fn generate_file_at_path(path: &PathBuf, root_dir: &PathBuf, ignored_files: &Vec<PathBuf>) -> Result<WisphaEntry> {
    let mut wispha_entry = generate_file_at_path_without_sub_and_sup(path)?;
    if path.is_dir() {
        for entry in fs::read_dir(&path).or(Err(GeneratorError::DirCannotRead(path.clone())))? {
            let entry = entry.or(Err(GeneratorError::Unexpected))?;
            if ignored_files.contains(&entry.path()) {
                let mut sub_entry = generate_file_at_path(&entry.path(), root_dir, ignored_files)?;
                if (&entry.path()).is_dir() {
                    let absolute_path = sub_entry.properties.absolute_path
                        .join(PathBuf::from(&wispha::DEFAULT_FILE_NAME_STR));
                    fs::write(absolute_path, &sub_entry.to_file_string(0, root_dir)?)
                        .or(Err(GeneratorError::Unexpected))?;

                    let relative_path = PathBuf::from(&sub_entry.properties.name)
                        .join(PathBuf::from(&wispha::DEFAULT_FILE_NAME_STR));

                    let intermediate_entry = WisphaIntermediateEntry {
                        entry_file_path: relative_path,
                    };

                    wispha_entry.sub_entries.try_borrow_mut().or(Err(GeneratorError::Unexpected))?
                        .push(Rc::new(RefCell::new(WisphaFatEntry::Intermediate(intermediate_entry))));
                } else {
                    wispha_entry.sub_entries.try_borrow_mut().or(Err(GeneratorError::Unexpected))?
                        .push(Rc::new(RefCell::new(WisphaFatEntry::Immediate(sub_entry))));
                }
            }
        }
    }
    Ok(wispha_entry)
}
