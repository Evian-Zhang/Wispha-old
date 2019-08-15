use onig::*;
use regex;
use std::fmt::Write as FmtWrite;
use std::io::Write as IoWrite;
use std::path::{Path, PathBuf};
use std::rc::{Rc, Weak};
use std::fs;
use std::env;
use crate::wispha::{self, WisphaEntry, WisphaEntryProperties, WisphaEntryType, WisphaSubentry, WisphaIntermediateEntry};

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

pub fn parse_with_depth(content: String, depth: u32, dir: &PathBuf) -> Result<Rc<RefCell<WisphaSubentry>>> {
    let raw_wispha_members = get_raw_wispha_members(&content, depth)?;

    let mut intermediate_entry: Option<WisphaIntermediateEntry> = None;
    for raw_wispha_member in &raw_wispha_members {
        if raw_wispha_member.header.as_str() == wispha::ENTRY_FILE_PATH_HEADER {
            intermediate_entry = Some(WisphaIntermediateEntry {
                entry_file_path: PathBuf::from(raw_wispha_member.body.clone()),
            });
        }
    }

    if let Some(intermediate_entry) = intermediate_entry {
        let actual_path = actual_path(&intermediate_entry.entry_file_path, Some(&dir))?;
        let content = fs::read_to_string(&actual_path).or(Err(ParserError::FileCannotRead))?;
        return parse_with_depth(content,
                                0,
                                &actual_path.parent().ok_or(ParserError::Unexpected)?
                                    .to_path_buf());
    }

    let wispha_entry = Rc::new(RefCell::new(WisphaSubentry::Immediate(WisphaEntry::default())));
    for raw_wispha_member in &raw_wispha_members {
        let body = raw_wispha_member.body.clone();
        match raw_wispha_member.header.as_str() {
            wispha::ABSOLUTE_PATH_HEADER => {
                if body.is_empty() {
                    return Err(ParserError::AbsolutePathEmpty);
                }
                wispha_entry.try_borrow_mut().or(Err(ParserError::Unexpected))?
                    .get_immediate_entry_mut().ok_or(ParserError::Unexpected)?
                    .properties.absolute_path = PathBuf::from(body);
            },
            wispha::NAME_HEADER => {
                if body.is_empty() {
                    return Err(ParserError::NameEmpty);
                }
                wispha_entry.try_borrow_mut().or(Err(ParserError::Unexpected))?
                    .get_immediate_entry_mut().ok_or(ParserError::Unexpected)?
                    .properties.name = body
            },
            wispha::ENTRY_TYPE_HEADER => {
                wispha_entry.try_borrow_mut().or(Err(ParserError::Unexpected))?
                    .get_immediate_entry_mut().ok_or(ParserError::Unexpected)?
                    .properties.entry_type = WisphaEntryType::from(body)
                    .ok_or(ParserError::UnrecognizedEntryFileType)?;
            },
            wispha::DESCRIPTION_HEADER => {
                wispha_entry.try_borrow_mut().or(Err(ParserError::Unexpected))?
                    .get_immediate_entry_mut().ok_or(ParserError::Unexpected)?
                    .properties.description = body
            },
            wispha::SUB_ENTRIES_HEADER => {
                let mut sub_entry = RefCell::new(parse_with_depth(body, depth + 1, &dir)?);
                sub_entry.try_borrow_mut().or(Err(ParserError::Unexpected))?
                    .try_borrow_mut().or(Err(ParserError::Unexpected))?
                    .get_immediate_entry_mut().ok_or(ParserError::Unexpected)?
                    .sup_entry = RefCell::new(Rc::downgrade(&wispha_entry));
                wispha_entry.try_borrow_mut().or(Err(ParserError::Unexpected))?
                    .get_immediate_entry_mut().ok_or(ParserError::Unexpected)?
                    .sub_entries.try_borrow_mut().or(Err(ParserError::Unexpected))?
                    .push(sub_entry.into_inner());
            },
            _ => continue,
        }
    }

    Ok(wispha_entry)
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

fn get_raw_wispha_members(content: &String, depth: u32) -> Result<Vec<RawWisphaMember>> {
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

    Ok(raw_wispha_members)
}

fn actual_path(raw: &PathBuf, current_dir: Option<&PathBuf>) -> Result<PathBuf> {
    if raw.is_absolute() {
        return Ok(raw.clone());
    }

    if raw.starts_with(wispha::ROOT_DIR) {
        let root_dir = PathBuf::from(env::var(wispha::ROOT_DIR_VAR).or(Err(ParserError::InvalidPath))?);
        return Ok(root_dir.join(&raw));
    }

    if let Some(current_dir) = current_dir {
        return Ok(current_dir.join(&raw));
    }

    Err(ParserError::InvalidPath)
}
