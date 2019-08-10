use std::env;
use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};
use std::io;
use crate::wispha::{WisphaEntry, WisphaEntryProperties, WisphaEntryType};
use crate::generator::error::GeneratorError;

mod error;

pub type Result<T> = std::result::Result<T, GeneratorError>;

static DEFAULT_FILE_NAME: PathBuf = PathBuf::from("LOOKME.wispha");

pub fn generate() -> Result<()> {
    Ok(())
}

pub fn generate_at_path(path: &Path, ignored_files: &[PathBuf]) -> Result<()> {
    let valid_entries = get_valid_entries_at_path(path, ignored_files)?;

    Ok(())
}

fn get_valid_entries_at_path(path: &Path, ignored_files: &[PathBuf]) -> Result<Vec<PathBuf>> {
    if path.is_dir() {
        let mut valid_entries: Vec<PathBuf> = Vec::new();
        for entry in fs::read_dir(path).or(Err(GeneratorError::DirCannotRead))? {
            let entry = entry.or(Err(GeneratorError::Unexpected))?;
            if ignored_files.contains(&entry.path()) {
                continue;
            }
            valid_entries.push(entry.path());
        }
        return Ok(valid_entries);
    }
    Err(GeneratorError::PathIsNotDir)
}

fn wispha_units_from_entries(entries: &[PathBuf]) -> Result<Vec<WisphaEntry>> {
    let mut wispha_entries: Vec<WisphaEntry> = Vec::new();
    for entry in entries {
        let name = entry.file_name().ok_or(GeneratorError::NameNotDetermined)?.to_str().ok_or
        (GeneratorError::NameNotValid)?.to_string();

        let description = String::new();

        let absolute_path = entry.clone();

        let (entry_type, entry_file_path) = match entry.is_dir() {
            true => (WisphaEntryType::Directory, Some(PathBuf::from(name).join
            (DEFAULT_FILE_NAME))),
            false => (WisphaEntryType::File, None),
        };

        let properties = WisphaEntryProperties { entry_type, name, description,
            absolute_path };

        let wispha_entry = WisphaEntry { properties, entry_file_path: None, sup_entry: None, sub_entries: Vec::new() };

        wispha_entries.push(wispha_entry);
    }
    Ok(wispha_entries)
}