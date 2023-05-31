use dotenvy::{from_filename_iter, Error as EnvyError};
use std::path::PathBuf;

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
