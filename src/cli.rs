use clap::{Args, Parser, Subcommand};

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
    Init(InitArgs),

    /// Upgrade rnr binaries to the latest version
    Upgrade,
}

#[derive(Args, Debug)]
pub struct InitArgs {
    /// Comma-separated list of platforms (e.g., linux-amd64,macos-arm64,windows-amd64)
    #[arg(long, value_delimiter = ',')]
    pub platforms: Option<Vec<String>>,

    /// Include all available platforms
    #[arg(long, conflicts_with_all = ["platforms", "current_platform_only"])]
    pub all_platforms: bool,

    /// Only include the current platform
    #[arg(long, conflicts_with_all = ["platforms", "all_platforms"])]
    pub current_platform_only: bool,

    /// Add a platform to existing setup
    #[arg(long, conflicts_with_all = ["platforms", "all_platforms", "current_platform_only", "remove_platform"])]
    pub add_platform: Option<String>,

    /// Remove a platform from existing setup
    #[arg(long, conflicts_with_all = ["platforms", "all_platforms", "current_platform_only", "add_platform"])]
    pub remove_platform: Option<String>,

    /// Show currently configured platforms
    #[arg(long)]
    pub show_platforms: bool,

    /// Skip git repository root check
    #[arg(long)]
    pub force: bool,
}
