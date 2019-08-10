use std::env;
use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};
use std::io;
use crate::wispha::{WisphaEntry, WisphaEntryProperties, WisphaEntryType};
use crate::generator::error::GeneratorError;

mod error;

pub type Result<T> = std::result::Result<T, GeneratorError>;

static DEFAULT_FILE_NAME_STR: &str = "LOOKME.wispha";

pub fn generate() -> Result<()> {
    let current_path = env::current_dir().or(Err(GeneratorError::DirCannotRead))?;
    let root = generate_from(&current_path, None)?;
    Ok(())
}

pub fn generate_wispha_entry_at_path<'a>(path: &PathBuf, sup_entry: Option<&'a WisphaEntry>) ->
                                                                             Result<WisphaEntry<'a>> {
    let name = path.file_name().ok_or(GeneratorError::NameNotDetermined)?.to_str().ok_or
    (GeneratorError::NameNotValid)?.to_string();

    let description = String::new();

    let absolute_path = path.clone();

    println!("{}, {}", name, absolute_path.to_str().unwrap());

    let (entry_type, entry_file_path) = match path.is_dir() {
        true => (WisphaEntryType::Directory, Some(PathBuf::from(&name).join
        (PathBuf::from(&DEFAULT_FILE_NAME_STR)))),
        false => (WisphaEntryType::File, None),
    };

    let properties = WisphaEntryProperties { entry_type, name, description,
        absolute_path };

    Ok(WisphaEntry { properties, entry_file_path, sup_entry, sub_entries: Vec::new() })
}

pub fn get_ignored_files_at_dir(dir: &PathBuf) -> Vec<PathBuf> {
    Vec::new()
}

pub fn generate_from<'a>(path: &PathBuf, sup_entry: Option<&'a WisphaEntry>) ->
                                                                             Result<WisphaEntry<'a>> {
    let mut root = generate_wispha_entry_at_path(path, sup_entry)?;
    if path.is_dir() {
        let ignored_files = get_ignored_files_at_dir(&path);
        for entry in fs::read_dir(path).or(Err(GeneratorError::DirCannotRead))? {
            let entry = entry.or(Err(GeneratorError::Unexpected))?;
            if !ignored_files.contains(&entry.path()) {
                let wispha_entry = generate_from(&entry.path(), Some(&root))?;
                root.sub_entries.push(Box::new(wispha_entry));
            }
        }
    }
    Ok(root)
}
