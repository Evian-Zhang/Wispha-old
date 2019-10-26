use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct WisphaRawToken {
    pub content: String,
    pub line_number: usize,
    pub file_path: PathBuf,
}

#[derive(Clone, PartialEq, Debug)]
pub enum WisphaExpectOption {
    IgnoreDepth,
    AllowLowerDepth,
    IgnoreContent,
}

#[derive(Debug)]
pub enum WisphaToken {
    Header(WisphaRawToken, usize),
    Body(WisphaRawToken),
}

impl WisphaToken {
    pub fn matches(&self, token: &WisphaToken, options: Vec<WisphaExpectOption>) -> bool {
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

    pub fn depth(&self) -> Option<usize> {
        match &self {
            WisphaToken::Header(_, depth) => {
                Some(depth.clone())
            },
            WisphaToken::Body(_) => {
                None
            },
        }
    }

    pub fn default_header_token_with_depth(depth: usize) -> WisphaToken {
        WisphaToken::Header(WisphaRawToken {
            content: "".to_string(),
            line_number: 0,
            file_path: PathBuf::new(),
        }, depth)
    }

    pub fn default_header_token_with_content(content: String) -> WisphaToken {
        WisphaToken::Header(WisphaRawToken {
            content,
            line_number: 0,
            file_path: PathBuf::new(),
        }, 1)
    }

    pub fn default_header_token_with_content_and_depth(content: String, depth: usize) -> WisphaToken {
        WisphaToken::Header(WisphaRawToken {
            content,
            line_number: 0,
            file_path: PathBuf::new(),
        }, depth)
    }

    pub fn empty_body_token() -> WisphaToken {
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

#[derive(Clone)]
pub struct WisphaRawProperty {
    pub header: Arc<WisphaToken>,
    pub body: Vec<Arc<WisphaToken>>,
}
