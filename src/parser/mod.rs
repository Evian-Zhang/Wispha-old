use onig::*;
use regex;
use std::fmt::Write as FmtWrite;
use std::io::Write as IoWrite;
use std::path::{Path, PathBuf};
use std::rc::{Rc, Weak};
use crate::wispha::{self, WisphaEntry, WisphaEntryProperties, WisphaEntryType};

mod error;
use error::ParserError;
use std::cell::RefCell;

type Result<T> = std::result::Result<T, ParserError>;

struct RawWisphaMember {
    header: String,
    body: String,
}

pub fn parse(raw_content: String) {

}

pub fn parse_with_depth(content: String, depth: u32) -> Result<Rc<WisphaEntry>> {
    let regex_pattern = prepare_regex_pattern(depth)?;

    let mut raw_wispha_members: Vec<RawWisphaMember> = Vec::new();

    for caps in regex_pattern.captures_iter(content.as_str()) {
        let header = caps.at(1).ok_or(ParserError::Unexpected)?.to_string();
        let body = caps.at(2).ok_or(ParserError::Unexpected)?
            .trim_start_matches('\n')
            .trim_end_matches('\n')
            .to_string();
        let raw_wispha_member = RawWisphaMember { header, body };
        raw_wispha_members.push(raw_wispha_member);
    }

    let mut has_entry_file_path = false;

    for raw_wispha_member in &raw_wispha_members {
        if raw_wispha_member.header.as_str() == wispha::ENTRY_FILE_PATH_HEADER {
            has_entry_file_path = true;
            break;
        }
    }

    let wispha_entry = RefCell::new(Rc::new(WisphaEntry::default()));

    for raw_wispha_member in &raw_wispha_members {
        let body = raw_wispha_member.body.clone();
        match raw_wispha_member.header.as_str() {
            wispha::ABSOLUTE_PATH_HEADER => {
                if body.is_empty() {
                    return Err(ParserError::AbsolutePathEmpty);
                }
                wispha_entry.try_borrow_mut().or(Err(ParserError::Unexpected))?
                    .properties.absolute_path = PathBuf::from(body);
            },
            wispha::NAME_HEADER => {
                if body.is_empty() {
                    return Err(ParserError::NameEmpty);
                }
                wispha_entry.try_borrow_mut().or(Err(ParserError::Unexpected))?
                    .properties.name = body
            },
            wispha::ENTRY_TYPE_HEADER => {
                wispha_entry.try_borrow_mut().or(Err(ParserError::Unexpected))?
                    .properties.entry_type = WisphaEntryType::from(body)
                    .ok_or(ParserError::UnrecognizedEntryFileType)?;
            },
            wispha::DESCRIPTION_HEADER => {
                wispha_entry.try_borrow_mut().or(Err(ParserError::Unexpected))?
                    .properties.description = body
            },
            wispha::ENTRY_FILE_PATH_HEADER => {
                if body.is_empty() {
                    return Err(ParserError::EntryFileTypeEmpty);
                }
                wispha_entry.try_borrow_mut().or(Err(ParserError::Unexpected))?
                    .entry_file_path = Some(PathBuf::from(body))
            },
            wispha::SUB_ENTRIES_HEADER if !has_entry_file_path => {
                let mut sub_entry = RefCell::new(parse_with_depth(body, depth + 1)?);
                sub_entry.try_borrow_mut().or(Err(ParserError::Unexpected))?
                    .sup_entry = RefCell::new(Rc::clone(
                    wispha_entry.try_borrow().or(Err(ParserError::Unexpected))?
                ));
                wispha_entry.try_borrow_mut().or(Err(ParserError::Unexpected))?
                    .sub_entries.try_borrow_mut().or(Err(ParserError::Unexpected))?
                   .push(sub_entry);
            },
            _ => continue,
        }
    }

    Ok(wispha_entry.into_inner())
}

fn begin_mark(depth: u32) -> Result<String> {
    let mut counter = 0;
    let mut begin_mark = String::new();
    while counter <= depth {
        write!(&mut begin_mark, "{}", wispha::BEGIN_MARK).or(Err(ParserError::Unexpected))?;
        counter += 1;
    }

    Ok(begin_mark)
}

fn prepare_regex_pattern(depth: u32) -> Result<Regex> {
    let begin_mark = begin_mark(depth)?;
    let begin_mark_regex = regex::escape(begin_mark.as_str());

    let prefix = r#"^[ \f\t\v]*"#;
    let postfix = r#"[ \f\t\v]*\[([^\r\n]*)]$"#;

    let header = format!("{}{}{}", prefix, begin_mark_regex, postfix);

    let body = r#"([\s\S]*?)"#;

    let pattern = format!("{}{}(?={}|\\z)", header, body, header);

    let regex_pattern = Regex::with_options(pattern.as_str(),
                                            RegexOptions::REGEX_OPTION_MULTILINE,
                                            Syntax::default())
        .or(Err(ParserError::Unexpected))?;

    Ok(regex_pattern)
}
