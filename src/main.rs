mod wispha;
mod parser;
mod generator;
mod commandline;
mod manipulator;
mod config_reader;
use crate::commandline::{WisphaCommand, Subcommand, Generate, Look};
use crate::generator::{error::GeneratorError, option::*};
use crate::parser::{error::{ParserError, ParserErrorInfo}, *};
use crate::manipulator::Manipulator;
use crate::config_reader::error::ConfigError;


use structopt::StructOpt;

use console::style;

use std::{env, fmt};

use std::path::{PathBuf, Path};

use std::fs;

use std::io::{self, Read};
use std::error::Error;
use std::fmt::{Display, Formatter};

use std::result::Result;
use std::boxed::Box;

fn actual_path(raw: &PathBuf) -> Result<PathBuf, MainError> {
    if raw.is_absolute() {
        return Ok(raw.clone());
    }

    let current_dir = env::current_dir().or(Err(MainError::DirectoryNotDetermined))?;
    Ok(current_dir.join(raw))
}

fn error_prefix(error_info: &parser::error::ParserErrorInfo) -> String {
    let mut error_prefix = String::new();
    let path_string = format!("In file {}", &error_info.path.to_str().unwrap());
    error_prefix.push_str(&path_string);
    if let Some(property) = &error_info.property {
        let property_string = format!(", for property {}", property);
        error_prefix.push_str(&property_string);
    }

    error_prefix.push('\n');

    error_prefix
}

fn deal_with_ignore_error(ignore_error: &ignore::Error) {
    match ignore_error {
        ignore::Error::Partial(errors) => {
            for error in errors {
                deal_with_ignore_error(&error);
            }
        },
        ignore::Error::WithLineNumber { line, err } => {
            eprintln!("in line {} ", line);
            deal_with_ignore_error(&*err);
        },
        ignore::Error::WithPath { path, err } => {
            eprintln!("in the file {} ", path.to_str().unwrap());
            deal_with_ignore_error(&*err);
        },
        ignore::Error::WithDepth { depth, err } => {
            eprintln!("to the depth {} ", depth);
            deal_with_ignore_error(&*err);
        },
        ignore::Error::Loop { ancestor, child } => {
            eprintln!("A dead loop occurred because of the {} in {}.", child.to_str().unwrap(), ancestor.to_str().unwrap());
        },
        ignore::Error::Io(_) => {
            eprintln!("IO error. May be lack permission");
        },
        ignore::Error::Glob { glob, err } => {
            let default_value = "".to_string();
            let glob = glob.as_ref().unwrap_or(&default_value);
            eprintln!("An error occurred when parsing {}, because {}", glob, err);
        },
        _ => {

        }
    }
}

fn deal_with_generator_error(generator_error: &GeneratorError) {
    eprintln!("{}", style("error").red());
    eprintln!("{}", generator_error);
}

fn deal_with_parser_error(parser_error: &ParserError) {
    use ParserError::*;
    eprintln!("{}", style("error").red());
    eprintln!("{}", parser_error);
}

fn deal_with_config_error(config_error: &ConfigError) {
    use ConfigError::*;
    eprintln!("{}", style("error").red());
    match config_error {
        DeserializeError(error) => {
            eprintln!("{}", error);
        },
    }
}

fn main_with_error() -> Result<(), Box<dyn Error>> {
    let wispha_command: WisphaCommand = WisphaCommand::from_args();
    match &wispha_command.subcommand {
        Subcommand::Generate(generate) => {
            let mut options = GeneratorOptions::default();
            options.update_from_commandline(generate)?;
            let path = &generate.path;
            let actual_path = actual_path(&path)?;
            println!("Generating...");
            let config = config_reader::read_configs_in_dir(&actual_path)?;
            generator::generate(&actual_path, options)?;
            println!("Successfully generate!");
        },
        Subcommand::Look(look) => {
            let path = &look.path;
            let actual_path = actual_path(&path)?;
            println!("Working on looking...");
            let mut parser = Parser::new();
            let root = parser.parse(&actual_path)?;
            let manipulator = Manipulator::new(&root, &root);
            println!("Looking ready!");
            commandline::continue_program(manipulator);
        }
    }
    Ok(())
}

fn main() {
    let result = main_with_error();
    if let Err(error) = result {
        eprintln!("{}", style("error").red());
        eprintln!("{}", error);
    }
}

#[derive(Debug)]
pub enum MainError {
    DirectoryNotDetermined,
}

impl Error for MainError { }

impl Display for MainError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use MainError::*;
        match &self {
            DirectoryNotDetermined => {
                write!(f, "Can't determine current directory.")
            }
        }
    }
}
