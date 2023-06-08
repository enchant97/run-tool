use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FileOrFiles {
    File(PathBuf),
    Files(Vec<PathBuf>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunConfig {
    pub program: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    pub env_file: Option<FileOrFiles>,
    pub cwd: Option<PathBuf>,
    #[serde(default)]
    pub before_hooks: Vec<String>,
    #[serde(default)]
    pub after_hooks: Vec<String>,
}

impl RunConfig {
    pub fn env_files(&self) -> Vec<PathBuf> {
        match &self.env_file {
            None => vec![],
            Some(v) => match v {
                FileOrFiles::File(v) => vec![v.to_owned()],
                FileOrFiles::Files(v) => v.to_owned(),
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub configurations: HashMap<String, RunConfig>,
}
