use clap::{Parser, Subcommand};

/// A cross-platform task runner with zero setup
#[derive(Parser, Debug)]
#[command(name = "rnr")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Task to run
    #[arg(value_name = "TASK")]
    pub task: Option<String>,

    /// List all available tasks
    #[arg(short, long)]
    pub list: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Initialize rnr in the current directory
    Init,

    /// Upgrade rnr binaries to the latest version
    Upgrade,
}
