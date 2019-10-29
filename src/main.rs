mod wispha;
mod parser;
mod generator;
mod commandline;
mod manipulator;
mod config_reader;
mod stator;
mod helper;
mod strings;

use crate::commandline::{WisphaCommand, Subcommand};
use crate::generator::{error::GeneratorError, option::*};
use crate::parser::{error::ParserError, option::*};
use crate::manipulator::Manipulator;
use crate::config_reader::error::ConfigError;

use structopt::StructOpt;
use console::style;

use std::{env, fmt};
use std::path::PathBuf;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::result::Result;
use crate::stator::option::StatorOptions;
use crate::stator::error::StatorError;

// `raw`: relative or absolute. If cannot determine current directory, an error is raised
fn actual_path(raw: &PathBuf) -> Result<PathBuf, MainError> {
    if raw.is_absolute() {
        return Ok(raw.clone());
    }

    let current_dir = env::current_dir().or(Err(MainError::DirectoryNotDetermined))?;
    Ok(current_dir.join(raw))
}

fn main_with_error() -> Result<(), MainError> {
    // get commandline arguments
    let wispha_command: WisphaCommand = WisphaCommand::from_args();

    match &wispha_command.subcommand {
        Subcommand::Generate(generate) => {
            let path = if let Some(path) = &generate.path {
                actual_path(&path)?
            } else {
                env::current_dir().or(Err(MainError::DirectoryNotDetermined))?
            };
            println!("Generating...");

            // get generator options from config and commandline
            let mut options = GeneratorOptions::default();
            let config = config_reader::read_configs_in_dir(&path)?;
            if let Some(config) = config {
                options.update_from_config(&config)?;
            }
            options.update_from_commandline(generate)?;

            generator::generate(path, options)?;
            println!("Successfully generate!");
        },

        Subcommand::Look(look) => {
            let path = &look.path;
            let actual_path = actual_path(&path)?;
            println!("Working on looking...");

            // get parser options from config
            let mut options = ParserOptions::default();
            let config = config_reader::read_configs_in_dir(&actual_path)?;
            if let Some(config) = config {
                options.update_from_config(&config)?;
            }
            options.update_from_commandline(look);

            let root = parser::parse(&actual_path, options)?;

            let manipulator = Manipulator::new(&root, &root);
            println!("Looking ready!");
            commandline::continue_program(manipulator);
        },

        Subcommand::State(state) => {
            let path = &state.path;
            let actual_path = actual_path(&path)?;
            println!("Traversing to determine state...");

            let mut options = StatorOptions::default();
            let config = config_reader::read_configs_in_dir(&actual_path)?;
            if let Some(config) = config {
                options.update_from_config(&config);
            }
            options.update_from_commandline(state);

            let unrecorded_files = stator::state_from_path(&actual_path, options)?;
            if unrecorded_files.is_empty() {
                println!("All valid files are recorded in Wispha.");
            } else {
                let unrecorded_files_strs: Vec<String> = unrecorded_files.iter().map(|path| path.to_str().unwrap().to_string()).collect();
                println!("The following file(s) are not recorded in Wispha:\n{}", unrecorded_files_strs.join("\n"));
            }
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
    GeneratorError(GeneratorError),
    ParserError(ParserError),
    GeneratorOptionError(GeneratorOptionError),
    ConfigError(ConfigError),
    ParserOptionError(ParserOptionError),
    StatorError(StatorError),
}

impl Error for MainError { }

impl Display for MainError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use MainError::*;
        let error_message = match &self {
            DirectoryNotDetermined => {
                format!("Can't determine current directory.")
            },
            GeneratorError(error) => {
                format!("{}", error)
            },
            ParserError(error) => {
                format!("{}", error)
            },
            GeneratorOptionError(error) => {
                format!("{}", error)
            },
            ConfigError(error) => {
                format!("{}", error)
            },
            ParserOptionError(error) => {
                format!("{}", error)
            },
            StatorError(error) => {
                format!("{}", error)
            }
        };
        write!(f, "{}", error_message)
    }
}

impl From<GeneratorError> for MainError {
    fn from(error: GeneratorError) -> Self {
        MainError::GeneratorError(error)
    }
}

impl From<ParserError> for MainError {
    fn from(error: ParserError) -> Self {
        MainError::ParserError(error)
    }
}

impl From<GeneratorOptionError> for MainError {
    fn from(error: GeneratorOptionError) -> Self {
        MainError::GeneratorOptionError(error)
    }
}

impl From<ConfigError> for MainError {
    fn from(error: ConfigError) -> Self {
        MainError::ConfigError(error)
    }
}

impl From<ParserOptionError> for MainError {
    fn from(error: ParserOptionError) -> Self {
        MainError::ParserOptionError(error)
    }
}

impl From<StatorError> for MainError {
    fn from(error: StatorError) -> Self {
        MainError::StatorError(error)
    }
}
