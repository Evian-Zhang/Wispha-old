use onig::*;

use std::path::{Path, PathBuf};
use std::rc::{Rc, Weak};
use std::{fs, sync};
use std::env;
use std::cell::RefCell;
use std::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;

//use crate::wispha::{WisphaEntry, WisphaEntryType, WisphaFatEntry, WisphaIntermediateEntry};
use crate::strings::*;
use crate::wispha::{common::*, intermediate::*, core::*};

mod parser_struct;
use parser_struct::*;

pub mod option;
use option::*;

pub mod error;
use error::ParserError;
use std::sync::{Arc, Mutex, mpsc, mpsc::Sender};
use std::thread;

type Result<T> = std::result::Result<T, ParserError>;

pub struct Parser {
    expected_tokens: Option<Vec<(WisphaToken, Vec<WisphaExpectOption>)>>,
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            expected_tokens: Some(vec![(WisphaToken::default_header_token_with_depth(1), vec![WisphaExpectOption::IgnoreContent])]),
        }
    }

    pub fn parse(&'static mut self, file_path: &Path, options: &'static ParserOptions) -> Result<Rc<RefCell<WisphaEntry>>> {
        env::set_var(ROOT_DIR_VAR, file_path.parent().unwrap().to_str().unwrap());
        let intermediate_entry = self.parse_with_env_set(file_path.to_path_buf(), &options, None)?;
        let locked_entry = intermediate_entry.lock().unwrap();
        if let Some(common) = locked_entry.to_common() {
            Ok(common)
        } else {
            Err(ParserError::Unexpected)
        }
    }

    fn parse_with_env_set(&'static mut self, file_path: PathBuf, options: &'static ParserOptions, tx_global: Option<Sender<bool>>) -> Result<Arc<Mutex<WisphaIntermediateEntry>>> {
        let (tx_global, rx_global_option) = if let Some(tx_global) = tx_global {
            (tx_global, None)
        } else {
            let (tx_global, rx_global) = mpsc::channel();
            (tx_global, Some(rx_global))
        };
        let content = fs::read_to_string(&file_path)
            .or(Err(ParserError::FileCannotRead(file_path.clone())))?;
        let tokens = self.tokenize(content, &file_path);
        let root = self.build_wispha_entry_with_relative_path(tokens, 1, options)?;
        self.resolve(Arc::clone(&root), options, Sender::clone(&tx_global))?;
        tx_global.send(true).or(Err(ParserError::Unexpected))?;
        drop(tx_global);
        if let Some(rx_global) = rx_global_option {
            for _ in rx_global { }
        }
        Ok(root)
    }

    fn tokenize(&mut self, content: String, file_path: &Path) -> Vec<Rc<WisphaToken>> {
        let mut tokens = Vec::new();
        for (line_index, line_content) in content.lines().enumerate() {
            let token = self.parse_line(line_content.to_string(), line_index + 1, file_path);
            tokens.push(Rc::new(token));
        }
        tokens
    }

    // `line_number` starts at 1
    fn parse_line(&self, line_content: String, line_number: usize, file_path: &Path) -> WisphaToken {
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

    fn build_wispha_entry_with_relative_path(&mut self, tokens: Vec<Rc<WisphaToken>>, depth: usize, options: &ParserOptions) -> Result<Arc<Mutex<WisphaIntermediateEntry>>> {
        let properties = self.build_wispha_properties(tokens, depth)?;
        self.build_wispha_entry_with_relative_path_from_properties(properties, options)
    }

    fn build_wispha_properties(&mut self, tokens: Vec<Rc<WisphaToken>>, depth: usize) -> Result<Vec<WisphaRawProperty>> {
        self.expected_tokens = Some(vec![(WisphaToken::default_header_token_with_depth(depth), vec![WisphaExpectOption::IgnoreContent]), (WisphaToken::empty_body_token(), vec![])]);
        let mut properties = Vec::new();
        let mut token_index = 0;
        while let Some(token) = tokens.get(token_index) {
            if !self.is_token_expected(&token) {
                let token: &WisphaToken = token.borrow();
                return Err(ParserError::UnexpectedToken(token.clone(), self.expected_tokens.clone()));
            }
            let mut property = WisphaRawProperty {
                header: Rc::clone(token),
                body: vec![]
            };
            token_index += 1;
            while let Some(next_token) = tokens.get(token_index) {
                match (*next_token).borrow() {
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

    fn build_wispha_entry_with_relative_path_from_properties(&mut self, properties: Vec<WisphaRawProperty>, options: &ParserOptions) -> Result<Arc<Mutex<WisphaIntermediateEntry>>> {
        let mut file_path_property = None;
        for property in &properties {
            if property.header.raw_token().content == ENTRY_FILE_PATH_HEADER.to_string() {
                file_path_property = Some(property.clone());
            }
        }
        if let Some(file_path_property) = file_path_property {
            return self.build_wispha_link_entry(file_path_property);
        } else {
            return self.build_wispha_direct_entry(properties, options);
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
            if let WisphaToken::Body(_) = token.borrow() {
                content_token = Some(Rc::clone(token));
                token_index += 1;
            }
        }
        while let Some(token) = body.get(token_index) {
            if !self.is_token_expected(token.borrow()) {
                let token: &WisphaToken = token.borrow();
                return Err(ParserError::UnexpectedToken(token.clone(), self.expected_tokens.clone()));
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
                let token: &WisphaToken = token.borrow();
                return Err(ParserError::UnexpectedToken(token.clone(), self.expected_tokens.clone()));
            }
            token_index += 1;
        }
        Ok(content_tokens)
    }

    fn build_wispha_link_entry(&mut self, file_path_property: WisphaRawProperty) -> Result<Arc<Mutex<WisphaIntermediateEntry>>> {
        if let Some(content_token) = self.get_content_token_from_body(file_path_property.body)? {
            let raw = content_token.raw_token().content.clone();
            let current_dir = content_token.raw_token().file_path.clone().parent().unwrap().to_path_buf();
            Ok(Arc::new(Mutex::new(WisphaIntermediateEntry::Link(WisphaLinkEntry {
                entry_file_path: self.actual_path(&raw, &current_dir)?
            }))))
        } else {
            let token: &WisphaToken = file_path_property.header.borrow();
            Err(ParserError::EmptyBody(token.clone()))
        }
    }

    fn build_wispha_direct_entry(&mut self, properties: Vec<WisphaRawProperty>, options: &ParserOptions) -> Result<Arc<Mutex<WisphaIntermediateEntry>>> {
        let mut direct_entry = WisphaDirectEntry::default();
        for property in properties {
            direct_entry.properties.file_path = property.header.raw_token().file_path.clone();
            let header_str = property.header.raw_token().content.as_str();
            match header_str {
                ABSOLUTE_PATH_HEADER => {
                    if let Some(content_token) = self.get_content_token_from_body(property.body)? {
                        let raw = content_token.raw_token().content.trim().to_string();
                        let current_dir = content_token.raw_token().file_path.clone().parent().unwrap().to_path_buf();
                        direct_entry.properties.absolute_path = self.actual_path(&raw, &current_dir)?
                    } else {
                        let token: &WisphaToken = property.header.borrow();
                        return Err(ParserError::EmptyBody(token.clone()));
                    }
                },
                NAME_HEADER => {
                    if let Some(content_token) = self.get_content_token_from_body(property.body)? {
                        direct_entry.properties.name = content_token.raw_token().content.trim().to_string();
                    } else {
                        let token: &WisphaToken = property.header.borrow();
                        return Err(ParserError::EmptyBody(token.clone()));
                    }
                },
                ENTRY_TYPE_HEADER => {
                    if let Some(content_token) = self.get_content_token_from_body(property.body)? {
                        let token: &WisphaToken = content_token.borrow();
                        direct_entry.properties.entry_type = WisphaEntryType::from(content_token.raw_token().content.trim().to_string())
                            .ok_or(ParserError::UnrecognizedEntryFileType(token.clone()))?;
                    } else {
                        let token: &WisphaToken = property.header.borrow();
                        return Err(ParserError::EmptyBody(token.clone()));
                    }
                },
                DESCRIPTION_HEADER => {
                    let content_tokens = self.get_multiline_content_tokens_from_body(property.body)?;
                    let mut content = String::new();
                    for token in &content_tokens {
                        content.push_str(&token.raw_token().content);
                        content.push_str("\n");
                    }
                    if content_tokens.len() > 0 {
                        content.pop();
                    }
                    direct_entry.properties.description = Some(content);
                },
                SUB_ENTRIES_HEADER => {
                    let sub_entry = self.build_wispha_entry_with_relative_path(property.body, property.header.depth().unwrap() + 1, options)?;
                    let mut locked_sub_entries = direct_entry.sub_entries.lock().unwrap();
                    locked_sub_entries.push(Arc::clone(&sub_entry));
                    drop(locked_sub_entries);
                },
                _ => {
                    let properties = &options.properties;
                    for property in properties {
                        if property.name.as_str() == header_str {
                            direct_entry.properties.customized.insert(property.name.clone(), header_str.to_string());
                            break;
                        }
                    }
                    continue;
                }
            }
        }
        Ok(Arc::new(Mutex::new(WisphaIntermediateEntry::Direct(direct_entry))))
    }

//    fn resolve(&mut self, entry: Arc<Mutex<WisphaIntermediateEntry>>, options: &ParserOptions, tx_global: Sender<bool>) -> Result<Arc<Mutex<WisphaIntermediateEntry>>> {
//        let locked_entry = entry.lock().unwrap();
//        match &mut *locked_entry {
//            WisphaIntermediateEntry::Direct(direct_entry) => {
//                direct_entry.sup_entry = Mutex::new(sync::Weak::new());
//                let locked_sub_entries = direct_entry.sub_entries.lock().unwrap();
//                for sub_entry in &mut *locked_sub_entries {
//                    *sub_entry = self.resolve(Arc::clone(sub_entry), options, tx_global)?;
//                    let mut locked_sub_entry = sub_entry.lock().unwrap();
//                    locked_sub_entry.get_direct_entry_mut().unwrap().sup_entry = Mutex::new(Arc::downgrade(&entry));
//                    drop(locked_sub_entry);
//                }
//                drop(locked_sub_entries);
//                Ok(Arc::clone(&entry))
//            }
//
//            WisphaIntermediateEntry::Link(link_entry) => {
//                let file_path = link_entry.entry_file_path.clone();
//                let files = self.files.lock().unwrap();
//                let entry_option = if let Some(entry) = files.get(&file_path) {
//                    Some(Arc::clone(entry))
//                } else {
//                    None
//                };
//                drop(files);
//                let entry = if let Some(entry) = entry_option {
//                    let locked_entry = entry.lock().unwrap();
//                    Arc::new(Mutex::new(locked_entry.clone()))
//                    // leaving the scope forces `locked_entry` unlock
//                } else {
//                    thread::spawn(move || -> Result<()> {
//
//                        Ok(())
//                    });
//                    self.parse_with_env_set(&file_path, options, Some(Sender::clone(&tx_global)))?
//                };
//                Ok(entry)
//            }
//        }
//    }

    fn resolve(&'static mut self, entry: Arc<Mutex<WisphaIntermediateEntry>>, options: &'static ParserOptions, tx_global: Sender<bool>) -> Result<()> {
        let locked_entry = entry.lock().unwrap();
        let entry_type = locked_entry.get_type();
        drop(locked_entry);
        match entry_type {
            WisphaIntermediateEntry::Direct(_) => {
                let mut locked_entry = entry.lock().unwrap();
                let mut direct_entry = locked_entry.get_direct_entry_mut().unwrap();
                direct_entry.sup_entry = Mutex::new(sync::Weak::new());
                let locked_sub_entries = direct_entry.sub_entries.lock().unwrap();
                for sub_entry in &mut *locked_sub_entries {
                    self.resolve(Arc::clone(sub_entry), options, Sender::clone(&tx_global))?;
                    let mut locked_sub_entry = sub_entry.lock().unwrap();
                    locked_sub_entry.get_direct_entry_mut().unwrap().sup_entry = Mutex::new(Arc::downgrade(&entry));
                    drop(locked_sub_entry);
                }
                drop(locked_sub_entries);
                drop(locked_entry);
                tx_global.send(true).unwrap();
            },
            WisphaIntermediateEntry::Link(_) => {
                let mut locked_entry = entry.lock().unwrap();
                let link_entry = locked_entry.get_link_entry().unwrap();
                let file_path = link_entry.entry_file_path.clone();
                drop(locked_entry);
                let cloned_tx = Sender::clone(&tx_global);
                thread::spawn(move || -> Result<()> {
                    let resolved_entry = self.parse_with_env_set(file_path, options, Some(cloned_tx))?;
                    *entry = *resolved_entry;
                    Ok(())
                });
                tx_global.send(true).unwrap();
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

        if raw.starts_with(ROOT_DIR) {
            let root_dir = PathBuf::from(env::var(ROOT_DIR_VAR).or(Err(ParserError::EnvNotFound))?);
            let relative_path = raw.strip_prefix(ROOT_DIR).unwrap().to_path_buf();
            return Ok(root_dir.join(relative_path));
        }

        // `raw` is not absolute and not starts with `ROOT_DIR`
        Ok(current_dir.join(&raw))
    }
}
