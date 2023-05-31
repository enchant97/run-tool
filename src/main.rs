use std::process::{exit, Command};

use args::Args;
use clap::Parser;
use config::Config;

mod args;
mod config;
mod helpers;

fn main() {
    let args = Args::parse();
    let run_config_raw =
        match helpers::read_with_fallbacks(&[".run-tool.yaml".into(), ".run-tool.yml".into()]) {
            Some(v) => v,
            None => {
                eprintln!("could not read run configuration");
                exit(1);
            }
        };
    let run_config: Config = match serde_yaml::from_str(&run_config_raw) {
        Ok(v) => v,
        Err(_) => {
            eprintln!("failed to parse configuration");
            exit(1);
        }
    };
    match args.command {
        args::Command::Run {
            config_name,
            extra_args,
        } => {
            let run_config = match run_config.configurations.get(&config_name) {
                Some(v) => v,
                None => {
                    eprintln!("run configuration not found");
                    exit(1);
                }
            };
            let file_envs = match helpers::read_env_files(&run_config.env_files()) {
                Ok(v) => v,
                Err(_) => {
                    eprintln!("failed to parse environment files");
                    exit(1);
                }
            };

            let mut command = Command::new(&run_config.program);

            if let Some(cwd) = &run_config.cwd {
                command.current_dir(cwd);
            }

            let mut process = match command
                .envs(file_envs)
                .envs(&run_config.env)
                .args(&run_config.args)
                .args(&extra_args)
                .spawn()
            {
                Ok(v) => v,
                Err(err) => {
                    eprintln!("{}", err);
                    exit(1);
                }
            };
            exit(
                process
                    .wait()
                    .expect("failed to execute")
                    .code()
                    .unwrap_or_default(),
            );
        }
    }
}
