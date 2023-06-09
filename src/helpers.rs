use dotenvy::{from_filename_iter, Error as EnvyError};
use std::path::PathBuf;
use std::process::exit;

const CONFIG_FOLDER_NAME: &str = "run-tool";

#[derive(Debug)]
pub struct AppError {
    pub msg: String,
    pub exitcode: i32,
}

impl AppError {
    pub fn handle(&self) -> ! {
        eprintln!("{}", self.msg);
        exit(self.exitcode);
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

pub fn read_env_files(paths: &[PathBuf]) -> Result<Vec<(String, String)>, EnvyError> {
    let mut variables = Vec::<(String, String)>::new();
    for file_path in paths {
        let v = match from_filename_iter(file_path) {
            Err(err) => return Err(err),
            Ok(v) => v.into_iter().filter_map(|item| item.ok()),
        };
        variables.extend(v);
    }
    Ok(variables)
}
