use std::{
    env, fs,
    path::PathBuf,
    process::{exit, Command},
};

use args::Args;
use clap::Parser;
use config::Config;

mod args;
mod config;
mod helpers;

use helpers::AppError;

// Gets the config, searching from current path.
fn get_config(
    base: &PathBuf,
    names: &[PathBuf],
    search: bool,
) -> Result<(PathBuf, Config), AppError> {
    let found_path = match search {
        true => helpers::find_config_with_fallbacks_recursive(base, names),
        false => helpers::find_config_with_fallbacks(base, names),
    };

    if let Some(found_path) = found_path {
        if let Ok(contents) = fs::read_to_string(&found_path) {
            if let Ok(v) = serde_yaml::from_str(&contents) {
                return Ok((found_path, v));
            }
            return Err(AppError {
                msg: format!(
                    "failed to parse configuration in '{}'",
                    found_path.display()
                ),
                exitcode: exitcode::CONFIG,
            });
        }
        return Err(AppError {
            msg: format!(
                "could not read run configuration in '{}'",
                found_path.display()
            ),
            exitcode: exitcode::IOERR,
        });
    }
    Err(AppError {
        msg: format!("failed to find config, searched in '{}'", base.display()),
        exitcode: exitcode::NOINPUT,
    })
}

fn main() {
    let config_file_names = [".run-tool.yaml".into(), ".run-tool.yml".into()];
    let lauched_from_dir = env::current_dir().unwrap_or_else(|_| {
        eprintln!("failed to get current working directory");
        exit(exitcode::OSERR);
    });
    let args = Args::parse();
    let app_config_base = helpers::get_app_config_path().unwrap_or_else(|| {
        eprintln!("could not locate user home directory");
        exit(exitcode::NOINPUT);
    });
    match args.command {
        args::Command::Run {
            config_name,
            global,
            extra_args,
        } => {
            let (config_path, selected_config) = match global {
                true => get_config(&app_config_base, &config_file_names, false)
                    .unwrap_or_else(|e| e.handle()),
                false => get_config(&lauched_from_dir, &config_file_names, true)
                    .unwrap_or_else(|e| e.handle()),
            };

            let config_path_parent = config_path.parent().unwrap_or_else(|| {
                eprintln!("config path has no parent");
                exit(exitcode::SOFTWARE)
            });
            if config_path_parent != lauched_from_dir {
                env::set_current_dir(config_path_parent).unwrap_or_else(|_| {
                    eprintln!("failed to change directory");
                    exit(exitcode::NOINPUT)
                });
            }

            let app_exe_path = env::current_exe().unwrap_or_else(|_| {
                eprintln!("failed to get current executable path");
                exit(exitcode::OSERR);
            });
            let run_config = match selected_config.configurations.get(&config_name) {
                Some(v) => v,
                None => {
                    eprintln!("run configuration not found");
                    exit(exitcode::USAGE);
                }
            };
            let file_envs = match helpers::read_env_files(&run_config.env_files()) {
                Ok(v) => v,
                Err(_) => {
                    eprintln!("failed to parse environment files");
                    exit(exitcode::DATAERR);
                }
            };

            let run_hook = |name: &str| {
                let process = Command::new(&app_exe_path)
                    .args(vec!["run", name])
                    .spawn()
                    .unwrap_or_else(|err| {
                        eprintln!("failed to run '{}': {}", name, err);
                        exit(exitcode::SOFTWARE);
                    })
                    .wait()
                    .unwrap_or_else(|_| {
                        eprintln!("failed to execute");
                        exit(exitcode::OSERR);
                    });
                if !process.success() {
                    exit(process.code().unwrap_or_default())
                }
            };

            for before in &run_config.before_hooks {
                run_hook(before);
            }

            let mut command = Command::new(&run_config.program);

            if let Some(cwd) = &run_config.cwd {
                command.current_dir(cwd);
            }

            let process = command
                .envs(file_envs)
                .envs(&run_config.env)
                .args(&run_config.args)
                .args(&extra_args)
                .spawn()
                .unwrap_or_else(|err| {
                    eprintln!("{}", err);
                    exit(exitcode::SOFTWARE);
                })
                .wait()
                .unwrap_or_else(|_| {
                    eprintln!("failed to execute");
                    exit(exitcode::OSERR);
                });
            if !process.success() {
                exit(process.code().unwrap_or_default())
            }

            for after in &run_config.after_hooks {
                run_hook(after);
            }
        }
    }
}
