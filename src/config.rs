use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// The main configuration file name
pub const CONFIG_FILE: &str = "rnr.yaml";

/// Represents a single task in the configuration
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum TaskDef {
    /// Shorthand: just a command string
    Shorthand(String),
    /// Full task definition
    Full(Task),
}

/// Full task definition with all properties
#[derive(Debug, Deserialize)]
pub struct Task {
    /// Human-readable description
    pub description: Option<String>,

    /// Working directory (relative to project root)
    pub dir: Option<String>,

    /// Environment variables
    pub env: Option<HashMap<String, String>>,

    /// Shell command to execute
    pub cmd: Option<String>,

    /// Another task to run
    pub task: Option<String>,

    /// Sequential steps
    pub steps: Option<Vec<Step>>,
}

/// A step in a task
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Step {
    /// Simple step with cmd/task/dir
    Simple(StepDef),
    /// Parallel execution block
    Parallel { parallel: Vec<StepDef> },
}

/// Definition of a single step
#[derive(Debug, Deserialize)]
pub struct StepDef {
    /// Working directory
    pub dir: Option<String>,

    /// Shell command
    pub cmd: Option<String>,

    /// Task to run
    pub task: Option<String>,
}

/// The complete rnr.yaml configuration
#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(flatten)]
    pub tasks: HashMap<String, TaskDef>,
}

impl Config {
    /// Load configuration from the default file
    pub fn load() -> Result<Self> {
        let path = find_config_file()?;
        Self::load_from(&path)
    }

    /// Load configuration from a specific path
    pub fn load_from(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let config: Config = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

        Ok(config)
    }

    /// Get a task by name
    pub fn get_task(&self, name: &str) -> Option<&TaskDef> {
        self.tasks.get(name)
    }

    /// List all task names
    pub fn task_names(&self) -> Vec<&str> {
        let mut names: Vec<_> = self.tasks.keys().map(|s| s.as_str()).collect();
        names.sort();
        names
    }
}

/// Find the config file by walking up from the current directory
pub fn find_config_file() -> Result<PathBuf> {
    let current_dir = std::env::current_dir().context("Failed to get current directory")?;

    let mut dir = current_dir.as_path();
    loop {
        let config_path = dir.join(CONFIG_FILE);
        if config_path.exists() {
            return Ok(config_path);
        }

        match dir.parent() {
            Some(parent) => dir = parent,
            None => break,
        }
    }

    anyhow::bail!(
        "No {} found in current directory or any parent directory",
        CONFIG_FILE
    )
}

/// Get the project root (directory containing rnr.yaml)
pub fn project_root() -> Result<PathBuf> {
    let config_path = find_config_file()?;
    config_path
        .parent()
        .map(|p| p.to_path_buf())
        .context("Config file has no parent directory")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_shorthand() {
        let yaml = r#"
build: cargo build --release
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert!(matches!(
            config.get_task("build"),
            Some(TaskDef::Shorthand(_))
        ));
    }

    #[test]
    fn test_parse_full_task() {
        let yaml = r#"
build:
  description: Build the project
  cmd: cargo build --release
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert!(matches!(config.get_task("build"), Some(TaskDef::Full(_))));
    }
}
