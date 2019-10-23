use crate::config_reader::Config;
use crate::commandline::State;

pub struct StatorOptions {
    pub ignored_files: Vec<String>,
    pub allow_hidden_files: bool,
    pub git: bool,
}

impl StatorOptions {
    pub fn default() -> StatorOptions {
        StatorOptions {
            ignored_files: vec![],
            allow_hidden_files: false,
            git: false,
        }
    }

    pub fn update_from_config(&mut self, config: &Config) {
        if let Some(generate) = &config.generate {
            if let Some(ignored_files) = &generate.ignored_files {
                self.ignored_files = ignored_files.clone();
            }
            if let Some(allow_hidden_files) = &generate.allow_hidden_files {
                self.allow_hidden_files = *allow_hidden_files;
            }
        }
    }

    pub fn update_from_commandline(&mut self, state: &State) {
        self.git = state.git;
    }
}