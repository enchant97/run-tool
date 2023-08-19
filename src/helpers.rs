use dotenvy::from_filename_iter;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::sync::OnceLock;

const CONFIG_FOLDER_NAME: &str = "run-tool";

static APP_BIN_PATH: OnceLock<PathBuf> = OnceLock::new();

pub type EnvVars = HashMap<String, String>;

pub fn get_app_binary_path() -> &'static PathBuf {
    APP_BIN_PATH.get_or_init(|| env::current_exe().expect("failed to get current binary path"))
}

pub fn get_config_file_names(custom_filename: Option<PathBuf>) -> Vec<PathBuf> {
    match (env::var("RUN_TOOL_FILENAME").ok(), custom_filename) {
        (None, None) => {
            log::debug!("setting config filenames from internal");
            vec![".run-tool.yaml".into(), ".run-tool.yml".into()]
        }
        (Some(v), None) => {
            log::debug!("setting config filename from environment variable");
            vec![v.into()]
        }
        (_, Some(custom_name)) => {
            log::debug!("setting config filename from argument");
            vec![custom_name]
        }
    }
}

pub fn get_app_config_path() -> Option<PathBuf> {
    if cfg!(windows) {
        match std::env::var("USERPROFILE") {
            Ok(v) => {
                let mut v = PathBuf::from(v);
                v.push(format!(".config/{CONFIG_FOLDER_NAME}"));
                Some(v)
            }
            Err(_) => None,
        }
    } else {
        match std::env::var("XDG_CONFIG_HOME") {
            Ok(v) => {
                let mut v = PathBuf::from(v);
                v.push(CONFIG_FOLDER_NAME);
                Some(v)
            }
            Err(_) => match std::env::var("HOME") {
                Ok(v) => {
                    let mut v = PathBuf::from(v);
                    v.push(format!(".config/{CONFIG_FOLDER_NAME}"));
                    Some(v)
                }
                Err(_) => None,
            },
        }
    }
}

pub fn find_config_with_fallbacks(base: &PathBuf, names: &[PathBuf]) -> Option<PathBuf> {
    if !base.is_dir() {
        return None;
    }
    for name in names {
        let full_path: PathBuf = [base, name].iter().collect();
        if full_path.is_file() {
            return Some(full_path);
        }
    }
    None
}

pub fn find_config_with_fallbacks_recursive(base: &PathBuf, names: &[PathBuf]) -> Option<PathBuf> {
    if let Some(v) = find_config_with_fallbacks(base, names) {
        return Some(v);
    }
    base.parent()
        .and_then(|base| find_config_with_fallbacks_recursive(&base.to_owned(), names))
}

pub fn read_env_files(paths: &[PathBuf]) -> Result<EnvVars, String> {
    let mut variables = EnvVars::new();
    for file_path in paths {
        let v = match from_filename_iter(file_path) {
            Err(err) => return Err(err.to_string()),
            Ok(v) => v.into_iter().filter_map(|item| item.ok()),
        };
        variables.extend(v);
    }
    Ok(variables)
}
