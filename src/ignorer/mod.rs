use std::string::String;
use std::char;
use std::usize;

use unicode_segmentation::{UnicodeSegmentation, Graphemes};

fn do_wild(pattern: &String, text: &String) -> bool {
    let mut pattern = UnicodeSegmentation::graphemes(pattern.as_str(), true);
    let mut text = UnicodeSegmentation::graphemes(text.as_str(), true);

    do_wild_grapheme(&mut pattern, &mut text)
}

fn do_wild_vec(pattern: &Vec<&str>, text: &Vec<&str>, pattern_index: usize, text_index: usize) -> bool {
    let mut pattern_index = start_index;
    while pattern.get(pattern_index) != None {
        let mut match_slash = false;
        let mut pattern_grapheme = pattern.get(pattern_index);
        let mut text_index = text_index;
        if text.get(text_index) == None && pattern_grapheme == Some(&"*") {
            return false;
        }
        let text_grapheme = text.get(text_index);
        match *pattern_grapheme.unwrap() {
            "\\" => {
                pattern_index += 1;
                pattern_grapheme = pattern.get(pattern_index);
                if text_grapheme != pattern_grapheme {
                    return false;
                }

                text_index += 1;
                pattern_index += 1;
                continue;
            }

            "?" => {
                if text_grapheme == Some(&"/") {
                    return false;
                }

                text_index += 1;
                pattern_index += 1;
                continue;
            }

            "*" => loop {
                {
                    pattern_index += 1;
                    if pattern.get(pattern_index) == Some(&"*") {
                        let (previous_pattern_index, overflow) = pattern_index.overflowing_sub(2);
                        while pattern.get(pattern_index) == Some(&"*") {
                            pattern_index += 1;
                        }
                        if (overflow || pattern.get(previous_pattern_index) == Some(&"/")) &&
                            (pattern.get(pattern_index) == None || pattern.get(pattern_index) == Some(&"/") ||
                                (pattern.get(pattern_index) == Some(&"\\") && pattern.get(pattern_index + 1) == Some(&"/"))) {
                            if pattern.get(pattern_index) == Some(&"/") && do_wild_vec(&pattern, &text, pattern_index, text_index) {
                                return true;
                            }
                            match_slash = true;
                        } else {
                            match_slash = false;
                        }
                    } else {
                        match_slash = false;
                    }
                    if pattern.get(pattern_index) == None {
                        if !match_slash {
                            if text.contains(&"/") {
                                return false;
                            }
                            return true;
                        }
                    } else if !match_slash && pattern.get(pattern_index) == Some(&"/") {
                        let slash = text.iter().position(|&r| { r == "/" });
                        if slash == None {
                            return false;
                        }
                        text_index = slash.unwrap();
                        break;
                    }
                    loop {
                        if text.get(text_index) == None {
                            break;
                        }
                        let tmp = pattern.get(pattern_index);
                        if !(tmp == Some(&"*") || tmp == Some(&"?") || tmp == Some(&"[") || tmp == Some(&"\\")) {
                            while text.get(text_index) != None &&
                                (match_slash || text.get(text_index) != Some(&"/")) {
                                if text.get(text_index) == pattern.get(pattern_index) {
                                    break;
                                }
                                text_index += 1;
                            }
                            if text.get(text_index) != pattern.get(pattern_index) {
                                return false;
                            }
                        }
                        if do_wild_vec(&pattern, &text, pattern_index, text_index) {
                            if !match_slash {
                                return true;
                            }
                        } else if !match_slash && text.get(text_index) == Some(&"/") {
                            return false;
                        }
                        text_index += 1;
                    }
                    return false;
                }
                break;
            }


            _ => {
                if text_grapheme != pattern_grapheme {
                    return false;
                }

                text_index += 1;
                pattern_index += 1;
                continue;
            }
        }

        text_index += 1;
        pattern_index += 1;
    }

    false
}

fn do_wild_grapheme(pattern: &mut Graphemes, text: &mut Graphemes) -> bool {
    let mut text_backup = text.clone();
    let pattern_vec: Vec<&str> = pattern.clone().collect();
    let mut pattern_index = usize::max_value();
    while let Some(pattern_grapheme) = pattern.next() {
        pattern_index += 1;

        let text_grapheme = text.next();
        if text_grapheme == None && pattern_grapheme != "*" {
            return false;
        }
        let text_grapheme = text_grapheme.unwrap();
        match pattern_grapheme {
            "\\" => {
                if let Some(pattern_grapheme) = pattern.next() {
                    pattern_index += 1;
                    if text_grapheme != pattern_grapheme {
                        return false;
                    }
                }
                continue;
            }

            "?" => {
                if text_grapheme == "/" {
                    return false;
                }
                continue;
            }

            "*" => {
                if let Some(pattern_grapheme) = pattern.next() {
                    pattern_index += 1;
                    if pattern_grapheme == "*" {
                        let (previous_pattern_index, overflow) = pattern_index.overflowing_sub(2);
                        let mut pattern_grapheme = pattern.next();
                        while pattern_grapheme == Some("*") {
                            pattern_grapheme = pattern.next();
                        }
                        let mut tmp_pattern = pattern.clone();
                        if (overflow || pattern_vec[previous_pattern_index] == "/") &&
                            (pattern_grapheme == None || pattern_grapheme == Some("/") ||
                                (pattern_grapheme == Some("\\") && tmp_pattern.next() == Some("/"))) {
                            if pattern_grapheme == Some("/") && do_wild_grapheme(&mut tmp_pattern, &mut text_backup) {
                                return true;
                            }
                        }
                    }
                }
            }

            _ => {
                if text_grapheme != pattern_grapheme {
                    return false;
                }
                continue;
            }
        }
    }
//    let text_vec: Vec<&str> = text.collect();
//    let text_len = text_vec.len();
//    if text_len == 0 {
//        return false;
//    }
//
//    let pattern_vec: Vec<&str> = pattern.collect();
//    let pattern_len = pattern_vec.len();
//    if pattern_len == 0 {
//        return false;
//    }
//
//    let mut pattern_index = 0;
//
//    while pattern_index < pattern_len {
//        let mut pattern_grapheme = pattern_vec[pattern_index];
//        let text_index = pattern_index;
//
//        if text_index >= text_len && pattern_grapheme != "*" {
//            return false;
//        }
//
//        let text_grapheme = text_vec[pattern_index];
//
//        match pattern_grapheme {
//            "\\" => {
//                pattern_index += 1;
//                pattern_grapheme = pattern_vec[pattern_index];
//                if text_grapheme != pattern_grapheme {
//                    return false;
//                }
//                continue;
//            }
//
//            "?" => {
//                if text_grapheme == "/" {
//                    return false;
//                }
//                continue;
//            }
//
//            "*" => {
//                pattern_index += 1;
//                if pattern_index < pattern_len && pattern_vec[pattern_index] == "*" {
//                    let (previous_pattern_index, overflow) = pattern_index.overflowing_sub(2);
//                    while pattern_index < pattern_len && pattern_vec[pattern_index] == "*" {
//                        pattern_index += 1;
//                    }
//                    if (!overflow || pattern_vec[previous_pattern_index] == "/") &&
//                        (pattern_index >= pattern_len || pattern_vec[pattern_index] == "/" ||
//                            (pattern_index + 1 < pattern_len &&
//                                pattern_vec[pattern_index] == "\\" && pattern_vec[pattern_index + 1] == "/")) {
//                        if pattern_vec[pattern_index] == "/" && do_wild() {
//
//                        }
//                    }
//                }
//            }
//
//            _ => {
//                if text_grapheme != pattern_grapheme {
//                    return false;
//                }
//                continue;
//            }
//        }
//
//    }

    true
}