//! Initialize rnr in the current directory

use anyhow::{bail, Context, Result};
use dialoguer::MultiSelect;
use std::fs;
use std::path::Path;

use crate::cli::InitArgs;
use crate::config::CONFIG_FILE;
use crate::platform::{format_size, total_size, Platform, ALL_PLATFORMS};
use crate::rnr_config::{bin_dir, is_initialized, RnrConfig};

/// Current rnr version
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Run the init command
pub fn run(args: &InitArgs) -> Result<()> {
    // Handle --show-platforms
    if args.show_platforms {
        return show_platforms();
    }

    // Handle --add-platform
    if let Some(platform_id) = &args.add_platform {
        return add_platform(platform_id);
    }

    // Handle --remove-platform
    if let Some(platform_id) = &args.remove_platform {
        return remove_platform(platform_id);
    }

    // Check if already initialized (for fresh init)
    if is_initialized()? {
        println!("rnr is already initialized in this directory.");
        println!("Use --add-platform or --remove-platform to modify platforms.");
        println!("Use --show-platforms to see configured platforms.");
        return Ok(());
    }

    // Error if not at git repo root (unless --force is used)
    if !args.force && !is_git_repo_root()? {
        bail!(
            "This directory does not appear to be a git repository root.\n\
             rnr is typically initialized at the root of a git repository.\n\
             Use --force to initialize anyway, or run from the directory containing your .git folder."
        );
    }

    // Determine platforms to install
    let platforms = select_platforms(args)?;

    if platforms.is_empty() {
        bail!("No platforms selected. At least one platform is required.");
    }

    // Perform initialization
    initialize(&platforms)
}

/// Check if the current directory is a git repository root
fn is_git_repo_root() -> Result<bool> {
    let current_dir = std::env::current_dir().context("Failed to get current directory")?;
    let git_dir = current_dir.join(".git");
    Ok(git_dir.exists())
}

/// Select platforms based on args or interactively
fn select_platforms(args: &InitArgs) -> Result<Vec<Platform>> {
    // --all-platforms
    if args.all_platforms {
        return Ok(ALL_PLATFORMS.to_vec());
    }

    // --current-platform-only
    if args.current_platform_only {
        let current = Platform::current()
            .context("Unable to detect current platform. Use --platforms to specify manually.")?;
        return Ok(vec![current]);
    }

    // --platforms list
    if let Some(platform_ids) = &args.platforms {
        let mut platforms = Vec::new();
        for id in platform_ids {
            let platform = Platform::from_id(id)
                .with_context(|| format!("Unknown platform: {}. Valid platforms: linux-amd64, macos-amd64, macos-arm64, windows-amd64, windows-arm64", id))?;
            platforms.push(platform);
        }
        return Ok(platforms);
    }

    // Interactive selection
    interactive_platform_select()
}

/// Interactive platform selection
fn interactive_platform_select() -> Result<Vec<Platform>> {
    let current = Platform::current();

    // Build items with size info
    let items: Vec<String> = ALL_PLATFORMS
        .iter()
        .map(|p| {
            let marker = if Some(*p) == current {
                " <- current"
            } else {
                ""
            };
            format!("{:<16} ({}){}", p.id(), p.size_display(), marker)
        })
        .collect();

    // Determine default selections (current platform pre-selected)
    let defaults: Vec<bool> = ALL_PLATFORMS.iter().map(|p| Some(*p) == current).collect();

    println!("\nWhich platforms should this project support?\n");

    let selections = MultiSelect::new()
        .items(&items)
        .defaults(&defaults)
        .interact()
        .context("Platform selection cancelled")?;

    let selected: Vec<Platform> = selections.iter().map(|&i| ALL_PLATFORMS[i]).collect();

    // Show total size
    let total = total_size(&selected);
    println!("\nSelected: {} total\n", format_size(total));

    Ok(selected)
}

/// Perform the actual initialization
fn initialize(platforms: &[Platform]) -> Result<()> {
    let current_dir = std::env::current_dir().context("Failed to get current directory")?;

    println!("Initializing rnr...\n");

    // Create .rnr/bin directory
    let bin_directory = bin_dir()?;
    fs::create_dir_all(&bin_directory).context("Failed to create .rnr/bin directory")?;
    println!("  Created .rnr/bin/");

    // Download binaries
    download_binaries(platforms, &bin_directory)?;

    // Save config
    let config = RnrConfig::new(VERSION, platforms);
    config.save()?;
    println!("  Created .rnr/config.yaml");

    // Create wrapper scripts
    create_wrapper_scripts(&current_dir)?;

    // Create starter rnr.yaml if it doesn't exist
    let task_config_path = current_dir.join(CONFIG_FILE);
    if !task_config_path.exists() {
        create_starter_config(&task_config_path)?;
    } else {
        println!("  {} already exists, skipping", CONFIG_FILE);
    }

    println!("\nrnr initialized successfully!");
    println!("\nConfigured platforms:");
    for p in platforms {
        println!("  - {}", p.id());
    }
    println!("\nNext steps:");
    println!("  1. Edit {} to define your tasks", CONFIG_FILE);
    println!("  2. Run ./rnr --list to see available tasks");
    println!("  3. Run ./rnr <task> to execute a task");
    println!("  4. Commit the .rnr directory and wrapper scripts to your repo");

    Ok(())
}

/// Download binaries for selected platforms
fn download_binaries(platforms: &[Platform], bin_directory: &Path) -> Result<()> {
    println!("  Downloading binaries...");

    for platform in platforms {
        let binary_path = bin_directory.join(platform.binary_name());

        #[cfg(feature = "network")]
        {
            download_binary(*platform, &binary_path)?;
        }

        #[cfg(not(feature = "network"))]
        {
            // Create placeholder for testing without network
            fs::write(
                &binary_path,
                format!("# placeholder for {}\n", platform.id()),
            )
            .with_context(|| format!("Failed to create {}", binary_path.display()))?;
        }

        println!(
            "    {} ({})",
            platform.binary_name(),
            platform.size_display()
        );
    }

    Ok(())
}

/// GitHub repository for releases
const GITHUB_REPO: &str = "CodingWithCalvin/rnr.cli";

/// Download a single binary from GitHub releases
#[cfg(feature = "network")]
fn download_binary(platform: Platform, dest: &Path) -> Result<()> {
    let url = format!(
        "https://github.com/{}/releases/latest/download/{}",
        GITHUB_REPO,
        platform.binary_name()
    );

    let client = reqwest::blocking::Client::builder()
        .user_agent("rnr-cli")
        .build()
        .context("Failed to create HTTP client")?;

    let response = client
        .get(&url)
        .send()
        .with_context(|| format!("Failed to download {}", platform.binary_name()))?;

    if !response.status().is_success() {
        anyhow::bail!(
            "Failed to download {}: HTTP {}",
            platform.binary_name(),
            response.status().as_u16()
        );
    }

    let bytes = response
        .bytes()
        .with_context(|| format!("Failed to read response for {}", platform.binary_name()))?;

    // Write to file
    fs::write(dest, &bytes).with_context(|| format!("Failed to write {}", dest.display()))?;

    // Make executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(dest)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(dest, perms)?;
    }

    Ok(())
}

/// Create the wrapper scripts at the project root
fn create_wrapper_scripts(project_root: &Path) -> Result<()> {
    // Unix wrapper script (smart detection)
    let unix_script = r#"#!/bin/sh
set -e

# Detect OS
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
EXT=""
case "$OS" in
  linux*) OS="linux" ;;
  darwin*) OS="macos" ;;
  mingw*|msys*|cygwin*) OS="windows"; EXT=".exe" ;;
  *) echo "Error: Unsupported OS: $OS" >&2; exit 1 ;;
esac

# Detect architecture
ARCH=$(uname -m)
case "$ARCH" in
  x86_64|amd64) ARCH="amd64" ;;
  arm64|aarch64) ARCH="arm64" ;;
  *) echo "Error: Unsupported architecture: $ARCH" >&2; exit 1 ;;
esac

BINARY="$(dirname "$0")/.rnr/bin/rnr-${OS}-${ARCH}${EXT}"

if [ ! -f "$BINARY" ]; then
  echo "Error: rnr is not configured for ${OS}-${ARCH}." >&2
  echo "Run 'rnr init --add-platform ${OS}-${ARCH}' to add support." >&2
  exit 1
fi

exec "$BINARY" "$@"
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

    // Windows wrapper script (smart detection)
    let windows_script = r#"@echo off
setlocal

:: Detect architecture
if "%PROCESSOR_ARCHITECTURE%"=="ARM64" (
  set "ARCH=arm64"
) else (
  set "ARCH=amd64"
)

set "BINARY=%~dp0.rnr\bin\rnr-windows-%ARCH%.exe"

if not exist "%BINARY%" (
  echo Error: rnr is not configured for windows-%ARCH%. >&2
  echo Run 'rnr init --add-platform windows-%ARCH%' to add support. >&2
  exit /b 1
)

"%BINARY%" %*
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

/// Show currently configured platforms
fn show_platforms() -> Result<()> {
    if !is_initialized()? {
        println!("rnr is not initialized in this directory.");
        println!("Run 'rnr init' to initialize.");
        return Ok(());
    }

    let config = RnrConfig::load()?;
    let platforms = config.get_platforms();

    println!("\nConfigured platforms:\n");
    let mut total: u64 = 0;
    for p in &platforms {
        println!("  {} ({})", p.id(), p.size_display());
        total += p.size_bytes();
    }
    println!("\nTotal: {}", format_size(total));

    Ok(())
}

/// Add a platform to existing setup
fn add_platform(platform_id: &str) -> Result<()> {
    if !is_initialized()? {
        bail!("rnr is not initialized. Run 'rnr init' first.");
    }

    let platform = Platform::from_id(platform_id).with_context(|| {
        format!(
            "Unknown platform: {}. Valid platforms: linux-amd64, macos-amd64, macos-arm64, windows-amd64, windows-arm64",
            platform_id
        )
    })?;

    let mut config = RnrConfig::load()?;

    if config.has_platform(platform) {
        println!("Platform {} is already configured.", platform_id);
        return Ok(());
    }

    // Download the binary
    let bin_directory = bin_dir()?;
    let binary_path = bin_directory.join(platform.binary_name());

    println!("Adding platform {}...", platform_id);

    #[cfg(feature = "network")]
    {
        download_binary(platform, &binary_path)?;
    }

    #[cfg(not(feature = "network"))]
    {
        fs::write(
            &binary_path,
            format!("# placeholder for {}\n", platform.id()),
        )?;
    }

    println!(
        "  Downloaded {} ({})",
        platform.binary_name(),
        platform.size_display()
    );

    // Update config
    config.add_platform(platform);
    config.save()?;
    println!("  Updated .rnr/config.yaml");

    println!("\nPlatform {} added successfully!", platform_id);

    Ok(())
}

/// Remove a platform from existing setup
fn remove_platform(platform_id: &str) -> Result<()> {
    if !is_initialized()? {
        bail!("rnr is not initialized. Run 'rnr init' first.");
    }

    let platform = Platform::from_id(platform_id).with_context(|| {
        format!(
            "Unknown platform: {}. Valid platforms: linux-amd64, macos-amd64, macos-arm64, windows-amd64, windows-arm64",
            platform_id
        )
    })?;

    let mut config = RnrConfig::load()?;

    if !config.has_platform(platform) {
        println!("Platform {} is not configured.", platform_id);
        return Ok(());
    }

    // Check if this is the last platform
    if config.get_platforms().len() == 1 {
        bail!("Cannot remove the last platform. At least one platform must be configured.");
    }

    println!("Removing platform {}...", platform_id);

    // Remove the binary
    let bin_directory = bin_dir()?;
    let binary_path = bin_directory.join(platform.binary_name());
    if binary_path.exists() {
        fs::remove_file(&binary_path)
            .with_context(|| format!("Failed to remove {}", binary_path.display()))?;
        println!("  Removed {}", platform.binary_name());
    }

    // Update config
    config.remove_platform(platform);
    config.save()?;
    println!("  Updated .rnr/config.yaml");

    println!("\nPlatform {} removed successfully!", platform_id);

    Ok(())
}
