use std::env;
use std::fs::{self, DirEntry};
use std::error::Error;
use std::path::{Path, PathBuf};
use std::io;
use crate::wispha::{WisphaEntry, WisphaEntryProperties, WisphaEntryType};

pub enum GeneratorError {

}

impl Error for GeneratorError {

}

pub fn generate() -> Result<(), GeneratorError> {

}

pub fn generate_at_path(path: &Path, ignored_files: &[PathBuf]) -> io::Result<()> {
    let valid_entries = get_valid_entries_at_path(path, ignored_files)?;

    Ok(())
}

fn get_valid_entries_at_path(path: &Path, ignored_files: &[PathBuf]) -> io::Result<Vec<PathBuf>> {
    if path.is_dir() {
        let mut valid_entries: Vec<PathBuf> = Vec::new();
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            if ignored_files.contains(&entry.path()) {
                continue;
            }
            valid_entries.push(entry.path());
        }
        return Ok(valid_entries);
    }
    Err
}

fn wispha_units_from_entries(entries: &[PathBuf]) -> Vec<WisphaEntry> {
    let mut wispha_entries: Vec<WisphaEntry> = Vec::new();
    for entry in entries {
        let entry_type = match entry.is_dir() {
            true => WisphaEntryType::Directory,
            false => WisphaEntryType::File,
        };

        let name = entry.file_name()?.to_str()?.to_string();

        let description = String::new();

        let absolute_path = entry.clone();

        let properties = WisphaEntryProperties { entry_type, name, description,
            absolute_path };

        let wispha_entry = WisphaEntry { properties: properties, sub_entries: Vec::new() };

        wispha_entries.push(wispha_entry);
    }
    wispha_entries
}