use nom;
//use regex::{self, RegexBuilder};
use onig::*;
use std::fmt::Write as FmtWrite;
use std::io::Write as IoWrite;
use crate::wispha::{self, WisphaEntry, WisphaEntryProperties, WisphaEntryType};

mod error;
use error::ParserError;

type Result<T> = std::result::Result<T, ParserError>;

pub fn parse(raw_content: String) {

}

pub fn parse_with_depth(content: String, depth: u32) -> Result<()> {
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

    for caps in regex_pattern.captures_iter(content.as_str()) {
        println!("{:?}, {:?}",
                 caps.at(1), caps.at(2));
    }

    Ok(())
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
