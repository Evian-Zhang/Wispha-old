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
use std::string::ParseError;
use std::borrow::Borrow;

type Result<T> = std::result::Result<T, ParserError>;

struct WisphaRawEntry {
    header: String,
    body: String,
}

struct WisphaRawToken {
    content: String,
    line_number: usize,
    file_path: PathBuf,
}

impl Clone for WisphaRawToken {
    fn clone(&self) -> Self {
        WisphaRawToken {
            content: self.content.clone(),
            line_number: self.line_number.clone(),
            file_path: self.file_path.clone(),
        }
    }
}

#[derive(Clone)]
enum WisphaExpectOption {
    IgnoreDepth,
    AllowLowerDepth,
    IgnoreContent,
}

enum WisphaToken {
    Header(WisphaRawToken, usize),
    Body(WisphaRawToken),
}

impl WisphaToken {
    fn matches(&self, token: &WisphaToken, options: Vec<WisphaExpectOption>) -> bool {
        use WisphaToken::*;
        match (&self, token) {
            (Header(self_raw_token, self_depth), Header(raw_token, depth)) => {
                if !options.contains(&WisphaExpectOption::IgnoreDepth) {
                    if options.contains(&WisphaExpectOption::AllowLowerDepth) && self_depth <= depth {
                        return true;
                    } else if self_depth == depth {
                        return true;
                    }
                    return false;
                }
                if !options.contains(&WisphaExpectOption::IgnoreContent) && self_raw_token.content != raw_token.content {
                    return false;
                }
                return true;
            },
            (Body(self_raw_token), Body(raw_token)) => {
                if !options.contains(&WisphaExpectOption::IgnoreContent) && self_raw_token.content != raw_token.content {
                    return false;
                }
                return true;
            },
            _ => {
                return false;
            }
        }
    }

    fn raw_token(&self) -> &WisphaRawToken {
        match &self {
            WisphaToken::Header(raw_token, _) => {
                raw_token
            },
            WisphaToken::Body(raw_token) => {
                raw_token
            },
        }
    }

    fn line_number(&self) -> usize {
        self.raw_token().line_number.clone()
    }

    fn default_header_token_with_depth(depth: usize) -> WisphaToken {
        WisphaToken::Header(WisphaRawToken {
            content: "".to_string(),
            line_number: 0,
            file_path: PathBuf::new(),
        }, depth)
    }

    fn default_header_token_with_content(content: String) -> WisphaToken {
        WisphaToken::Header(WisphaRawToken {
            content,
            line_number: 0,
            file_path: PathBuf::new(),
        }, 1)
    }

    fn default_header_token_with_content_and_depth(content: String, depth: usize) -> WisphaToken {
        WisphaToken::Header(WisphaRawToken {
            content,
            line_number: 0,
            file_path: PathBuf::new(),
        }, depth)
    }

    fn empty_body_token() -> WisphaToken {
        WisphaToken::Body(WisphaRawToken {
            content: "".to_string(),
            line_number: 0,
            file_path: PathBuf::new(),
        })
    }
}

impl Clone for WisphaToken {
    fn clone(&self) -> Self {
        match &self {
            WisphaToken::Header(raw_token, depth) => {
                WisphaToken::Header(raw_token.clone(), depth.clone())
            },
            WisphaToken::Body(raw_token) => {
                WisphaToken::Body(raw_token.clone())
            },
        }
    }
}

impl PartialEq for WisphaToken {
    fn eq(&self, other: &Self) -> bool {
        use WisphaToken::*;
        match (&self, other) {
            (Header(_, _), Header(_, _)) => {
                return true;
            },
            (Body(self_raw_token), Body(raw_token)) => {
                return self_raw_token.content == raw_token.content;
            },
            _ => {
                return false;
            }
        }
    }
}

impl Eq for WisphaToken { }

struct WisphaRawProperty {
    header: Rc<WisphaToken>,
    body: Vec<Rc<WisphaToken>>,
}

pub struct Parser {
    expected_tokens: Option<Vec<(WisphaToken, Vec<WisphaExpectOption>)>>,
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            expected_tokens: Some(vec![(WisphaToken::default_header_token_with_depth(1), vec![WisphaExpectOption::IgnoreContent])]),
        }
    }

    pub fn parse(&mut self, file_path: &Path) -> Result<Rc<RefCell<WisphaFatEntry>>> {
        let content = fs::read_to_string(&file_path)
            .or(Err(ParserError::FileCannotRead(file_path.to_path_buf())))?;
        let tokens = self.tokenize(content, file_path);
        Ok(())
    }

    fn tokenize(&mut self, mut content: String, file_path: &Path) -> Vec<WisphaToken> {
        let mut tokens = Vec::new();
        content = content.trim().to_string();
        for (line_index, line_content) in content.lines().enumerate() {
            let token = self.parse_line(line_content.to_string(), line_index + 1, file_path);
            tokens.push(token);
        }
        tokens
    }

    // `line_number` starts at 1
    fn parse_line(&self, mut line_content: String, line_number: usize, file_path: &Path) -> WisphaToken {
        let header_pattern = r#"^[ \f\t\v]*(\++)\[(.+?)][ \f\t\v]*$"#;
        let header_regex = Regex::new(header_pattern).unwrap();
        let wispha_token = if let Some(capture) = header_regex.captures(&line_content) {
            let content = capture.at(2).unwrap().to_string();
            let pluses = capture.at(1).unwrap();
            let depth = pluses.len();
            WisphaToken::Header(WisphaRawToken {
                content,
                line_number,
                file_path: file_path.to_path_buf(),
            }, depth)
        } else {
            WisphaToken::Body(WisphaRawToken {
                content: line_content.clone(),
                line_number,
                file_path: file_path.to_path_buf(),
            })
        };
        wispha_token
    }

    fn build_wispha_properties(&mut self, tokens: Vec<Rc<WisphaToken>>, depth: usize) -> Result<Vec<WisphaRawProperty>> {
        self.expected_tokens = Some(vec![(WisphaToken::default_header_token_with_depth(depth), vec![WisphaExpectOption::IgnoreContent])]);
        let mut properties = Vec::new();
        let mut token_index = 0;
        while let Some(token) = tokens.get(token_index) {
            if !self.is_token_expected(&token) {
                return Err(ParserError::UnexpectedToken(token.borrow().clone(), self.expected_tokens.clone(), token.line_number()));
            }
            let mut property = WisphaRawProperty {
                header: Rc::clone(token),
                body: vec![]
            };
            token_index += 1;
            loop {
                if let Some(next_token) = tokens.get(token_index) {
                    match next_token.borrow() {
                        WisphaToken::Header(_, token_depth) => {
                            if token_depth.clone() <= depth {
                                break;
                            }
                        },
                        WisphaToken::Body(_) => { },
                    }
                    property.body.push(Rc::clone(next_token));
                    token_index += 1;
                } else {
                    break;
                }
            }
        }
        Ok(properties)
    }

    fn build_wispha_entry_with_relative_path(&mut self, properties: Vec<WisphaRawProperty>) -> Result<Rc<RefCell<WisphaFatEntry>>> {
        let mut file_path_property_body = None;
        for property in properties {
            if property.header.borrow().raw_token().content == wispha::ENTRY_FILE_PATH_HEADER.to_string() {
                file_path_property_body = Some(property.body);
            }
        }
        if let Some(file_path_property_body) = file_path_property_body {
            return self.build_wispha_intermediate_entry_with_relative_path(file_path_property_body);
        } else {
            return self.build_wispha_immediate_entry_with_relative_path(properties)
        }
        Ok(())
    }

    fn build_wispha_intermediate_entry_with_relative_path(&mut self, file_path_body: Vec<Rc<WisphaToken>>) -> Result<Rc<RefCell<WisphaFatEntry>>> {
        let mut intermediate_entry = WisphaIntermediateEntry {
            entry_file_path: PathBuf::new(),
        };
        self.expected_tokens = vec![(WisphaToken::empty_body_token(), vec![])];
        let mut token_index = 0;
        while let Some(token) = file_path_body.get(token_index) {
            if !self.is_token_expected(token.borrow()) {
                break;
            }
            token_index += 1;
        }
        if let Some(token) = file_path_body.get(token_index) {
            if let WisphaToken::Body(raw_token) = token.borrow() {
                intermediate_entry.entry_file_path = PathBuf::from(raw_token.content.clone());
                token_index += 1;
            }
        }
        while let Some(token) = file_path_body.get(token_index) {
            if !self.is_token_expected(token.borrow()) {
                return Err(ParserError::UnexpectedToken(token.borrow().clone(), self.expected_tokens.clone(), token.line_number()));
            }
            token_index += 1;
        }
        Ok(Rc::new(RefCell::new(WisphaFatEntry::Intermediate(intermediate_entry))))
    }

    fn build_wispha_immediate_entry_with_relative_path(&mut self, properties: Vec<WisphaRawProperty>) -> Result<Rc<RefCell<WisphaFatEntry>>> {
        let mut immediate_entry = WisphaEntry::default();
        self.expected_tokens = vec![(WisphaToken::empty_body_token(), vec![])];
        let mut token_index = 0;
        while let Some(token) = properties.get(token_index) {
            if !self.is_token_expected(token.borrow()) {
                break;
            }
            token_index += 1;
        }
        Ok(Rc::new(RefCell::new(WisphaFatEntry::Immediate(immediate_entry))))
    }

    fn is_token_expected(&self, token: &WisphaToken) -> bool {
        if let Some(expected_tokens) = &self.expected_tokens {
            for (expected_token, options) in expected_tokens {
                if token.matches(expected_token, options.clone()) {
                    return true;
                }
            }
            return false;
        } else {
            return true;
        }
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

fn get_raw_wispha_members(content: &String, depth: u32) -> Result<Vec<WisphaRawEntry>> {
    let regex_pattern = prepare_regex_pattern(depth);

    let mut raw_wispha_members: Vec<WisphaRawEntry> = Vec::new();

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
        let raw_wispha_member = WisphaRawEntry { header, body };
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
