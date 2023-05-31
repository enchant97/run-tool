use clap::{Parser, Subcommand};

#[derive(Subcommand, Debug)]
pub enum Command {
    Run {
        #[arg(name = "config name", help = "name of config to run")]
        config_name: String,
        #[arg(name = "args", last = true, allow_hyphen_values = true)]
        extra_args: Vec<String>,
    },
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}
