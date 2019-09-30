use onig::*;
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

pub struct WisphaRawToken {
    pub content: String,
    pub line_number: usize,
    pub file_path: PathBuf,
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

#[derive(Clone, PartialEq)]
pub enum WisphaExpectOption {
    IgnoreDepth,
    AllowLowerDepth,
    IgnoreContent,
}

pub enum WisphaToken {
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

    pub fn raw_token(&self) -> &WisphaRawToken {
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

    fn depth(&self) -> Option<usize> {
        match &self {
            WisphaToken::Header(_, depth) => {
                Some(depth.clone())
            },
            WisphaToken::Body(_) => {
                None
            },
        }
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

impl Clone for WisphaRawProperty {
    fn clone(&self) -> Self {
        WisphaRawProperty {
            header: self.header.clone(),
            body: self.body.clone(),
        }
    }
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
        env::set_var(wispha::ROOT_DIR_VAR, file_path.parent().unwrap().to_str().unwrap());
        self.parse_with_env_set(file_path)
    }

    fn parse_with_env_set(&mut self, file_path: &Path) -> Result<Rc<RefCell<WisphaFatEntry>>> {
        let content = fs::read_to_string(&file_path)
            .or(Err(ParserError::FileCannotRead(file_path.to_path_buf())))?;
        let tokens = self.tokenize(content, file_path);
        let mut root = self.build_wispha_entry_with_relative_path(tokens, 1)?;
        self.resolve(&mut RefCell::new(Rc::clone(&root)))?;
        Ok(root)
    }

    fn tokenize(&mut self, mut content: String, file_path: &Path) -> Vec<Rc<WisphaToken>> {
        let mut tokens = Vec::new();
        content = content.trim().to_string();
        for (line_index, line_content) in content.lines().enumerate() {
            let token = self.parse_line(line_content.to_string(), line_index + 1, file_path);
            tokens.push(Rc::new(token));
        }
        tokens
    }

    // `line_number` starts at 1
    fn parse_line(&self, mut line_content: String, line_number: usize, file_path: &Path) -> WisphaToken {
        let header_pattern = r#"^[ \f\t\v]*(\++)[ \f\t\v]*\[(.+?)][ \f\t\v]*$"#;
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

    fn build_wispha_entry_with_relative_path(&mut self, tokens: Vec<Rc<WisphaToken>>, depth: usize) -> Result<Rc<RefCell<WisphaFatEntry>>> {
        let properties = self.build_wispha_properties(tokens, depth)?;
        self.build_wispha_entry_with_relative_path_from_properties(properties)
    }

    fn build_wispha_properties(&mut self, tokens: Vec<Rc<WisphaToken>>, depth: usize) -> Result<Vec<WisphaRawProperty>> {
        self.expected_tokens = Some(vec![(WisphaToken::default_header_token_with_depth(depth), vec![WisphaExpectOption::IgnoreContent])]);
        let mut properties = Vec::new();
        let mut token_index = 0;
        while let Some(token) = tokens.get(token_index) {
            if !self.is_token_expected(&token) {
                return Err(ParserError::UnexpectedToken(Rc::clone(token), self.expected_tokens.clone()));
            }
            let mut property = WisphaRawProperty {
                header: Rc::clone(token),
                body: vec![]
            };
            token_index += 1;
            while let Some(next_token) = tokens.get(token_index) {
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
            }
            properties.push(property);
        }
        Ok(properties)
    }

    fn build_wispha_entry_with_relative_path_from_properties(&mut self, properties: Vec<WisphaRawProperty>) -> Result<Rc<RefCell<WisphaFatEntry>>> {
        let mut file_path_property = None;
        for property in &properties {
            if property.header.raw_token().content == wispha::ENTRY_FILE_PATH_HEADER.to_string() {
                file_path_property = Some(property.clone());
            }
        }
        if let Some(file_path_property) = file_path_property {
            return self.build_wispha_intermediate_entry(file_path_property);
        } else {
            return self.build_wispha_immediate_entry(properties);
        }
    }

    fn get_content_token_from_body(&mut self, body: Vec<Rc<WisphaToken>>) -> Result<Option<Rc<WisphaToken>>> {
        let mut content_token = None;
        self.expected_tokens = Some(vec![(WisphaToken::empty_body_token(), vec![])]);
        let mut token_index = 0;
        while let Some(token) = body.get(token_index) {
            if !self.is_token_expected(token.borrow()) {
                break;
            }
            token_index += 1;
        }
        if let Some(token) = body.get(token_index) {
            if let WisphaToken::Body(raw_token) = token.borrow() {
                content_token = Some(Rc::clone(token));
                token_index += 1;
            }
        }
        while let Some(token) = body.get(token_index) {
            if !self.is_token_expected(token.borrow()) {
                return Err(ParserError::UnexpectedToken(Rc::clone(token), self.expected_tokens.clone()));
            }
            token_index += 1;
        }
        Ok(content_token)
    }

    fn get_multiline_content_tokens_from_body(&mut self, body: Vec<Rc<WisphaToken>>) -> Result<Vec<Rc<WisphaToken>>> {
        let mut content_tokens = vec![];
        self.expected_tokens = Some(vec![(WisphaToken::empty_body_token(), vec![WisphaExpectOption::IgnoreContent])]);
        let mut token_index = 0;
        while let Some(token) = body.get(token_index) {
            if self.is_token_expected(token.borrow()) {
                content_tokens.push(Rc::clone(token));
            } else {
                return Err(ParserError::UnexpectedToken(Rc::clone(token), self.expected_tokens.clone()));
            }
            token_index += 1;
        }
        Ok(content_tokens)
    }

    fn build_wispha_intermediate_entry(&mut self, file_path_property: WisphaRawProperty) -> Result<Rc<RefCell<WisphaFatEntry>>> {
        if let Some(content_token) = self.get_content_token_from_body(file_path_property.body)? {
            let raw = content_token.raw_token().content.clone();
            let current_dir = content_token.raw_token().file_path.clone().parent().unwrap().to_path_buf();
            Ok(Rc::new(RefCell::new(WisphaFatEntry::Intermediate(WisphaIntermediateEntry{
                entry_file_path: self.actual_path(&raw, &current_dir)?
            }))))
        } else {
            Err(ParserError::EmptyBody(Rc::clone(file_path_property.header.borrow())))
        }
    }

    fn build_wispha_immediate_entry(&mut self, properties: Vec<WisphaRawProperty>) -> Result<Rc<RefCell<WisphaFatEntry>>> {
        let mut immediate_entry = WisphaEntry::default();
        for property in properties {
            match property.header.raw_token().content.as_str() {
                wispha::ABSOLUTE_PATH_HEADER => {;
                    if let Some(content_token) = self.get_content_token_from_body(property.body)? {
                        let raw = content_token.raw_token().content.trim().to_string();
                        let current_dir = content_token.raw_token().file_path.clone().parent().unwrap().to_path_buf();
                        immediate_entry.properties.absolute_path = self.actual_path(&raw, &current_dir)?
                    } else {
                        return Err(ParserError::EmptyBody(Rc::clone(&property.header)));
                    }
                },
                wispha::NAME_HEADER => {
                    if let Some(content_token) = self.get_content_token_from_body(property.body)? {
                        immediate_entry.properties.name = content_token.raw_token().content.trim().to_string();
                    } else {
                        return Err(ParserError::EmptyBody(Rc::clone(&property.header)));
                    }
                },
                wispha::ENTRY_TYPE_HEADER => {
                    if let Some(content_token) = self.get_content_token_from_body(property.body)? {
                        immediate_entry.properties.entry_type = WisphaEntryType::from(content_token.raw_token().content.trim().to_string())
                            .ok_or(ParserError::UnrecognizedEntryFileType(Rc::clone(&content_token)))?;
                    } else {
                        return Err(ParserError::EmptyBody(Rc::clone(&property.header)));
                    }
                },
                wispha::DESCRIPTION_HEADER => {
                    let content_tokens = self.get_multiline_content_tokens_from_body(property.body)?;
                    let mut content = String::new();
                    for token in &content_tokens {
                        content.push_str(&token.raw_token().content);
                        content.push_str("\n");
                    }
                    if content_tokens.len() > 0 {
                        content.pop();
                    }
                    immediate_entry.properties.description = content;
                },
                wispha::SUB_ENTRIES_HEADER => {
                    let sub_entry = self.build_wispha_entry_with_relative_path(property.body, property.header.depth().unwrap() + 1)?;
                    immediate_entry.sub_entries.borrow_mut().push(Rc::clone(&sub_entry));
                },
                _ => {
                    continue;
                }
            }
        }
        Ok(Rc::new(RefCell::new(WisphaFatEntry::Immediate(immediate_entry))))
    }

    fn resolve(&mut self, entry: &RefCell<Rc<RefCell<WisphaFatEntry>>>) -> Result<()> {
        let entry_mut = &mut *entry.borrow_mut();
        let entry_mut_mut = &mut *entry_mut.borrow_mut();
        match entry_mut_mut {
            WisphaFatEntry::Immediate(immediate_entry) => {
                for sub_entry in &mut *immediate_entry.sub_entries.borrow_mut() {
                    self.resolve(&RefCell::new(Rc::clone(sub_entry)));
                    sub_entry.borrow_mut().get_immediate_entry_mut().unwrap().sup_entry = RefCell::new(Rc::downgrade(&entry_mut));
                }
            }

            WisphaFatEntry::Intermediate(intermediate_entry) => {
                *entry.borrow_mut() = self.parse_with_env_set(&intermediate_entry.entry_file_path.clone())?;
            }
        }
        Ok(())
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

    fn actual_path(&self, raw: &String, current_dir: &PathBuf) -> Result<PathBuf> {
        let raw = PathBuf::from(raw);
        if raw.is_absolute() {
            return Ok(raw);
        }

        if raw.starts_with(wispha::ROOT_DIR) {
            let root_dir = PathBuf::from(env::var(wispha::ROOT_DIR_VAR).or(Err(ParserError::EnvNotFound))?);
            let relative_path = raw.strip_prefix(wispha::ROOT_DIR).unwrap().to_path_buf();
            return Ok(root_dir.join(relative_path));
        }

        // `raw` is not absolute and not starts with `ROOT_DIR`
        Ok(current_dir.join(&raw))
    }
}
