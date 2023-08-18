use std::{env, fs, path::PathBuf, process::exit, vec};

use args::Args;
use clap::Parser;
use config::{Config, TargetCheck, TargetCheckConfig};

mod args;
mod config;
mod errors;
mod helpers;
mod runner;

use errors::{AppError, AppErrorResult};
use runner::ProcessRunner;

// Gets the config, searching from current path.
fn get_config(
    base: &PathBuf,
    names: &[PathBuf],
    search: bool,
) -> AppErrorResult<(PathBuf, Config)> {
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

fn check_if_run_needed<'a>(checks: impl Iterator<Item = &'a TargetCheckConfig>) -> bool {
    let checks = checks.map(|check| match &check.when {
        TargetCheck::ExecOk(fields) => {
            exitcode::is_success(
                ProcessRunner {
                    program: fields.program.clone(),
                    args: fields.args.clone(),
                    // TODO include vars
                    vars: Default::default(),
                    // TODO include cwd
                    cwd: None,
                }
                .run_interactive()
                .unwrap_or_else(|err| err.handle()),
            ) != check.invert
        }
        TargetCheck::ExecErr(fields) => {
            exitcode::is_error(
                ProcessRunner {
                    program: fields.program.clone(),
                    args: fields.args.clone(),
                    // TODO include vars
                    vars: Default::default(),
                    // TODO include cwd
                    cwd: None,
                }
                .run_interactive()
                .unwrap_or_else(|err| err.handle()),
            ) != check.invert
        }
        TargetCheck::PathExists { path } => path.exists() != check.invert,
        TargetCheck::PathIsFile { path } => path.is_file() != check.invert,
        TargetCheck::PathIsDir { path } => path.is_dir() != check.invert,
    });
    for ok in checks {
        if !ok {
            return false;
        }
    }
    true
}

fn main() {
    let lauched_from_dir = env::current_dir().unwrap_or_else(|_| {
        eprintln!("failed to get current working directory");
        exit(exitcode::OSERR);
    });
    let args = Args::parse();
    let app_config_base = helpers::get_app_config_path().unwrap_or_else(|| {
        eprintln!("could not locate user home directory");
        exit(exitcode::NOINPUT);
    });

    let config_file_names = match args.custom_filename {
        None => vec![".run-tool.yaml".into(), ".run-tool.yml".into()],
        Some(custom_name) => vec![custom_name],
    };

    match args.command {
        args::Command::Run {
            target_name,
            extra_args,
        } => {
            let (config_path, selected_config) = match (args.use_global_config, args.custom_path) {
                (true, _) => get_config(&app_config_base, &config_file_names, false)
                    .unwrap_or_else(|e| e.handle()),
                (false, Some(custom_path)) => get_config(&custom_path, &config_file_names, true)
                    .unwrap_or_else(|e| e.handle()),
                (false, None) => get_config(&lauched_from_dir, &config_file_names, true)
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
            let target_config = match selected_config.targets.get(&target_name) {
                Some(v) => v,
                None => {
                    eprintln!("run configuration not found");
                    exit(exitcode::USAGE);
                }
            };
            let file_envs = match helpers::read_env_files(
                &target_config
                    .exec
                    .env_file
                    .as_ref()
                    .map(|v| Into::<Vec<PathBuf>>::into(v.clone()))
                    .unwrap_or_default(),
            ) {
                Ok(v) => v,
                Err(_) => {
                    eprintln!("failed to parse environment files");
                    exit(exitcode::DATAERR);
                }
            };

            if !check_if_run_needed(target_config.run_when.iter()) {
                println!("skipping '{}'", target_name);
                exit(exitcode::OK);
            }

            let run_hook = |name: &str| {
                let status = ProcessRunner {
                    program: app_exe_path.to_str().unwrap().to_owned(),
                    args: vec![String::from("run"), String::from(name)],
                    vars: Default::default(),
                    cwd: None,
                }
                .run_interactive()
                .unwrap_or_else(|err| {
                    eprintln!("hook '{}' encountered an error: '{}'", name, err.msg);
                    exit(exitcode::SOFTWARE);
                });
                if exitcode::is_error(status) {
                    exit(status)
                }
            };

            for before in &target_config.before_hooks {
                run_hook(before);
            }

            let mut args = target_config.exec.args.clone();
            args.extend(extra_args);
            let mut vars = file_envs.clone();
            vars.extend(target_config.exec.env.clone());

            let status = ProcessRunner {
                program: target_config.exec.program.clone(),
                args,
                vars,
                cwd: target_config.exec.cwd.clone(),
            }
            .run_interactive()
            .unwrap_or_else(|err| err.handle());
            if status != exitcode::OK {
                exit(status);
            }

            for after in &target_config.after_hooks {
                run_hook(after);
            }
        }
    }
}
