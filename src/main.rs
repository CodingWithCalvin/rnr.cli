mod cli;
mod commands;
mod config;
mod runner;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Init) => commands::init::run()?,
        Some(Command::Upgrade) => commands::upgrade::run()?,
        None => {
            if cli.list {
                commands::list::run()?;
            } else if let Some(task_name) = cli.task {
                runner::run_task(&task_name)?;
            } else {
                // No task specified, show help or list
                commands::list::run()?;
            }
        }
    }

    Ok(())
}
