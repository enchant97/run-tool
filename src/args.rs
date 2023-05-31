use clap::{Parser, Subcommand};

#[derive(Subcommand, Debug)]
pub enum Command {
    #[command(about = "Run a config")]
    Run {
        #[arg(name = "config name", help = "Name of config to run")]
        config_name: String,
        #[arg(name = "args", help = "Extra arguments to pass to app", last = true, allow_hyphen_values = true)]
        extra_args: Vec<String>,
    },
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = "A tool to aid developers in running their projects")]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}
