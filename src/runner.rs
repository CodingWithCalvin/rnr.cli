use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use crate::config::{Config, Step, StepDef, Task, TaskDef};

/// Run a task by name
pub fn run_task(task_name: &str) -> Result<()> {
    let config = Config::load()?;
    let project_root = crate::config::project_root()?;

    let task = config
        .get_task(task_name)
        .with_context(|| format!("Task '{}' not found", task_name))?;

    execute_task_def(task, &project_root, &config)
}

/// Execute a task definition
fn execute_task_def(task_def: &TaskDef, project_root: &Path, config: &Config) -> Result<()> {
    match task_def {
        TaskDef::Shorthand(cmd) => execute_command(cmd, project_root, &HashMap::new()),
        TaskDef::Full(task) => execute_full_task(task, project_root, config),
    }
}

/// Execute a full task definition
fn execute_full_task(task: &Task, project_root: &Path, config: &Config) -> Result<()> {
    let work_dir = match &task.dir {
        Some(dir) => project_root.join(dir),
        None => project_root.to_path_buf(),
    };

    let env = task.env.clone().unwrap_or_default();

    // If task has steps, execute them
    if let Some(steps) = &task.steps {
        for step in steps {
            execute_step(step, &work_dir, &env, config)?;
        }
        return Ok(());
    }

    // If task delegates to another task
    if let Some(task_name) = &task.task {
        // If dir is specified, look for rnr.yaml in that directory
        if task.dir.is_some() {
            let nested_config_path = work_dir.join(crate::config::CONFIG_FILE);
            if nested_config_path.exists() {
                let nested_config = Config::load_from(&nested_config_path)?;
                let nested_task = nested_config.get_task(task_name).with_context(|| {
                    format!(
                        "Task '{}' not found in {}",
                        task_name,
                        nested_config_path.display()
                    )
                })?;
                return execute_task_def(nested_task, &work_dir, &nested_config);
            }
        }

        // Otherwise, look in current config
        let target_task = config
            .get_task(task_name)
            .with_context(|| format!("Task '{}' not found", task_name))?;
        return execute_task_def(target_task, project_root, config);
    }

    // Execute command if present
    if let Some(cmd) = &task.cmd {
        return execute_command(cmd, &work_dir, &env);
    }

    anyhow::bail!("Task has no cmd, task, or steps defined")
}

/// Execute a single step
fn execute_step(
    step: &Step,
    default_dir: &Path,
    default_env: &HashMap<String, String>,
    config: &Config,
) -> Result<()> {
    match step {
        Step::Simple(step_def) => execute_step_def(step_def, default_dir, default_env, config),
        Step::Parallel { parallel } => execute_parallel(parallel, default_dir, default_env, config),
    }
}

/// Execute steps in parallel using scoped threads
fn execute_parallel(
    steps: &[StepDef],
    default_dir: &Path,
    default_env: &HashMap<String, String>,
    config: &Config,
) -> Result<()> {
    use std::sync::Mutex;
    use std::thread;

    let errors: Mutex<Vec<anyhow::Error>> = Mutex::new(Vec::new());

    thread::scope(|s| {
        for step_def in steps {
            s.spawn(|| {
                if let Err(e) = execute_step_def(step_def, default_dir, default_env, config) {
                    errors.lock().unwrap().push(e);
                }
            });
        }
    });

    let errors = errors.into_inner().unwrap();
    if errors.is_empty() {
        Ok(())
    } else {
        // Combine all errors into one message
        let error_messages: Vec<String> = errors.iter().map(|e| format!("  - {}", e)).collect();
        anyhow::bail!(
            "Parallel execution failed with {} error(s):\n{}",
            error_messages.len(),
            error_messages.join("\n")
        )
    }
}

/// Execute a step definition
fn execute_step_def(
    step_def: &StepDef,
    default_dir: &Path,
    default_env: &HashMap<String, String>,
    config: &Config,
) -> Result<()> {
    let work_dir = match &step_def.dir {
        Some(dir) => {
            let project_root = crate::config::project_root()?;
            project_root.join(dir)
        }
        None => default_dir.to_path_buf(),
    };

    // If step delegates to a task
    if let Some(task_name) = &step_def.task {
        // Check for nested rnr.yaml if dir is specified
        if step_def.dir.is_some() {
            let nested_config_path = work_dir.join(crate::config::CONFIG_FILE);
            if nested_config_path.exists() {
                let nested_config = Config::load_from(&nested_config_path)?;
                let nested_task = nested_config.get_task(task_name).with_context(|| {
                    format!(
                        "Task '{}' not found in {}",
                        task_name,
                        nested_config_path.display()
                    )
                })?;
                return execute_task_def(nested_task, &work_dir, &nested_config);
            }
        }

        let target_task = config
            .get_task(task_name)
            .with_context(|| format!("Task '{}' not found", task_name))?;
        let project_root = crate::config::project_root()?;
        return execute_task_def(target_task, &project_root, config);
    }

    // Execute command
    if let Some(cmd) = &step_def.cmd {
        return execute_command(cmd, &work_dir, default_env);
    }

    anyhow::bail!("Step has no cmd or task defined")
}

/// Execute a shell command
fn execute_command(cmd: &str, work_dir: &Path, env: &HashMap<String, String>) -> Result<()> {
    println!("$ {}", cmd);

    let mut command = if cfg!(target_os = "windows") {
        let mut c = Command::new("cmd");
        c.args(["/C", cmd]);
        c
    } else {
        let mut c = Command::new("sh");
        c.args(["-c", cmd]);
        c
    };

    command.current_dir(work_dir);
    command.envs(env);

    let status = command
        .status()
        .with_context(|| format!("Failed to execute command: {}", cmd))?;

    if !status.success() {
        let code = status.code().unwrap_or(1);
        anyhow::bail!("Command failed with exit code {}", code);
    }

    Ok(())
}
