use dotenvy::{from_filename_iter, Error as EnvyError};
use std::fs;
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

pub fn read_with_fallbacks(base: &PathBuf, paths: &[PathBuf]) -> Option<String> {
    for path in paths {
        let full_path: PathBuf = [base, &*path].iter().collect();
        if let Ok(v) = fs::read_to_string(full_path) {
            return Some(v);
        }
    }
    None
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
