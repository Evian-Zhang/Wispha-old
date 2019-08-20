use structopt::StructOpt;

use std::path::PathBuf;
use std::io;

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
    let mut stdin = io::stdin();
    let mut input = String::new();
    loop {
        print!("(wispha)");
        input.clear();
        stdin.read_line(&mut input).unwrap();
        input = input.trim().to_string();
    }
}
