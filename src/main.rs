mod wispha;
mod parser;
mod generator;
mod commandline;
mod manipulator;
mod error;

use error::MainError;
use crate::commandline::{WisphaCommand, Subcommand, Generate, Look};
use crate::generator::{error::GeneratorError, option::*};
use crate::parser::{error::{ParserError, ParserErrorInfo}, *};
use crate::manipulator::Manipulator;


use structopt::StructOpt;

use console::style;

use std::env;

use std::path::{PathBuf, Path};

use std::fs;

use std::io::{self, Read};

type Result<T> = std::result::Result<T, MainError>;

fn actual_path(raw: &PathBuf) -> Result<PathBuf> {
    if raw.is_absolute() {
        return Ok(raw.clone());
    }

    let current_dir = env::current_dir().or(Err(MainError::PathInvalid))?;
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
    match generator_error {
        GeneratorError::DirCannotRead(path) => {
            eprintln!("Cannot read directory {}.", path.to_str().unwrap());
        },
        GeneratorError::PathIsNotDir(path) => {
            eprintln!("Path {} is not a directory.", path.to_str().unwrap());
        },
        GeneratorError::NameNotDetermined(path) => {
            eprintln!("Cannot determine the entry name of {}.", path.to_str().unwrap());
        },
        GeneratorError::NameNotValid(path) => {
            eprintln!("Path {} contains invalid characters.", path.to_str().unwrap());
        },
        GeneratorError::IgnoreError(ignore_error) => {
            deal_with_ignore_error(&ignore_error);
        }
        GeneratorError::Unexpected => {
            eprintln!("Unexpected error. Please retry.");
        },
        GeneratorError::FileCannotWrite(path) => {
            eprintln!("Cannot write to file {}. Permission denied.", path.to_str().unwrap());
        }
    }
}

fn deal_with_parser_error(parser_error: &ParserError) {
    use ParserError::*;
    eprintln!("{}", style("error").red());
    match parser_error {
        UnrecognizedEntryFileType(token) => {
            eprintln!("In file {}, line {}:\nUnrecognized entry file type {}.", token.raw_token().file_path.to_str().unwrap(), token.raw_token().line_number, token.raw_token().content);
        },
        FileCannotRead(path) => {
            eprintln!("Cannot read file {}.", path.to_str().unwrap());
        },
        UnexpectedToken(token, _) => {
            eprintln!("In file {}, line {}:\nUnexpected token {}", token.raw_token().file_path.to_str().unwrap(), token.raw_token().line_number, token.raw_token().content);
        },
        EmptyBody(token) => {
            eprintln!("In file {}, line {}:\nProperty {} has empty body.", token.raw_token().file_path.to_str().unwrap(), token.raw_token().line_number, token.raw_token().content);
        },
        EnvNotFound => {
            eprintln!("Cannot determine the environment variable.");
        },
    }
}

fn main() {
    let wispha_command: WisphaCommand = WisphaCommand::from_args();
    match &wispha_command.subcommand {
        Subcommand::Generate(generate) => {
            if generate.flat && generate.recursively {
                eprintln!("Cannot specify flat and recursively at same time.");
            } else {
                let layer = if generate.flat {
                    GenerateLayer::Flat
                } else {
                    GenerateLayer::Recursive
                };
                let options = GeneratorOptions {
                    layer
                };
                let path = &generate.path;
                let actual_path_result = actual_path(&path);
                if let Ok(actual_path) = actual_path_result {
                    println!("Generating...");
                    let result = generator::generate(&actual_path, options);
                    match result {
                        Ok(_) => {
                            println!("Successfully generate!");
                        },
                        Err(generator_error) => {
                            deal_with_generator_error(&generator_error);
                        },
                    }
                } else {
                    eprintln!("Path {} does not exist.", path.to_str().unwrap());
                }
            }
        },
        Subcommand::Look(look) => {
            let path = &look.path;
            let actual_path_result = actual_path(&path);
            if let Ok(actual_path) = actual_path_result {
                println!("Working on looking...");
                let mut parser = Parser::new();
                let result = parser.parse(&actual_path);
                match result {
                    Ok(root) => {
                        let manipulator = Manipulator::new(&root, &root);
                        println!("Looking ready!");
                        commandline::continue_program(manipulator);
                    },
                    Err(parser_error) => {
                        deal_with_parser_error(&parser_error);
                    }
                }
            } else {
                eprintln!("Path {} does not exist.", path.to_str().unwrap());
            }
        },
    }
}
