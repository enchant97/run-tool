use std::{
    env, fs,
    path::PathBuf,
    process::exit,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::sleep,
    time::Duration,
};

use args::Args;
use clap::Parser;
use config::{Config, TargetCheck, TargetCheckConfig};

mod args;
mod config;
mod errors;
mod helpers;
mod runner;

use errors::{AppError, AppErrorResult};
use helpers::get_app_binary_path;
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode};
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
            if let Ok(v) = serde_yml::from_str(&contents) {
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

fn command_config(config_path: PathBuf, config: Config, minimal: bool) -> AppErrorResult<()> {
    println!("file:");
    println!("  {}", config_path.display());
    println!("targets:");
    for target in config.targets {
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
        if let Some(exec) = target.1.exec {
            println!("      {} {}", exec.program, exec.args.join(" "));
            if let Some(cwd) = exec.cwd {
                println!("    cwd:");
                println!("      {}", cwd);
            }
        }
        if !target.1.before_hooks.is_empty() {
            println!("    before hooks:");
            println!("      {}", target.1.before_hooks.join(", "));
        }
        if !target.1.after_hooks.is_empty() {
            println!("    after hooks:");
            println!("      {}", target.1.after_hooks.join(", "));
        }
        println!("    run checks:");
        println!("      {}", target.1.run_when.len());
    }
    Ok(())
}

fn command_run(
    config: Config,
    target_name: &str,
    extra_args: Vec<String>,
    watch: bool,
) -> AppErrorResult<()> {
    let target_config = config.targets.get(target_name).ok_or_else(|| AppError {
        msg: "run configuration not found".to_owned(),
        exitcode: exitcode::USAGE,
    })?;

    let environment_variables = match &target_config.exec {
        Some(exec) => exec.all_vars().map_err(|err| AppError {
            msg: format!("failed to parse environment files: '{}'", err),
            exitcode: exitcode::DATAERR,
        })?,
        None => Default::default(),
    };

    if !check_if_run_needed(target_config.run_when.iter())? {
        return Err(AppError {
            msg: format!("skipping '{}'", target_name),
            exitcode: exitcode::OK,
        });
    }

    let run_hook = |name: &str| -> AppErrorResult<()> {
        let status = ProcessRunner {
            program: get_app_binary_path().to_str().unwrap().to_owned(),
            args: vec![String::from("run"), String::from(name)],
            vars: Default::default(),
            cwd: None,
        }
        .run_interactive()
        .map_err(|err| AppError {
            msg: format!("hook '{}' encountered an error: '{}'", name, err.msg),
            exitcode: exitcode::SOFTWARE,
        })?;
        if exitcode::is_error(status) {
            Err(AppError {
                msg: "stop hook run due to error".to_owned(),
                exitcode: status,
            })
        } else {
            Ok(())
        }
    };

    // TODO maybe polling can be improved to instead use `std::sync::Condvar` or something else?
    // Since this is expensive!
    let should_restart = Arc::new(AtomicBool::new(true));
    let should_cancel = Arc::new(AtomicBool::new(false));

    let mut debounced_watcher = new_debouncer(Duration::from_secs(2), {
        let should_restart = should_restart.clone();
        let should_cancel = should_cancel.clone();
        move |res| {
            if let Ok(_) = res {
                should_restart.store(true, Ordering::Relaxed);
            }
            should_cancel.store(true, Ordering::Relaxed);
        }
    })
    .map_err(|e| AppError {
        msg: format!(
            "an issue occurred while trying to construct the file watcher: '{:?}'",
            e,
        ),
        exitcode: exitcode::SOFTWARE,
    })?;
    if watch {
        for p in target_config.watch.paths.iter() {
            debounced_watcher
                .watcher()
                .watch(p, RecursiveMode::Recursive)
                .map_err(|e| AppError {
                    msg: format!(
                        "an issue occurred while trying to add a path to the watcher: '{:?}'",
                        e
                    ),
                    exitcode: exitcode::SOFTWARE,
                })?;
        }
    }

    loop {
        while should_restart.load(Ordering::Relaxed) {
            should_restart.store(false, Ordering::Relaxed);
            should_cancel.store(false, Ordering::Relaxed);

            for before in &target_config.before_hooks {
                run_hook(before)?;
            }

            if let Some(exec) = &target_config.exec {
                let mut args = exec.args.clone();
                args.extend(extra_args.clone());

                let status = ProcessRunner {
                    program: exec.program.clone(),
                    args,
                    vars: environment_variables.clone(),
                    cwd: exec.cwd.clone(),
                }
                .run_interactive_cancelable(&should_cancel)
                .unwrap_or_else(|err| err.handle());
                if status != exitcode::OK {
                    exit(status);
                }
            } else {
                log::info!("no program specified in target '{target_name}', skipping");
            }

            for after in &target_config.after_hooks {
                run_hook(after)?;
            }
        }
        if watch && should_cancel.load(Ordering::Relaxed) || !watch {
            break;
        }
        sleep(Duration::from_millis(1));
    }

    Ok(())
}

fn main() {
    let args = Args::parse();
    let log_level = match args.verbose_logging {
        true => log::Level::Debug,
        false => log::Level::Info,
    };
    simple_logger::init_with_level(log_level).expect("failed to setup logging");

    let lauched_from_dir = env::current_dir().unwrap_or_else(|_| {
        eprintln!("failed to get current working directory");
        exit(exitcode::OSERR);
    });
    let app_config_base = helpers::get_app_config_path().unwrap_or_else(|| {
        eprintln!("could not locate user home directory");
        exit(exitcode::NOINPUT);
    });

    let config_file_names = helpers::get_config_file_names(args.custom_filename);

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
        args::Command::Config { minimal } => command_config(config_path, selected_config, minimal),
        args::Command::Run {
            watch,
            target_name,
            extra_args,
        } => match &target_name {
            None => {
                println!("possible targets:");
                for (name, _) in selected_config.targets {
                    println!("  {}", name);
                }
                Err(AppError {
                    msg: String::from("target not specified"),
                    exitcode: exitcode::USAGE,
                })
            }
            Some(target_name) => command_run(selected_config, &target_name, extra_args, watch),
        },
    }
    .unwrap_or_else(|err| err.handle());
}
