use std::env;
use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};
use std::ops::Add;
use std::fmt::Write as FmtWrite;
use std::io::Write as IoWrite;
use crate::wispha::{self, WisphaEntry, WisphaEntryProperties, WisphaEntryType};
use crate::generator::{self, error::GeneratorError};

type Result<T> = std::result::Result<T, GeneratorError>;

impl WisphaEntryProperties {
    fn to_string(&self, depth: u32, root_dir: &PathBuf) -> Result<String> {
        let mut begin_mark = String::new();
        let mut counter = 0;
        while counter <= depth {
            write!(&mut begin_mark, "{}", wispha::BEGIN_MARK).or(Err(GeneratorError::Unexpected));
            counter += 1;
        }

        let absolute_path_header_string = format!("{} [{}]", begin_mark, wispha::ABSOLUTE_PATH_HEADER);
        let relative_path = self.absolute_path.clone()
            .strip_prefix(root_dir).or(Err(GeneratorError::Unexpected))?
            .to_path_buf();
        let dir_absolute_path_str = PathBuf::from(wispha::ROOT_DIR)
            .join(relative_path)
            .to_str().ok_or(GeneratorError::Unexpected)?
            .to_string();
        let absolute_path_string = format!("{}{}{}{}",
                                           absolute_path_header_string,
                                           wispha::LINE_SEPARATOR,
                                           dir_absolute_path_str,
                                           wispha::LINE_SEPARATOR);

        let name_header_string = format!("{} [{}]", begin_mark, wispha::NAME_HEADER);
        let name_string = format!("{}{}{}{}",
                                  name_header_string,
                                  wispha::LINE_SEPARATOR,
                                  &self.name,
                                  wispha::LINE_SEPARATOR);

        let entry_type_header_string = format!("{} [{}]", begin_mark, wispha::ENTRY_TYPE_HEADER);
        let entry_type_string = format!("{}{}{}{}",
                                        entry_type_header_string,
                                        wispha::LINE_SEPARATOR,
                                        &self.entry_type.to_str(),
                                        wispha::LINE_SEPARATOR);

        let description_header_string = format!("{} [{}]", begin_mark, wispha::DESCRIPTION_HEADER);
        let description_string = format!("{}{}{}{}",
                                         description_header_string,
                                         wispha::LINE_SEPARATOR,
                                         &self.description,
                                         wispha::LINE_SEPARATOR);

        return Ok([absolute_path_string,
            name_string,
            entry_type_string,
            description_string].join(wispha::LINE_SEPARATOR));
    }
}

impl WisphaEntry {
    pub fn to_file_string(&self, depth: u32, root_dir: &PathBuf) -> Result<String> {
        let mut begin_mark = String::new();
        let mut counter = 0;
        while counter <= depth {
            write!(&mut begin_mark, "{}", wispha::BEGIN_MARK).or(Err(GeneratorError::Unexpected));
            counter += 1;
        }

        let properties_string = self.properties.to_string(depth, root_dir)?;

        if let Some(entry_file_path) = &self.entry_file_path {
            let entry_file_path_header_string = format!("{} [{}]", begin_mark,
                                                        wispha::ENTRY_FILE_PATH_HEADER);
            let entry_file_path_string = format!("{}{}{}{}",
                                                 entry_file_path_header_string,
                                                 wispha::LINE_SEPARATOR,
                                                 entry_file_path.to_str().ok_or(GeneratorError::NameNotValid)?,
                                                 wispha::LINE_SEPARATOR);
            return Ok([properties_string, entry_file_path_string].join(wispha::LINE_SEPARATOR));
        } else {
            let mut sub_entry_strings: Vec<String> = Vec::new();
            let sub_entries_header_string = format!("{} [{}]", begin_mark, wispha::SUB_ENTRIES_HEADER);
            for sub_entry in &*self.sub_entries.try_borrow().or(Err(GeneratorError::Unexpected))? {
                let sub_entry = sub_entry.try_borrow().or(Err(GeneratorError::Unexpected))?;
                let sub_entry_string = [sub_entries_header_string.clone(),
                    sub_entry.to_file_string(depth + 1, root_dir)?].join(wispha::LINE_SEPARATOR);
                sub_entry_strings.push(sub_entry_string);
            }
            if sub_entry_strings.len() > 0 {
                return Ok([properties_string, sub_entry_strings.join(wispha::LINE_SEPARATOR)].join(wispha::LINE_SEPARATOR));
            } else {
                return Ok(properties_string)
            }
        }
        Err(GeneratorError::Unexpected)
    }
}