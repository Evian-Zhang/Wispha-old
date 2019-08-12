use nom;
use crate::wispha::{self, WisphaEntry, WisphaEntryProperties, WisphaEntryType};

mod error;
use error::ParserError;

pub fn parse(raw_content: String) {

}

fn parse_with_depth(content: String, depth: u32) {
    let begin_mark = begin_mark(depth);
    let begin_mark_tag = tag!(begin_mark);
}

fn begin_mark(depth: u32) -> String {
    let mut counter = 0;
    let mut begin_mark = String::new();
    while counter <= depth {
        write!(&mut begin_mark, "{}", wispha::BEGIN_MARK).or(Err(ParserError::Unexpected));
        counter += 1;
    }

    begin_mark
}
