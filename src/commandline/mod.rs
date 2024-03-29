use structopt::StructOpt;
use console::style;

use std::path::PathBuf;
use std::io::{self, Read, BufReader, Write, Stdin, Take, BufRead};
use std::result::Result;

use crate::manipulator::{Manipulator, error::ManipulatorError};
use crate::commandline::input_parser::InputParser;
use crate::parser::option::ParserOptions;

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
    State(State),
    Convert(Convert),
}

#[derive(StructOpt)]
pub struct Generate {
    #[structopt(short, long)]
    pub flat: bool,
    #[structopt(short, long)]
    pub recursively: bool,
    #[structopt(short, long)]
    pub all: bool,
    #[structopt(short, long)]
    pub threads: Option<usize>,
    pub path: Option<PathBuf>,
}

#[derive(StructOpt)]
pub struct Look {
    #[structopt(short, long)]
    pub threads: Option<usize>,
    pub path: PathBuf,
}

#[derive(StructOpt)]
pub struct State {
    #[structopt(short, long)]
    pub git: bool,
    #[structopt(short, long)]
    pub threads: Option<usize>,
    pub path: PathBuf,
}

#[derive(StructOpt)]
pub struct Convert {
    #[structopt(short, long)]
    pub threads: Option<usize>,
    #[structopt(short, long)]
    pub output: Option<PathBuf>,
    #[structopt(short, long)]
    pub language: Option<String>,
    pub path: PathBuf,
}

impl Convert {
    pub fn update_parser_options(&self, options: &mut ParserOptions) {
        if let Some(threads) = &self.threads {
            options.threads = threads.clone();
        }
    }
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
    Info(Info),
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

#[derive(StructOpt)]
pub struct Info {
    pub name: String,
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
    let look_command = LookCommand::from_iter_safe(input_tokens);
    if let Err(error) = look_command {
        println!("{}", error);
        return Ok(ProgramState::Continuing);
    }
    let look_command = look_command.unwrap();
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

        LookSubcommand::Info(info) => {
            println!("{}", manipulator.info_of_property(&info.name)?);
        }

        LookSubcommand::Quit => {
            return Ok(ProgramState::Quiting);
        },
    }
    Ok(ProgramState::Continuing)
}