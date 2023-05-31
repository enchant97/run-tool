use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RunConfig {
    pub program: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub env_file: Vec<PathBuf>,
    pub cwd: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub configurations: HashMap<String, RunConfig>,
}
