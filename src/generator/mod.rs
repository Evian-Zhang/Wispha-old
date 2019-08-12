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
//    let current_path = env::current_dir().or(Err(GeneratorError::DirCannotRead))?;
    let current_path = PathBuf::from("/Users/evian/Downloads/llvm/llvm/");
//    let root = generate_from(&current_path, Weak::new())?;
//    print(&root, 0);
//    println!("{}", root.to_file_string(0)?);
    let root = generate_file_at_path(&current_path)?;
    if let () = fs::write(&current_path.join(PathBuf::from(&DEFAULT_FILE_NAME_STR)), &root.to_file_string(0)?).or(Err(GeneratorError::Unexpected))? {

    } else {
        println!("Error!");
    }
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

//pub fn generate_wispha_entry_at_path(path: &PathBuf, sup_entry: Weak<WisphaEntry>) -> Result<WisphaEntry> {
//    let mut wispha_entry = WisphaEntry::default();
//
//    wispha_entry.properties.name = path.file_name().ok_or(GeneratorError::NameNotDetermined)?
//        .to_str().ok_or(GeneratorError::NameNotValid)?
//        .to_string();
//
//    wispha_entry.properties.absolute_path = path.clone();
//
//    let (entry_type, entry_file_path) = match path.is_dir() {
//        true => (
//            WisphaEntryType::Directory,
//            Some(PathBuf::from(&wispha_entry.properties.name)
//                .join(PathBuf::from(&DEFAULT_FILE_NAME_STR))
//            )
//        ),
//        false => (WisphaEntryType::File, None),
//    };
//
//    wispha_entry.properties.entry_type = entry_type;
//
//    wispha_entry.entry_file_path = entry_file_path;
//
//    Ok(wispha_entry)
//}

pub fn get_ignored_files_at_dir(dir: &PathBuf) -> Vec<PathBuf> {
    Vec::new()
}

//pub fn generate_from(path: &PathBuf, sup_entry: Weak<WisphaEntry>) -> Result<Rc<WisphaEntry>> {
//    let root = RefCell::new(Rc::new(generate_wispha_entry_at_path(path, sup_entry)?));
//    if path.is_dir() {
//        let ignored_files = get_ignored_files_at_dir(&path);
//        for entry in fs::read_dir(path).or(Err(GeneratorError::DirCannotRead))? {
//            let entry = entry.or(Err(GeneratorError::Unexpected))?;
//            if !ignored_files.contains(&entry.path()) {
//                let wispha_entry = generate_from(
//                    &entry.path(),
//                    Rc::downgrade(&*root.try_borrow().or(Err(GeneratorError::Unexpected))?)
//                )?;
//
//                // May be removed
//                root.try_borrow_mut().or(Err(GeneratorError::Unexpected))?
//                    .sub_entries.try_borrow_mut().or(Err(GeneratorError::Unexpected))?
//                    .push(wispha_entry);
//            }
//        }
//    }
//
//    let root = root.into_inner();
//    let root_file_string = root.to_file_string(0)?;
//    println!("\n\n\n\n\n{}", root_file_string);
//    Ok(root)
//}

pub fn generate_intermediate_file_at_path(path: &PathBuf) -> Result<WisphaEntry> {
    let mut wispha_entry = WisphaEntry::default();

    wispha_entry.properties.name = path.file_name().ok_or(GeneratorError::NameNotDetermined)?
        .to_str().ok_or(GeneratorError::NameNotValid)?
        .to_string();

    wispha_entry.properties.absolute_path = path.clone();

    let (entry_type, entry_file_path) = match path.is_dir() {
        true => (
            WisphaEntryType::Directory,
            None
        ),
        false => (WisphaEntryType::File, None),
    };

    wispha_entry.properties.entry_type = entry_type;

    wispha_entry.entry_file_path = entry_file_path;
    Ok(wispha_entry)
}

pub fn generate_file_at_path(path: &PathBuf) -> Result<WisphaEntry> {
    println!("zs");
        println!("{}", &path.to_str().unwrap());
    let mut wispha_entry = generate_intermediate_file_at_path(path)?;
    if path.is_dir() {
        let ignored_files = get_ignored_files_at_dir(&path);
        for entry in fs::read_dir(&path).or(Err(GeneratorError::DirCannotRead))? {
            let entry = entry.or(Err(GeneratorError::Unexpected))?;
            println!("{}", &entry.path().to_str().unwrap());
            if !ignored_files.contains(&entry.path()) {
                let mut sub_entry = generate_file_at_path(&entry.path())?;
                if sub_entry.properties.absolute_path.is_dir() {
                    sub_entry.entry_file_path = Some(PathBuf::from(&sub_entry.properties.name)
                        .join(PathBuf::from(&DEFAULT_FILE_NAME_STR))
                    );
//                    println!("{}", sub_entry.entry_file_path.clone().unwrap().to_str().unwrap());
                    fs::write(&sub_entry.entry_file_path.clone().ok_or(GeneratorError::Unexpected)?, &sub_entry.to_file_string(0)?).or(Err(GeneratorError::Unexpected))?;
                }
                wispha_entry.sub_entries.try_borrow_mut().or(Err(GeneratorError::Unexpected))?
                    .push(Rc::new(sub_entry));
            }
        }
        println!("yyk");
    }
//    println!("{}", wispha_entry.to_file_string(0)?);
    Ok(wispha_entry)
}
