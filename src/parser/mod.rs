use onig::*;
use regex; // only use `regex::escape`
use std::fmt::Write as FmtWrite;
use std::io::Write as IoWrite;
use std::path::{Path, PathBuf};
use std::rc::{Rc, Weak};
use std::fs;
use std::env;
use crate::wispha::{self, WisphaEntry, WisphaEntryProperties, WisphaEntryType, WisphaFatEntry, WisphaIntermediateEntry};

pub mod error;
use error::{ParserErrorInfo, ParserError};
use std::cell::RefCell;

type Result<T> = std::result::Result<T, ParserError>;

struct RawWisphaMember {
    header: String,
    body: String,
}

struct WisphaRawToken {
    content: String,
    line_number: usize,
    file_path: PathBuf,
}

enum WisphaToken {
    Header(WisphaRawToken, u8),
    Body(WisphaRawToken),
}

pub struct Parser {
    tokens: Vec<WisphaToken>,
    current_header: Option<String>,
}

impl Parser {
    pub fn parse(&mut self, file_path: &Path) -> Result<Rc<RefCell<WisphaFatEntry>>> {
        let content = fs::read_to_string(&file_path)
            .or(Err(ParserError::FileCannotRead(file_path.to_path_buf())))?;
        self.tokenize(content, file_path);
        Ok(())
    }

    fn tokenize(&mut self, mut content: String, file_path: &Path) {
        content = content.trim().to_string();
        for (line_index, line_content) in content.lines().enumerate() {
            self.parse_line(line_content.to_string(), line_index + 1, file_path);
        }
    }

    // `line_number` starts at 1
    fn parse_line(&mut self, mut line_content: String, line_number: usize, file_path: &Path) {
        let header_pattern = r#"^[ \f\t\v]*(\++)\[(.+?)][ \f\t\v]*$"#;
        let header_regex = Regex::new(header_pattern).unwrap();
        let wispha_token = if let Some(capture) = header_regex.captures(&line_content) {
            WisphaToken::Header(WisphaRawToken {
                content: capture.at(2).unwrap().to_string(),
                line_number,
                file_path: file_path.to_path_buf(),
            }, (u8)(capture.at(1).unwrap()))
        } else {
            WisphaToken::Body(WisphaRawToken {
                content: line_content.clone(),
                line_number,
                file_path: file_path.to_path_buf(),
            })
        };
        self.tokens.push(wispha_token);
    }

    fn build_wispha_entry(&mut self) -> Result<()> {
        for wispha_token in &self.tokens {
            match wispha_token {
                WisphaToken::Header(raw_token) => {
                    self.current_header = Some(raw_token.content.clone());
                },
                WisphaToken::Body(raw_token) => {
                    if let Some(header) = self.current_header {

                    } else {
                        return Err(ParserError::LackHeader(raw_token.file_path.clone(), raw_token.line_number));
                    }
                },
            }
        }
        Ok(())
    }
}

// `file_path`: absolute root path
pub fn parse(file_path: &PathBuf) -> Result<Rc<RefCell<WisphaFatEntry>>> {
    let content = fs::read_to_string(&file_path)
        .or(Err(ParserError::FileCannotRead(file_path.clone())))?;
    let root_dir = file_path.parent().ok_or(ParserError::DirectoryNotDetermined(file_path.clone()))?.to_path_buf();
    env::set_var(wispha::ROOT_DIR_VAR, &root_dir.to_str().unwrap());
    let root = parse_with_depth(&content, 0, &root_dir, &file_path)?;
    Ok(root)
}

fn line_number_in_content(content: &String, pos: usize) -> usize {
    let slice = content.get(..pos).unwrap();
    slice.lines().count()
}

fn parse_with_depth(content: &String, depth: u32, dir: &PathBuf, file_path: &PathBuf) -> Result<Rc<RefCell<WisphaFatEntry>>> {
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
        let actual_path = actual_path(&intermediate_entry.entry_file_path, Some(&dir), &file_path, None)?;
        let content = fs::read_to_string(&actual_path).or(Err(ParserError::FileCannotRead(
            actual_path.clone(),
        )))?;
        return parse_with_depth(&content,
                                0,
                                &actual_path.parent().ok_or(ParserError::Unexpected)?
                                    .to_path_buf(),
                                &actual_path);
    }

    let wispha_entry = Rc::new(RefCell::new(WisphaFatEntry::Immediate(WisphaEntry::default())));
    wispha_entry.try_borrow_mut().or(Err(ParserError::Unexpected))?
        .get_immediate_entry_mut().ok_or(ParserError::Unexpected)?
        .properties.file_path = file_path.clone();
    for raw_wispha_member in &raw_wispha_members {
        let body = raw_wispha_member.body.clone();
        match raw_wispha_member.header.as_str() {
            wispha::ABSOLUTE_PATH_HEADER => {
                if body.is_empty() {
                    return Err(ParserError::AbsolutePathEmpty(
                        ParserErrorInfo {
                            path: file_path.clone(),
                            property: Some(wispha::ABSOLUTE_PATH_HEADER.to_string())
                        }
                    ));
                }
                wispha_entry.try_borrow_mut().or(Err(ParserError::Unexpected))?
                    .get_immediate_entry_mut().ok_or(ParserError::Unexpected)?
                    .properties.absolute_path = actual_path(&PathBuf::from(&body),
                                                            Some(dir),
                                                            file_path,
                                                            Some(wispha::ABSOLUTE_PATH_HEADER.to_string()))?;
            },
            wispha::NAME_HEADER => {
                if body.is_empty() {
                    return Err(ParserError::NameEmpty(
                        ParserErrorInfo {
                            path: file_path.clone(),
                            property: Some(wispha::NAME_HEADER.to_string())
                        }
                    ));
                }
                wispha_entry.try_borrow_mut().or(Err(ParserError::Unexpected))?
                    .get_immediate_entry_mut().ok_or(ParserError::Unexpected)?
                    .properties.name = body
            },
            wispha::ENTRY_TYPE_HEADER => {
                wispha_entry.try_borrow_mut().or(Err(ParserError::Unexpected))?
                    .get_immediate_entry_mut().ok_or(ParserError::Unexpected)?
                    .properties.entry_type = WisphaEntryType::from(body.clone())
                    .ok_or(ParserError::UnrecognizedEntryFileType(
                        ParserErrorInfo {
                            path: file_path.clone(),
                            property: Some(wispha::ENTRY_TYPE_HEADER.to_string())
                        },
                        body.clone(),
                    ))?;
            },
            wispha::DESCRIPTION_HEADER => {
                wispha_entry.try_borrow_mut().or(Err(ParserError::Unexpected))?
                    .get_immediate_entry_mut().ok_or(ParserError::Unexpected)?
                    .properties.description = body
            },
            wispha::SUB_ENTRIES_HEADER => {
                let mut sub_entry = RefCell::new(parse_with_depth(&body, depth + 1, &dir, &file_path)?);
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

fn begin_mark(depth: u32) -> String {
    let mut counter = 0;
    let mut begin_mark = String::new();
    while counter <= depth {
        write!(&mut begin_mark, "{}", wispha::BEGIN_MARK).unwrap();
        counter += 1;
    }

    begin_mark
}

fn prepare_regex_pattern(depth: u32) -> Regex {
    let begin_mark = begin_mark(depth);
    let begin_mark_regex = regex::escape(begin_mark.as_str());

    let prefix = r#"^[ \f\t\v]*"#;
    let postfix = r#"[ \f\t\v]*\[([^\r\n]*)]$"#;

    let header = format!("{}{}{}", prefix, begin_mark_regex, postfix);

    let body = r#"([\s\S]*?)"#;

    let pattern = format!("{}{}(?={}|\\z)", header, body, header);

    let regex_pattern = Regex::with_options(pattern.as_str(),
                                            RegexOptions::REGEX_OPTION_MULTILINE,
                                            Syntax::default()).unwrap();

    regex_pattern
}

fn get_raw_wispha_members(content: &String, depth: u32) -> Result<Vec<RawWisphaMember>> {
    let regex_pattern = prepare_regex_pattern(depth);

    let mut raw_wispha_members: Vec<RawWisphaMember> = Vec::new();

    for caps in regex_pattern.captures_iter(content.as_str()) {
        let header = caps.at(1).ok_or(ParserError::Unexpected)?.to_string();
        let raw_body = caps.at(2).ok_or(ParserError::Unexpected)?.to_string();

        let body_pattern = r#"\A\s*(\S[\s\S]*\S)\s*\z"#;
        let body_regex_pattern = Regex::with_options(body_pattern,
                                                RegexOptions::REGEX_OPTION_MULTILINE,
                                                Syntax::default())
            .or(Err(ParserError::Unexpected))?;
        let body = match body_regex_pattern.captures(raw_body.as_str()) {
            Some(cap) => {
                cap.at(1).unwrap_or("")
            },
            None => {
                ""
            },
        }.to_string();
        let raw_wispha_member = RawWisphaMember { header, body };
        raw_wispha_members.push(raw_wispha_member);
    }

    Ok(raw_wispha_members)
}

fn actual_path(raw: &PathBuf, current_dir: Option<&PathBuf>, file_path: &PathBuf, property: Option<String>) -> Result<PathBuf> {
    if raw.is_absolute() {
        return Ok(raw.clone());
    }

    if raw.starts_with(wispha::ROOT_DIR) {
        let root_dir = PathBuf::from(env::var(wispha::ROOT_DIR_VAR).or(Err(ParserError::InvalidPath(
            ParserErrorInfo {
                path: file_path.clone(),
                property,
            },
            raw.clone(),
        )))?);
        let relative_path = raw.strip_prefix(wispha::ROOT_DIR).or(Err(ParserError::Unexpected))?.to_path_buf();
        return Ok(root_dir.join(relative_path));
    }

    if let Some(current_dir) = current_dir {
        return Ok(current_dir.join(&raw));
    }

    Err(ParserError::InvalidPath(
        ParserErrorInfo {
            path: file_path.clone(),
            property,
        },
        raw.clone(),
    ))
}
