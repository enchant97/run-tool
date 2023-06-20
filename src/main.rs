use std::{
    env::current_exe,
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

// Gets the config, searching in current path.
fn get_config(base: PathBuf) -> Result<Config, AppError> {
    match helpers::read_with_fallbacks(&base, &[".run-tool.yaml".into(), ".run-tool.yml".into()]) {
        Some(v) => match serde_yaml::from_str(&v) {
            Ok(v) => Ok(v),
            Err(_) => Err(AppError {
                msg: format!("failed to parse configuration in '{:?}'", base),
                exitcode: exitcode::CONFIG,
            }),
        },
        None => Err(AppError {
            msg: format!("could not read run configuration in '{:?}'", base),
            exitcode: exitcode::IOERR,
        }),
    }
}

fn main() {
    let args = Args::parse();
    let app_config_base = helpers::get_app_config_path().unwrap_or_else(|| {
        eprintln!("could not locate user home directory");
        exit(exitcode::CONFIG);
    });
    match args.command {
        args::Command::Run {
            config_name,
            global,
            extra_args,
        } => {
            let selected_config = match global {
                true => get_config(app_config_base).unwrap_or_else(|e| e.handle()),
                false => get_config(PathBuf::new()).unwrap_or_else(|e| e.handle()),
            };
            let app_exe_path = current_exe().unwrap_or_else(|_| {
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
