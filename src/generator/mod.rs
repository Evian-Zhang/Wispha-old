use std::env;
use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};
use std::io;
use std::rc::{Rc, Weak};
use std::cell::{RefCell, Ref};
use std::ops::Add;
use crate::wispha::{WisphaEntry, WisphaEntryProperties, WisphaEntryType, WisphaFatEntry, WisphaIntermediateEntry};
use crate::generator::error::GeneratorError;
use std::borrow::Borrow;

mod error;
mod converter;

pub type Result<T> = std::result::Result<T, GeneratorError>;

static DEFAULT_FILE_NAME_STR: &str = "LOOKME.wispha";

pub fn generate() -> Result<()> {
//    let current_path = env::current_dir().or(Err(GeneratorError::DirCannotRead))?;
    let current_path = PathBuf::from("/Users/evian/Downloads/llvm/llvm/");
//    let root = generate_from(&current_path, Weak::new())?;
//    print(&root, 0);
//    println!("{}", root.to_file_string(0)?);
    let root = generate_file_at_path(&current_path, &current_path)?;
    fs::write(&current_path.join(PathBuf::from(&DEFAULT_FILE_NAME_STR)), &root.to_file_string(0, &current_path)?)
        .or(Err(GeneratorError::Unexpected))?;
    Ok(())
}

//fn print(entry: &WisphaEntry, indent: i32) {
//    let mut blank = 0;
//    while blank < indent {
//        print!("\t");
//        blank += 1;
//    }
//    println!("{}", entry.properties.name);
//    for sub_entry in &*entry.sub_entries.borrow() {
//        print(&*(**sub_entry).borrow(), indent + 1);
//    }
//}

pub fn get_ignored_files_at_dir(dir: &PathBuf) -> Vec<PathBuf> {
    Vec::new()
}

fn generate_link_file_at_path(path: &PathBuf) -> Result<WisphaEntry> {
    let mut wispha_entry = WisphaEntry::default();

    wispha_entry.properties.name = path.file_name().ok_or(GeneratorError::NameNotDetermined)?
        .to_str().ok_or(GeneratorError::NameNotValid)?
        .to_string();

    wispha_entry.properties.absolute_path = path.clone();

    wispha_entry.properties.entry_type = match path.is_dir() {
        true => WisphaEntryType::Directory,
        false => WisphaEntryType::File,
    };

    Ok(wispha_entry)
}

pub fn generate_file_at_path(path: &PathBuf, root_dir: &PathBuf) -> Result<WisphaEntry> {
    let mut wispha_entry = generate_link_file_at_path(path)?;
    if path.is_dir() {
        let ignored_files = get_ignored_files_at_dir(&path);
        for entry in fs::read_dir(&path).or(Err(GeneratorError::DirCannotRead))? {
            let entry = entry.or(Err(GeneratorError::Unexpected))?;
            if !ignored_files.contains(&entry.path()) {
                let mut sub_entry = generate_file_at_path(&entry.path(), root_dir)?;
                if (&entry.path()).is_dir() {
                    let absolute_path = sub_entry.properties.absolute_path
                        .join(PathBuf::from(&DEFAULT_FILE_NAME_STR));
                    fs::write(absolute_path, &sub_entry.to_file_string(0, root_dir)?)
                        .or(Err(GeneratorError::Unexpected))?;

                    let relative_path = PathBuf::from(&sub_entry.properties.name)
                        .join(PathBuf::from(&DEFAULT_FILE_NAME_STR));

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
