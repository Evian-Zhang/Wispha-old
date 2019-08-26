use structopt::StructOpt;

use std::path::PathBuf;
use std::io::{self, Read, BufReader, BufRead, Write};

use crate::manipulator::Manipulator;

mod error;
mod input_parser;


#[derive(StructOpt)]
pub struct WisphaCommand {
    #[structopt(subcommand)]
    pub subcommand: Subcommand,
}

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub enum Subcommand {
    Generate(Generate),
    Look(Look),
}

#[derive(StructOpt)]
pub struct Generate {
    pub path: PathBuf,
}

#[derive(StructOpt)]
pub struct Look {
    pub path: PathBuf,
}


const MAX_INPUT_LENGTH: u64 = 256;

pub fn continue_program(manipulator: Manipulator) {
    let mut input = String::new();
    let stdin = io::stdin();
    let mut bstdin = BufReader::new(stdin.take(MAX_INPUT_LENGTH));
    loop {
        print!("(wispha)");
        io::stdout().flush().unwrap();
        input.clear();
        bstdin.read_line(&mut input).unwrap();
        input = input.trim().to_string();
        println!("{}", input);
    }
}
