mod wispha;
mod parser;
mod generator;

use std::fs;

fn main() {
    println!("Hello, world!");
//    generator::generate();
    let content = fs::read_to_string("/Users/evian/Downloads/zs.txt").unwrap();
    parser::parse_with_depth(content, 0);
}
