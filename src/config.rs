use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::helpers::{self, EnvVars};

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

impl ExecConfig {
    pub fn all_vars(&self) -> Result<EnvVars, String> {
        let mut vars = self.env.clone();
        if let Some(env_file) = &self.env_file {
            vars.extend(helpers::read_env_files(&Into::<Vec<PathBuf>>::into(
                env_file.clone(),
            ))?);
        }
        Ok(vars)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "when", content = "fields")]
pub enum TargetCheck {
    #[serde(rename = "exec_ok")]
    ExecOk(ExecConfig),
    #[serde(rename = "path_exists")]
    PathExists { path: PathBuf },
    #[serde(rename = "path_is_file")]
    PathIsFile { path: PathBuf },
    #[serde(rename = "path_is_dir")]
    PathIsDir { path: PathBuf },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TargetCheckConfig {
    #[serde(flatten)]
    pub when: TargetCheck,
    #[serde(default)]
    pub invert: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TargetConfig {
    pub description: Option<String>,
    #[serde(flatten)]
    pub exec: Option<ExecConfig>,
    #[serde(default)]
    pub run_when: Vec<TargetCheckConfig>,
    #[serde(default)]
    pub before_hooks: Vec<String>,
    #[serde(default)]
    pub after_hooks: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub targets: HashMap<String, TargetConfig>,
}
