use std::path::PathBuf;

use clap::{Parser, Subcommand};

fn path_only_filename(p: &str) -> Result<PathBuf, String> {
    let filename = p.parse::<PathBuf>().map_err(|_| "not a valid filename")?;
    if let Some(parent) = filename.parent() {
        if parent != PathBuf::new() {
            return Err("not a valid filename")?;
        }
    }
    Ok(filename)
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Show the loaded configuration
    Config {
        /// Reduce the displayed information
        #[arg(short, long)]
        minimal: bool,
    },
    /// Run a configuration
    #[command()]
    Run {
        /// Name of target to run
        #[arg(name = "target name")]
        target_name: String,
        /// Extra arguments to pass to app
        #[arg(name = "args", last = true, allow_hyphen_values = true)]
        extra_args: Vec<String>,
    },
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about)]
pub struct Args {
    /// Use user's global configuration
    #[arg(short = 'g', long = "global")]
    pub use_global_config: bool,
    /// Search for configuration from different path
    #[arg(name = "search path", short = 'p', long = "path")]
    pub custom_path: Option<PathBuf>,
    /// Custom configuration filename
    #[arg(name = "filename", short = 'f', long = "file", value_parser = path_only_filename)]
    pub custom_filename: Option<PathBuf>,
    #[command(subcommand)]
    pub command: Command,
}
