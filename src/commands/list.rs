use anyhow::Result;

use crate::config::{Config, TaskDef};

/// Run the list command
pub fn run() -> Result<()> {
    let config = Config::load()?;

    println!("\nAvailable tasks:\n");

    let task_names = config.task_names();

    if task_names.is_empty() {
        println!("  No tasks defined in rnr.yaml");
        return Ok(());
    }

    // Find the longest task name for alignment
    let max_len = task_names.iter().map(|n| n.len()).max().unwrap_or(0);

    for name in task_names {
        let description = get_task_description(&config, name);
        match description {
            Some(desc) => println!("  {:<width$}  {}", name, desc, width = max_len),
            None => println!("  {}", name),
        }
    }

    println!();
    Ok(())
}

/// Get the description for a task, if any
fn get_task_description(config: &Config, name: &str) -> Option<String> {
    match config.get_task(name)? {
        TaskDef::Shorthand(_) => None,
        TaskDef::Full(task) => task.description.clone(),
    }
}
