use structopt::StructOpt;

use std::path::PathBuf;
use std::io::{self, Read, BufReader, BufRead, Write};

use crate::manipulator::{Manipulator, error::ManipulatorError};
use crate::commandline::input_parser::InputParser;

mod input_parser;

use console::style;

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

const MAX_INPUT_LENGTH: u64 = 256;

fn handle_manipulator_error(error: ManipulatorError) {
    match error {
        ManipulatorError::PathNotEntry(path) => {
            eprintln!("Cannot find entry in {}", path.to_str().unwrap());
        },
        ManipulatorError::PathNotExist => {
            eprintln!("Path not exist!");
        },
        ManipulatorError::AbsolutePathNotSupported => {
            eprintln!("Don't support absolute path.");
        },
        ManipulatorError::BeyondDomain => {
            eprintln!("Path is beyond wispha domain.");
        },
        ManipulatorError::EntryNotFound(path) => {
            eprintln!("Cannot find entry in {}", path.to_str().unwrap());
        },
        ManipulatorError::Unexpected => {
            eprintln!("Unexpected error.");},
    }
}

pub fn continue_program(mut manipulator: Manipulator) {
    let mut input = String::new();
    let stdin = io::stdin();
    let mut bstdin = BufReader::new(stdin.take(MAX_INPUT_LENGTH));
    loop {
        let prompt = format!("wispha@{} >", manipulator.current_path().to_str().unwrap());
        print!("{}", style(prompt).cyan());
        io::stdout().flush().unwrap();
        input.clear();
        bstdin.read_line(&mut input).unwrap();
        input = input.trim().to_string();

        let input_parser = InputParser::new(input.clone());
        let mut input_tokens: Vec<String> = input_parser.collect();
        input_tokens.insert(0, String::from("wispha"));
        match LookCommand::from_iter_safe(input_tokens) {
            Ok(look_command) => {
                match look_command.subcommand {
                    LookSubcommand::Cd(cd) => {
                        if cd.local {
                            if let Err(error) = manipulator.set_current_entry_to_local_path(&cd.path) {
                                handle_manipulator_error(error);
                            }
                        } else {
                            if let Err(error) = manipulator.set_current_entry_to_path(&cd.path) {
                                handle_manipulator_error(error);
                            }
                        }
                    },

                    LookSubcommand::Ls(ls) => {
                        match ls.path {
                            Some(path) => {
                                if ls.local {
                                    match manipulator.list_of_local_path(&path) {
                                        Ok(list) => {
                                            if list.len() > 0 {
                                                println!("{}", list);
                                            }
                                        },

                                        Err(err) => {
                                            handle_manipulator_error(err);
                                        }
                                    }
                                } else {
                                    match manipulator.list_of_path(&path) {
                                        Ok(list) => {
                                            if list.len() > 0 {
                                                println!("{}", list);
                                            }
                                        },

                                        Err(err) => {
                                            handle_manipulator_error(err);
                                        }
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
                        return;
                    },
                }
            },
            Err(error) => {
                println!("{}", error);
            }
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