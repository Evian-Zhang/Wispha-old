use ignore::{gitignore::{GitignoreBuilder, Gitignore}};

pub mod option;
use option::*;

pub mod error;
use error::*;

use std::path::PathBuf;

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
    let ignored = get_ignored_files_from_root(&root_dir, &options.ignored_files);
    Ok(())
}