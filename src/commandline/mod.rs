use structopt::StructOpt;
use console::style;

use std::path::PathBuf;
use std::io::{self, Read, BufReader, Write, Stdin, Take, BufRead};
use std::result::Result;
use std::error::Error;

use crate::manipulator::{Manipulator, error::ManipulatorError};
use crate::commandline::input_parser::InputParser;

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
    #[structopt(short, long)]
    pub flat: bool,
    #[structopt(short, long)]
    pub recursively: bool,
    pub path: PathBuf,
}

#[derive(StructOpt)]
pub struct Look {
    pub path: PathBuf,
}

#[derive(StructOpt)]
pub struct LookCommand {
    #[structopt(subcommand)]
    pub subcommand: LookSubcommand,
}

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub enum LookSubcommand {
    Cd(Cd),
    Ls(Ls),
    Quit,
}

#[derive(StructOpt)]
pub struct Cd {
    #[structopt(short, long)]
    pub local: bool,
    pub path: PathBuf,
}

#[derive(StructOpt)]
pub struct Ls {
    #[structopt(short, long)]
    pub local: bool,
    pub path: Option<PathBuf>,
}

enum ProgramState {
    Continuing,
    Quiting,
}

const MAX_INPUT_LENGTH: u64 = 256;

pub fn continue_program(mut manipulator: Manipulator) {
    let stdin = io::stdin();
    let mut bstdin = BufReader::new(stdin.take(MAX_INPUT_LENGTH));
    loop {
        let result = continue_program_with_error(&mut manipulator, &mut bstdin);
        match result {
            Ok(state) => {
                match state {
                    ProgramState::Continuing => {

                    },
                    ProgramState::Quiting => {
                        return;
                    },
                }
            },
            Err(error) => {
                eprintln!("{}", error);
            }
        }
    }
}

fn continue_program_with_error(manipulator: &mut Manipulator, bstdin: &mut BufReader<Take<Stdin>>) -> Result<ProgramState, ManipulatorError> {
    let prompt = format!("wispha@{} >", manipulator.current_path().to_str().unwrap());
    print!("{}", style(prompt).cyan());
    io::stdout().flush().unwrap();
    let mut input = String::new();
    input.clear();
    bstdin.read_line(&mut input).unwrap();
    let input = input.trim().to_string();

    let input_parser = InputParser::new(input.clone());
    let mut input_tokens: Vec<String> = input_parser.collect();
    input_tokens.insert(0, String::from("wispha"));
    let look_command = LookCommand::from_iter_safe(input_tokens).unwrap();
    match look_command.subcommand {
        LookSubcommand::Cd(cd) => {
            if cd.local {
                manipulator.set_current_entry_to_local_path(&cd.path)?;
            } else {
                manipulator.set_current_entry_to_path(&cd.path)?;
            }
        },

        LookSubcommand::Ls(ls) => {
            match ls.path {
                Some(path) => {
                    if ls.local {
                        let list = manipulator.list_of_local_path(&path)?;
                        if list.len() > 0 {
                            println!("{}", list);
                        }
                    } else {
                        let list = manipulator.list_of_path(&path)?;
                        if list.len() > 0 {
                            println!("{}", list);
                        }
                    }
                }

                None => {
                    let list = manipulator.current_list();

                    if list.len() > 0 {
                        println!("{}", list);
                    }
                }
            }
        },

        LookSubcommand::Quit => {
            return Ok(ProgramState::Quiting);
        },
    }
    Ok(ProgramState::Continuing)
}