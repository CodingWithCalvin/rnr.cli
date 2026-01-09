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
#[serde(deny_unknown_fields)]
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

    // ==================== Shorthand Parsing ====================

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
    fn test_parse_shorthand_value() {
        let yaml = "build: cargo build --release";
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        if let Some(TaskDef::Shorthand(cmd)) = config.get_task("build") {
            assert_eq!(cmd, "cargo build --release");
        } else {
            panic!("Expected shorthand task");
        }
    }

    #[test]
    fn test_parse_multiple_shorthand() {
        let yaml = r#"
build: cargo build
test: cargo test
lint: cargo clippy
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.tasks.len(), 3);
        assert!(config.get_task("build").is_some());
        assert!(config.get_task("test").is_some());
        assert!(config.get_task("lint").is_some());
    }

    // ==================== Full Task Parsing ====================

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

    #[test]
    fn test_parse_full_task_with_description() {
        let yaml = r#"
build:
  description: Build the project for production
  cmd: cargo build --release
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        if let Some(TaskDef::Full(task)) = config.get_task("build") {
            assert_eq!(
                task.description,
                Some("Build the project for production".to_string())
            );
        } else {
            panic!("Expected full task");
        }
    }

    #[test]
    fn test_parse_full_task_with_dir() {
        let yaml = r#"
build:
  dir: src/subproject
  cmd: cargo build
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        if let Some(TaskDef::Full(task)) = config.get_task("build") {
            assert_eq!(task.dir, Some("src/subproject".to_string()));
        } else {
            panic!("Expected full task");
        }
    }

    #[test]
    fn test_parse_full_task_with_env() {
        let yaml = r#"
build:
  env:
    NODE_ENV: production
    DEBUG: "false"
  cmd: npm run build
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        if let Some(TaskDef::Full(task)) = config.get_task("build") {
            let env = task.env.as_ref().unwrap();
            assert_eq!(env.get("NODE_ENV"), Some(&"production".to_string()));
            assert_eq!(env.get("DEBUG"), Some(&"false".to_string()));
        } else {
            panic!("Expected full task");
        }
    }

    #[test]
    fn test_parse_full_task_with_task_delegation() {
        let yaml = r#"
build:
  dir: services/api
  task: build
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        if let Some(TaskDef::Full(task)) = config.get_task("build") {
            assert_eq!(task.task, Some("build".to_string()));
            assert_eq!(task.dir, Some("services/api".to_string()));
        } else {
            panic!("Expected full task");
        }
    }

    // ==================== Steps Parsing ====================

    #[test]
    fn test_parse_sequential_steps() {
        let yaml = r#"
ci:
  steps:
    - cmd: echo "Step 1"
    - cmd: echo "Step 2"
    - cmd: echo "Step 3"
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        if let Some(TaskDef::Full(task)) = config.get_task("ci") {
            let steps = task.steps.as_ref().unwrap();
            assert_eq!(steps.len(), 3);
        } else {
            panic!("Expected full task with steps");
        }
    }

    #[test]
    fn test_parse_steps_with_task_delegation() {
        let yaml = r#"
ci:
  steps:
    - task: lint
    - task: test
    - task: build
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        if let Some(TaskDef::Full(task)) = config.get_task("ci") {
            let steps = task.steps.as_ref().unwrap();
            assert_eq!(steps.len(), 3);
            if let Step::Simple(step) = &steps[0] {
                assert_eq!(step.task, Some("lint".to_string()));
            } else {
                panic!("Expected simple step");
            }
        } else {
            panic!("Expected full task with steps");
        }
    }

    #[test]
    fn test_parse_steps_with_dir() {
        let yaml = r#"
build-all:
  steps:
    - dir: services/api
      cmd: cargo build
    - dir: services/web
      cmd: npm run build
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        if let Some(TaskDef::Full(task)) = config.get_task("build-all") {
            let steps = task.steps.as_ref().unwrap();
            assert_eq!(steps.len(), 2);
            if let Step::Simple(step) = &steps[0] {
                assert_eq!(step.dir, Some("services/api".to_string()));
                assert_eq!(step.cmd, Some("cargo build".to_string()));
            } else {
                panic!("Expected simple step");
            }
        } else {
            panic!("Expected full task with steps");
        }
    }

    // ==================== Parallel Parsing ====================

    #[test]
    fn test_parse_parallel_block() {
        let yaml = r#"
build-all:
  steps:
    - parallel:
        - cmd: cargo build
        - cmd: npm run build
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        if let Some(TaskDef::Full(task)) = config.get_task("build-all") {
            let steps = task.steps.as_ref().unwrap();
            assert_eq!(steps.len(), 1);
            if let Step::Parallel { parallel } = &steps[0] {
                assert_eq!(parallel.len(), 2);
            } else {
                panic!("Expected parallel step");
            }
        } else {
            panic!("Expected full task with steps");
        }
    }

    #[test]
    fn test_parse_mixed_sequential_and_parallel() {
        let yaml = r#"
deploy:
  steps:
    - cmd: echo "Starting"
    - parallel:
        - task: build-api
        - task: build-web
    - cmd: echo "Done"
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        if let Some(TaskDef::Full(task)) = config.get_task("deploy") {
            let steps = task.steps.as_ref().unwrap();
            assert_eq!(steps.len(), 3);
            assert!(matches!(&steps[0], Step::Simple(_)));
            assert!(matches!(&steps[1], Step::Parallel { .. }));
            assert!(matches!(&steps[2], Step::Simple(_)));
        } else {
            panic!("Expected full task with steps");
        }
    }

    // ==================== Task Names ====================

    #[test]
    fn test_task_names_sorted() {
        let yaml = r#"
zebra: echo zebra
alpha: echo alpha
middle: echo middle
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        let names = config.task_names();
        assert_eq!(names, vec!["alpha", "middle", "zebra"]);
    }

    #[test]
    fn test_task_names_empty() {
        let yaml = "{}";
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        let names = config.task_names();
        assert!(names.is_empty());
    }

    // ==================== Get Task ====================

    #[test]
    fn test_get_task_exists() {
        let yaml = "build: cargo build";
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert!(config.get_task("build").is_some());
    }

    #[test]
    fn test_get_task_not_exists() {
        let yaml = "build: cargo build";
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert!(config.get_task("nonexistent").is_none());
    }

    // ==================== Mixed Tasks ====================

    #[test]
    fn test_parse_mixed_shorthand_and_full() {
        let yaml = r#"
lint: cargo clippy
build:
  description: Build the project
  cmd: cargo build --release
test: cargo test
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert!(matches!(
            config.get_task("lint"),
            Some(TaskDef::Shorthand(_))
        ));
        assert!(matches!(config.get_task("build"), Some(TaskDef::Full(_))));
        assert!(matches!(
            config.get_task("test"),
            Some(TaskDef::Shorthand(_))
        ));
    }

    // ==================== Complex Config ====================

    #[test]
    fn test_parse_complex_config() {
        let yaml = r#"
lint: cargo clippy

build-api:
  description: Build API service
  dir: services/api
  env:
    RUST_LOG: info
  cmd: cargo build --release

build-web:
  description: Build web frontend
  dir: services/web
  env:
    NODE_ENV: production
  cmd: npm run build

build-all:
  description: Build everything
  steps:
    - cmd: echo "Starting builds..."
    - parallel:
        - task: build-api
        - task: build-web
    - cmd: echo "All builds complete"

deploy:
  description: Deploy to production
  steps:
    - task: build-all
    - cmd: ./scripts/deploy.sh
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.tasks.len(), 5);

        // Check lint (shorthand)
        if let Some(TaskDef::Shorthand(cmd)) = config.get_task("lint") {
            assert_eq!(cmd, "cargo clippy");
        } else {
            panic!("Expected shorthand lint task");
        }

        // Check build-api (full with env)
        if let Some(TaskDef::Full(task)) = config.get_task("build-api") {
            assert_eq!(task.description, Some("Build API service".to_string()));
            assert_eq!(task.dir, Some("services/api".to_string()));
            assert!(task.env.is_some());
        } else {
            panic!("Expected full build-api task");
        }

        // Check build-all (steps with parallel)
        if let Some(TaskDef::Full(task)) = config.get_task("build-all") {
            let steps = task.steps.as_ref().unwrap();
            assert_eq!(steps.len(), 3);
        } else {
            panic!("Expected full build-all task");
        }
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_parse_task_with_colons_in_name() {
        let yaml = r#"
"api:build": cargo build
"web:build": npm run build
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert!(config.get_task("api:build").is_some());
        assert!(config.get_task("web:build").is_some());
    }

    #[test]
    fn test_parse_empty_env() {
        let yaml = r#"
build:
  env: {}
  cmd: cargo build
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        if let Some(TaskDef::Full(task)) = config.get_task("build") {
            assert!(task.env.as_ref().unwrap().is_empty());
        } else {
            panic!("Expected full task");
        }
    }
}
