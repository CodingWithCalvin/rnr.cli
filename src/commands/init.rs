use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::config::CONFIG_FILE;

/// Directory for rnr binaries
const RNR_DIR: &str = ".rnr";
const BIN_DIR: &str = "bin";

/// Run the init command
pub fn run() -> Result<()> {
    let current_dir = std::env::current_dir().context("Failed to get current directory")?;

    // Check if already initialized
    let rnr_dir = current_dir.join(RNR_DIR);
    if rnr_dir.exists() {
        println!("rnr is already initialized in this directory");
        return Ok(());
    }

    println!("Initializing rnr...");

    // Create .rnr/bin directory
    let bin_dir = rnr_dir.join(BIN_DIR);
    fs::create_dir_all(&bin_dir).context("Failed to create .rnr/bin directory")?;
    println!("  Created {}/{}", RNR_DIR, BIN_DIR);

    // Download binaries for all platforms
    #[cfg(feature = "network")]
    {
        download_binaries(&bin_dir)?;
    }

    #[cfg(not(feature = "network"))]
    {
        println!("  Skipping binary download (network feature disabled)");
        println!("  Please manually copy binaries to {}/{}", RNR_DIR, BIN_DIR);
    }

    // Create wrapper scripts
    create_wrapper_scripts(&current_dir)?;

    // Create starter rnr.yaml if it doesn't exist
    let config_path = current_dir.join(CONFIG_FILE);
    if !config_path.exists() {
        create_starter_config(&config_path)?;
    } else {
        println!("  {} already exists, skipping", CONFIG_FILE);
    }

    println!("\nrnr initialized successfully!");
    println!("\nNext steps:");
    println!("  1. Edit {} to define your tasks", CONFIG_FILE);
    println!("  2. Run ./rnr --list to see available tasks");
    println!("  3. Run ./rnr <task> to execute a task");
    println!("  4. Commit the .rnr directory and wrapper scripts to your repo");

    Ok(())
}

/// Download binaries for all platforms
#[cfg(feature = "network")]
fn download_binaries(bin_dir: &Path) -> Result<()> {
    // TODO: Implement actual binary downloads from rnr.dev
    // For now, just create placeholder files
    println!("  Downloading binaries...");
    println!("    TODO: Download from https://rnr.dev/bin/latest/");

    // Placeholder - in real implementation, download from server
    let platforms = ["rnr-linux", "rnr-macos", "rnr.exe"];
    for platform in platforms {
        let path = bin_dir.join(platform);
        fs::write(&path, "# placeholder binary\n")
            .with_context(|| format!("Failed to create {}", path.display()))?;
        println!("    Created {}", platform);
    }

    Ok(())
}

/// Create the wrapper scripts at the project root
fn create_wrapper_scripts(project_root: &Path) -> Result<()> {
    // Unix wrapper script
    let unix_script = r#"#!/bin/sh
exec "$(dirname "$0")/.rnr/bin/rnr-$(uname -s | tr A-Z a-z)" "$@"
"#;

    let unix_path = project_root.join("rnr");
    fs::write(&unix_path, unix_script).context("Failed to create rnr wrapper script")?;

    // Make executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&unix_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&unix_path, perms)?;
    }

    println!("  Created rnr (Unix wrapper)");

    // Windows wrapper script
    let windows_script = r#"@echo off
"%~dp0.rnr\bin\rnr.exe" %*
"#;

    let windows_path = project_root.join("rnr.cmd");
    fs::write(&windows_path, windows_script).context("Failed to create rnr.cmd wrapper script")?;
    println!("  Created rnr.cmd (Windows wrapper)");

    Ok(())
}

/// Create a starter rnr.yaml configuration
fn create_starter_config(path: &Path) -> Result<()> {
    let starter = r#"# rnr task definitions
# See https://github.com/CodingWithCalvin/rnr.cli for documentation

# Simple command (shorthand)
hello: echo "Hello from rnr!"

# Full task definition
build:
  description: Build the project
  cmd: echo "Add your build command here"

# Task with steps
ci:
  description: Run CI pipeline
  steps:
    - cmd: echo "Step 1: Lint"
    - cmd: echo "Step 2: Test"
    - cmd: echo "Step 3: Build"
"#;

    fs::write(path, starter).context("Failed to create rnr.yaml")?;
    println!("  Created {}", CONFIG_FILE);

    Ok(())
}
