use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::helpers::EnvVars;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum FileOrFiles {
    File(PathBuf),
    Files(Vec<PathBuf>),
}

impl From<FileOrFiles> for Vec<PathBuf> {
    fn from(value: FileOrFiles) -> Self {
        match value {
            FileOrFiles::File(v) => vec![v.to_owned()],
            FileOrFiles::Files(v) => v.to_owned(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecConfig {
    pub program: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: EnvVars,
    pub env_file: Option<FileOrFiles>,
    pub cwd: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "when", content = "fields")]
pub enum RunCheck {
    #[serde(rename = "exec_ok")]
    ExecOk(ExecConfig),
    #[serde(rename = "exec_err")]
    ExecErr(ExecConfig),
    #[serde(rename = "path_exists")]
    PathExists { path: PathBuf },
    #[serde(rename = "path_is_file")]
    PathIsFile { path: PathBuf },
    #[serde(rename = "path_is_dir")]
    PathIsDir { path: PathBuf },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunCheckConfig {
    #[serde(flatten)]
    pub when: RunCheck,
    #[serde(default)]
    pub invert: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunConfig {
    #[serde(flatten)]
    pub exec: ExecConfig,
    #[serde(default)]
    pub run_when: Vec<RunCheckConfig>,
    #[serde(default)]
    pub before_hooks: Vec<String>,
    #[serde(default)]
    pub after_hooks: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub configurations: HashMap<String, RunConfig>,
}
