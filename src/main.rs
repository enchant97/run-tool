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

fn check_if_run_needed<'a>(
    checks: impl Iterator<Item = &'a TargetCheckConfig>,
) -> AppErrorResult<bool> {
    let checks = checks.map(|check| match &check.when {
        TargetCheck::ExecOk(fields) => Ok(exitcode::is_success(
            ProcessRunner {
                program: fields.program.clone(),
                args: fields.args.clone(),
                vars: fields.all_vars().map_err(|err| AppError {
                    msg: format!("failed to parse environment files: '{}'", err),
                    exitcode: exitcode::DATAERR,
                })?,
                cwd: fields.cwd.clone(),
            }
            .run_interactive()?,
        ) != check.invert),
        TargetCheck::PathExists { path } => Ok(path.exists() != check.invert),
        TargetCheck::PathIsFile { path } => Ok(path.is_file() != check.invert),
        TargetCheck::PathIsDir { path } => Ok(path.is_dir() != check.invert),
    });
    for ok in checks {
        if !ok? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn main() {
    let args = Args::parse();

    let lauched_from_dir = env::current_dir().unwrap_or_else(|_| {
        eprintln!("failed to get current working directory");
        exit(exitcode::OSERR);
    });
    let app_exe_path = env::current_exe().unwrap_or_else(|_| {
        eprintln!("failed to get current executable path");
        exit(exitcode::OSERR);
    });
    let app_config_base = helpers::get_app_config_path().unwrap_or_else(|| {
        eprintln!("could not locate user home directory");
        exit(exitcode::NOINPUT);
    });

    let config_file_names = match args.custom_filename {
        None => vec![".run-tool.yaml".into(), ".run-tool.yml".into()],
        Some(custom_name) => vec![custom_name],
    };

    let (config_path, selected_config) =
        match (args.use_global_config, args.custom_path) {
            (true, _) => get_config(&app_config_base, &config_file_names, false)
                .unwrap_or_else(|e| e.handle()),
            (false, Some(custom_path)) => {
                get_config(&custom_path, &config_file_names, true).unwrap_or_else(|e| e.handle())
            }
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

    match args.command {
        args::Command::Config { minimal } => {
            println!("file:");
            println!("  {}", config_path.display());
            println!("targets:");
            for target in selected_config.targets {
                if minimal {
                    println!(
                        "  {}: {}",
                        target.0,
                        target.1.description.unwrap_or_default()
                    );
                    continue;
                }
                println!("  {}:", target.0);
                if let Some(description) = target.1.description {
                    println!("    description:");
                    println!("      {}", description);
                }
                println!("    exec:");
                println!(
                    "      {} {}",
                    target.1.exec.program,
                    target.1.exec.args.join(" ")
                );
                if let Some(cwd) = target.1.exec.cwd {
                    println!("    cwd:");
                    println!("      {}", cwd);
                }
                if target.1.before_hooks.len() != 0 {
                    println!("    before hooks:");
                    println!("      {}", target.1.before_hooks.join(", "));
                }
                if target.1.after_hooks.len() != 0 {
                    println!("    after hooks:");
                    println!("      {}", target.1.after_hooks.join(", "));
                }
                println!("    run checks:");
                println!("      {}", target.1.run_when.len());
            }
        }
        args::Command::Run {
            target_name,
            extra_args,
        } => {
            let target_config = match selected_config.targets.get(&target_name) {
                Some(v) => v,
                None => {
                    eprintln!("run configuration not found");
                    exit(exitcode::USAGE);
                }
            };

            let environment_variables = &target_config.exec.all_vars().unwrap_or_else(|err| {
                eprintln!("failed to parse environment files: '{}'", err);
                exit(exitcode::DATAERR);
            });

            if !check_if_run_needed(target_config.run_when.iter()).unwrap_or_else(|err| {
                err.handle();
            }) {
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

            let status = ProcessRunner {
                program: target_config.exec.program.clone(),
                args,
                vars: environment_variables.clone(),
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
