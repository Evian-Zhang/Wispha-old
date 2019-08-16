mod wispha;
mod parser;
mod generator;

use std::path::PathBuf;

use std::fs;

fn main() {
    println!("Hello, world!");
//    generator::generate();
    let content = fs::read_to_string("/Users/evian/Downloads/llvm/llvm/LOOKME.wispha").unwrap();
//    parser::parse_with_depth(content, 0);
    parser::parse(content, &PathBuf::from("/Users/evian/Downloads/llvm/llvm/"));
}
