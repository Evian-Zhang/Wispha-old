use ignore::{gitignore::{GitignoreBuilder, Gitignore}};

pub mod option;
use option::*;

pub mod error;
use error::*;

use std::path::PathBuf;
use crate::parser::option::ParserOptions;
use crate::config_reader;
use crate::parser::Parser;
use std::rc::Rc;
use std::cell::RefCell;
use std::fs;
use crate::wispha::WisphaFatEntry;

type Result<T> = std::result::Result<T, StatorError>;

fn get_ignored_files_from_root(root_dir: &PathBuf, ignored_files: &Vec<String>) -> Result<Gitignore> {
    let mut ignore_builder = GitignoreBuilder::new(root_dir);
    for ignored_file in ignored_files {
        ignore_builder.add_line(None, ignored_file).or_else(|error| Err(StatorError::IgnoreError(error)))?;
    }
    let wispha_ignore = ignore_builder.build().or_else(|error| Err(StatorError::IgnoreError(error)))?;
    Ok(wispha_ignore)
}

pub fn state_from_path(path: &PathBuf, options: StatorOptions) -> Result<Vec<PathBuf>> {
    let root_dir = path.parent().unwrap().to_path_buf();
    let ignored = get_ignored_files_from_root(&root_dir, &options.ignored_files)?;

    let mut options = ParserOptions::default();
    let config = config_reader::read_configs_in_dir(&path)?;
    if let Some(config) = config {
        options.update_from_config(&config)?;
    }
    let mut parser = Parser::new();
    let root = parser.parse(&path, options)?;

    let mut recorded_paths = vec![];
    let mut entry = Rc::clone(&root);
    get_recorded_files_from_root(entry, recorded_paths);

    let mut unrecorded_paths = vec![];
    get_unrecorded_files_from_root(&root.borrow().get_immediate_entry().unwrap().properties.absolute_path, &mut unrecorded_paths, &recorded_paths, &ignored);
    Ok(unrecorded_paths)
}

fn get_recorded_files_from_root(root: Rc<RefCell<WisphaFatEntry>>, recorded_paths: &mut Vec<PathBuf>) {
    recorded_paths.push(root.borrow().get_immediate_entry().unwrap().properties.absolute_path.clone());
    for subentry in &*root.borrow().get_immediate_entry().unwrap().sub_entries.borrow() {
        get_recorded_files_from_root(Rc::clone(subentry), recorded_paths);
    }
}

fn get_unrecorded_files_from_root(root_dir: &PathBuf, unrecorded_paths: &mut Vec<PathBuf>, recorded_paths: &Vec<PathBuf>, ignored: &Gitignore) {
    if !recorded_paths.contains(root_dir) && !ignored.matched(root_dir, root_dir.is_dir()).is_ignore() {
        unrecorded_paths.push(root_dir.clone());
    }
    for entry in fs::read_dir(root_dir).unwrap() {
        let entry = entry?;
        get_unrecorded_files_from_root(&entry.path(), unrecorded_paths, recorded_paths, ignored);
    }
}