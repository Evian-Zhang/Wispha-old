use std::path::PathBuf;
use std::fmt::Write as FmtWrite;
use std::rc::Rc;

use crate::wispha::{WisphaEntry, WisphaEntryProperties, WisphaFatEntry};
use crate::generator::error::GeneratorError;
use crate::strings::*;

type Result<T> = std::result::Result<T, GeneratorError>;

impl WisphaEntryProperties {
    fn to_string(&self, depth: u32, root_dir: &PathBuf) -> Result<String> {
        let mut begin_mark = String::new();
        let mut counter = 0;
        while counter <= depth {
            write!(&mut begin_mark, "{}", BEGIN_MARK).or(Err(GeneratorError::Unexpected))?;
            counter += 1;
        }

        let mut headers = vec![];

        let absolute_path_header_string = format!("{} [{}]", begin_mark, ABSOLUTE_PATH_HEADER);
        let relative_path = self.absolute_path.clone()
            .strip_prefix(root_dir).or(Err(GeneratorError::Unexpected))?
            .to_path_buf();
        let dir_absolute_path_str = PathBuf::from(ROOT_DIR)
            .join(relative_path)
            .to_str().ok_or(GeneratorError::Unexpected)?
            .to_string();
        let absolute_path_string = format!("{}{}{}{}",
                                           absolute_path_header_string,
                                           LINE_SEPARATOR,
                                           dir_absolute_path_str,
                                           LINE_SEPARATOR);
        headers.push(absolute_path_string);

        let name_header_string = format!("{} [{}]", begin_mark, NAME_HEADER);
        let name_string = format!("{}{}{}{}",
                                  name_header_string,
                                  LINE_SEPARATOR,
                                  &self.name,
                                  LINE_SEPARATOR);
        headers.push(name_string);

        let entry_type_header_string = format!("{} [{}]", begin_mark, ENTRY_TYPE_HEADER);
        let entry_type_string = format!("{}{}{}{}",
                                        entry_type_header_string,
                                        LINE_SEPARATOR,
                                        &self.entry_type.to_str(),
                                        LINE_SEPARATOR);
        headers.push(entry_type_string);

        if let Some(description) = &self.description {
            let description_header_string = format!("{} [{}]", begin_mark, DESCRIPTION_HEADER);
            let description_string = format!("{}{}{}{}",
                                             description_header_string,
                                             LINE_SEPARATOR,
                                             description,
                                             LINE_SEPARATOR);
            headers.push(description_string);
        }

        let mut customized_strings = vec![];
        for (name, value) in &self.customized {
            let customized_header_string = format!("{} [{}]", begin_mark, name);
            let customized_string = format!("{}{}{}{}",
                                            customized_header_string,
                                            LINE_SEPARATOR,
                                            value,
                                            LINE_SEPARATOR);
            customized_strings.push(customized_string);
        }
        let customized_string = customized_strings.join(LINE_SEPARATOR);
        headers.push(customized_string);

        return Ok(headers.join(LINE_SEPARATOR));
    }
}

impl WisphaEntry {
    pub fn to_file_string(&self, depth: u32, root_dir: &PathBuf) -> Result<String> {
        let mut begin_mark = String::new();
        let mut counter = 0;
        while counter <= depth {
            write!(&mut begin_mark, "{}", BEGIN_MARK).or(Err(GeneratorError::Unexpected))?;
            counter += 1;
        }

        let properties_string = self.properties.to_string(depth, root_dir)?;

        let mut sub_entry_strings: Vec<String> = Vec::new();
        let sub_entries_header_string = format!("{} [{}]", begin_mark, SUB_ENTRIES_HEADER);
        for sub_entry in &*self.sub_entries.try_borrow().or(Err(GeneratorError::Unexpected))? {
            let sub_entry_reference_count_pointer = Rc::clone(sub_entry);
            let sub_entry = &*sub_entry_reference_count_pointer.try_borrow().or(Err(GeneratorError::Unexpected))?;
            let sub_entry_content = match sub_entry {
                WisphaFatEntry::Intermediate(entry) => {
                    let entry_file_path_header_string = format!("{}{} [{}]",
                                                                begin_mark,
                                                                BEGIN_MARK,
                                                                ENTRY_FILE_PATH_HEADER);
                    format!("{}{}{}{}",
                            entry_file_path_header_string,
                            LINE_SEPARATOR,
                            entry.entry_file_path.to_str().ok_or(GeneratorError::NameNotValid(entry.entry_file_path.clone()))?,
                            LINE_SEPARATOR)
                }
                WisphaFatEntry::Immediate(entry) => {
                    entry.to_file_string(depth + 1, root_dir)?
                }
            };
            let sub_entry_string = [sub_entries_header_string.clone(), sub_entry_content].join(LINE_SEPARATOR);
            sub_entry_strings.push(sub_entry_string);
        }
        if sub_entry_strings.len() > 0 {
            Ok([properties_string, sub_entry_strings.join(LINE_SEPARATOR)].join(LINE_SEPARATOR))
        } else {
            Ok(properties_string)
        }
    }
}