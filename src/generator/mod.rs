use std::env;
use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};
use std::io;
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::ops::Add;
use crate::wispha::{WisphaEntry, WisphaEntryProperties, WisphaEntryType};
use crate::generator::error::GeneratorError;

mod error;
mod converter;

pub type Result<T> = std::result::Result<T, GeneratorError>;

static DEFAULT_FILE_NAME_STR: &str = "LOOKME.wispha";

pub fn generate() -> Result<()> {
    let current_path = env::current_dir().or(Err(GeneratorError::DirCannotRead))?;
    let root = generate_from(&current_path, Weak::new())?;
//    print(&root, 0);
    println!("{}", root.to_file_string(0)?);
    Ok(())
}

fn print(entry: &WisphaEntry, indent: i32) {
    let mut blank = 0;
    while blank < indent {
        print!("\t");
        blank += 1;
    }
    println!("{}", entry.properties.name);
    for sub_entry in &*entry.sub_entries.borrow() {
        print(&sub_entry, indent + 1);
    }
}

pub fn generate_wispha_entry_at_path(path: &PathBuf, sup_entry: Weak<WisphaEntry>) ->
                                                                             Result<WisphaEntry> {
    let name = path.file_name().ok_or(GeneratorError::NameNotDetermined)?
        .to_str().ok_or(GeneratorError::NameNotValid)?
        .to_string();

    let description = String::new();

    let absolute_path = path.clone();

    let (entry_type, entry_file_path) = match path.is_dir() {
        true => (
            WisphaEntryType::Directory,
//            Some(PathBuf::from(&name)
////                .join(PathBuf::from(&DEFAULT_FILE_NAME_STR))
////            )
            None
        ),
        false => (WisphaEntryType::File, None),
    };

    let properties = WisphaEntryProperties { entry_type, name, description, absolute_path };

    Ok(WisphaEntry {
        properties,
        entry_file_path,
        sup_entry: RefCell::new(sup_entry),
        sub_entries: RefCell::new(Vec::new())
    })
}

pub fn get_ignored_files_at_dir(dir: &PathBuf) -> Vec<PathBuf> {
    Vec::new()
}

pub fn generate_from(path: &PathBuf, sup_entry: Weak<WisphaEntry>) -> Result<Rc<WisphaEntry>> {
    let root = RefCell::new(Rc::new(generate_wispha_entry_at_path(path, sup_entry)?));
    if path.is_dir() {
        let ignored_files = get_ignored_files_at_dir(&path);
        for entry in fs::read_dir(path).or(Err(GeneratorError::DirCannotRead))? {
            let entry = entry.or(Err(GeneratorError::Unexpected))?;
            if !ignored_files.contains(&entry.path()) {
                let wispha_entry = generate_from(
                    &entry.path(),
                    Rc::downgrade(&*root.try_borrow().or(Err(GeneratorError::Unexpected))?)
                )?;
                root.try_borrow_mut().or(Err(GeneratorError::Unexpected))?
                    .sub_entries.try_borrow_mut().or(Err(GeneratorError::Unexpected))?
                    .push(wispha_entry);
            }
        }
    }
    Ok(root.into_inner())
}
