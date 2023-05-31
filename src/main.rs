use std::{fs, process::Command};

use args::Args;
use clap::Parser;
use config::Config;

mod args;
mod config;
mod helpers;

fn main() {
    let args = Args::parse();
    let run_config: Config =
        serde_yaml::from_str(&fs::read_to_string(".run.yml").unwrap()).unwrap();
    match args.command {
        args::Command::Run {
            config_name,
            extra_args,
        } => {
            let run_config = run_config.configurations.get(&config_name).unwrap();
            let file_envs = helpers::read_env_files(&run_config.env_file).unwrap();

            let mut command = Command::new(&run_config.program);

            if let Some(cwd) = &run_config.cwd {
                command.current_dir(&cwd);
            }

            command
                .envs(file_envs)
                .envs(&run_config.env)
                .args(&run_config.args)
                .args(&extra_args)
                .spawn()
                .unwrap()
                .wait()
                .unwrap();
        }
    }
}
