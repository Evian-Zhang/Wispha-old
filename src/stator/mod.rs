use ignore::{gitignore::{GitignoreBuilder, Gitignore}};
use git2::{Repository, TreeWalkMode};

pub mod option;
use option::*;

pub mod error;
use error::*;

use crate::parser::option::ParserOptions;
use crate::config_reader;
use crate::parser;
use crate::wispha::common::*;

use std::path::PathBuf;
use std::rc::Rc;
use std::cell::RefCell;
use std::fs;

type Result<T> = std::result::Result<T, StatorError>;

pub fn state_from_path(path: &PathBuf, options: StatorOptions) -> Result<Vec<PathBuf>> {
    let root_dir = path.parent().unwrap().to_path_buf();
    let ignored = get_ignored_files_from_root(&root_dir, &options.ignored_files)?;

    let mut parser_options = ParserOptions::default();
    let config = config_reader::read_configs_in_dir(&path).or_else(|error| Err(StatorError::ConfigError(error)))?;
    if let Some(config) = config {
        parser_options.update_from_config(&config).or_else(|error| Err(StatorError::ParserOptionError(error)))?;
    }
    let root = parser::parse(&path, parser_options).or_else(|error| Err(StatorError::ParserError(error)))?;

    let mut recorded_paths = vec![];
    let entry = Rc::clone(&root);
    get_recorded_files_from_root(entry, &mut recorded_paths);

    let git_files = if options.git {
        get_git_files_from_root(&root_dir)?
    } else {
        vec![]
    };

    let mut unrecorded_paths = vec![];
    get_unrecorded_files_from_root(&root.borrow().properties.absolute_path, &mut unrecorded_paths, &recorded_paths, &ignored, &git_files, &options)?;
    Ok(unrecorded_paths)
}

fn get_ignored_files_from_root(root_dir: &PathBuf, ignored_files: &Vec<String>) -> Result<Gitignore> {
    let mut ignore_builder = GitignoreBuilder::new(root_dir);
    for ignored_file in ignored_files {
        ignore_builder.add_line(None, ignored_file).or_else(|error| Err(StatorError::IgnoreError(error)))?;
    }
    let wispha_ignore = ignore_builder.build().or_else(|error| Err(StatorError::IgnoreError(error)))?;
    Ok(wispha_ignore)
}

fn get_recorded_files_from_root(root: Rc<RefCell<WisphaEntry>>, recorded_paths: &mut Vec<PathBuf>) {
    recorded_paths.push(root.borrow().properties.absolute_path.clone());
    for sub_entry in &*root.borrow().sub_entries.borrow() {
        get_recorded_files_from_root(Rc::clone(sub_entry), recorded_paths);
    }
}

fn get_git_files_from_root(root_dir: &PathBuf) -> Result<Vec<PathBuf>> {
    let repository = Repository::open(root_dir).or(Err(StatorError::CanNotOpenGitRepository(root_dir.clone())))?;
    let head_tree = repository.head().or(Err(StatorError::Unexpected))?.peel_to_tree().or(Err(StatorError::Unexpected))?;
    let mut git_files = vec![];
    head_tree.walk(TreeWalkMode::PreOrder, |root, entry| {
        git_files.push(PathBuf::from(root.to_string() + entry.name().unwrap()));
        1
    }).unwrap();
    Ok(git_files)
}

// If a directory is not recorded, entries of this directory are not included in unrecorded_paths
fn get_unrecorded_files_from_root(root_dir: &PathBuf, unrecorded_paths: &mut Vec<PathBuf>, recorded_paths: &Vec<PathBuf>, ignored: &Gitignore, git_files: &Vec<PathBuf>, options: &StatorOptions) -> Result<()> {
    if is_path_unrecorded(root_dir, &ignored, &recorded_paths, git_files, options) {
        unrecorded_paths.push(root_dir.clone());
        return Ok(());
    }
    if root_dir.is_dir() {
        for entry in fs::read_dir(root_dir).or(Err(StatorError::DirCannotRead(root_dir.clone())))? {
            let entry = entry.unwrap();
            get_unrecorded_files_from_root(&entry.path(), unrecorded_paths, recorded_paths, ignored, git_files, options)?;
        }
    }
    Ok(())
}

fn is_path_unrecorded(path: &PathBuf, wispha_ignore: &Gitignore, recorded_paths: &Vec<PathBuf>, git_files: &Vec<PathBuf>, options: &StatorOptions) -> bool {
    if wispha_ignore.matched(path, path.is_dir()).is_ignore() {
        return false;
    }

    if recorded_paths.contains(&path.to_path_buf()) {
        return false;
    }

    if options.git {
        if git_files.contains(path) {
            return false;
        }
    }

    if path.file_name().unwrap().to_str().map(|s| s.starts_with(".")).unwrap_or(false) {
        return options.allow_hidden_files;
    }

    true
}