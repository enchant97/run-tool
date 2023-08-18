use clap::{Parser, Subcommand};

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Run a configuration
    #[command()]
    Run {
        /// Name of target to run
        #[arg(name = "target name")]
        target_name: String,
        /// Use config from global
        #[arg(short, long)]
        global: bool,
        /// Extra arguments to pass to app
        #[arg(name = "args", last = true, allow_hyphen_values = true)]
        extra_args: Vec<String>,
    },
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}
