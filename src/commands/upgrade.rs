//! Upgrade rnr binaries to the latest version

use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

use crate::platform::Platform;
use crate::rnr_config::RnrConfig;

/// GitHub repository for releases
const GITHUB_REPO: &str = "CodingWithCalvin/rnr.cli";

/// Run the upgrade command
pub fn run() -> Result<()> {
    let rnr_dir = find_rnr_dir()?;
    let bin_dir = rnr_dir.join("bin");

    if !bin_dir.exists() {
        anyhow::bail!("rnr is not initialized. Run 'rnr init' first.");
    }

    // Load current config
    let config_path = rnr_dir.join("config.yaml");
    let mut config = RnrConfig::load_from(&config_path)?;
    let platforms = config.get_platforms();

    if platforms.is_empty() {
        anyhow::bail!("No platforms configured. Run 'rnr init' to set up platforms.");
    }

    println!("Checking for updates...\n");
    println!("  Current version: v{}", config.version);

    #[cfg(feature = "network")]
    {
        upgrade_binaries(&bin_dir, &mut config, &config_path, &platforms)?;
    }

    #[cfg(not(feature = "network"))]
    {
        println!("\nNetwork feature is disabled. Cannot check for updates.");
        println!("Please manually update binaries in .rnr/bin/");
    }

    Ok(())
}

/// Find the .rnr directory by walking up from current directory
fn find_rnr_dir() -> Result<PathBuf> {
    let current_dir = std::env::current_dir().context("Failed to get current directory")?;

    let mut dir = current_dir.as_path();
    loop {
        let rnr_path = dir.join(".rnr");
        if rnr_path.exists() && rnr_path.is_dir() {
            return Ok(rnr_path);
        }

        match dir.parent() {
            Some(parent) => dir = parent,
            None => break,
        }
    }

    anyhow::bail!("No .rnr directory found. Run 'rnr init' first.")
}

/// Upgrade binaries to the latest version
#[cfg(feature = "network")]
fn upgrade_binaries(
    bin_dir: &std::path::Path,
    config: &mut RnrConfig,
    config_path: &std::path::Path,
    platforms: &[Platform],
) -> Result<()> {
    // Get latest release info from GitHub
    let latest_version = get_latest_version()?;
    println!("  Latest version:  v{}", latest_version);

    // Compare versions
    if !is_newer_version(&config.version, &latest_version) {
        println!("\nYou're already on the latest version!");
        return Ok(());
    }

    println!("\nUpgrading to v{}...\n", latest_version);

    // Download new binaries for all configured platforms
    for platform in platforms {
        print!("  Downloading {}...", platform.binary_name());
        let binary_path = bin_dir.join(platform.binary_name());
        download_binary(*platform, &latest_version, &binary_path)?;
        println!(" done");
    }

    // Update config version
    config.version = latest_version.clone();
    config.save_to(config_path)?;

    println!("\nUpgrade complete! Now running v{}", latest_version);

    Ok(())
}

/// Get the latest release version from GitHub
#[cfg(feature = "network")]
fn get_latest_version() -> Result<String> {
    let url = format!(
        "https://api.github.com/repos/{}/releases/latest",
        GITHUB_REPO
    );

    let client = reqwest::blocking::Client::builder()
        .user_agent("rnr-cli")
        .build()
        .context("Failed to create HTTP client")?;

    let response = client
        .get(&url)
        .send()
        .context("Failed to fetch latest release info")?;

    if !response.status().is_success() {
        if response.status().as_u16() == 404 {
            anyhow::bail!("No releases found. This may be the first version.");
        }
        anyhow::bail!(
            "Failed to fetch release info: HTTP {}",
            response.status().as_u16()
        );
    }

    let json: serde_json::Value = response
        .json()
        .context("Failed to parse release info as JSON")?;

    let tag = json["tag_name"]
        .as_str()
        .context("Release missing tag_name")?;

    // Strip 'v' prefix if present
    let version = tag.strip_prefix('v').unwrap_or(tag);
    Ok(version.to_string())
}

/// Download a binary for a specific platform and version
#[cfg(feature = "network")]
fn download_binary(platform: Platform, version: &str, dest: &std::path::Path) -> Result<()> {
    let url = format!(
        "https://github.com/{}/releases/download/v{}/{}",
        GITHUB_REPO,
        version,
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

/// Compare semantic versions, returns true if latest is newer than current
#[cfg(feature = "network")]
fn is_newer_version(current: &str, latest: &str) -> bool {
    let parse_version = |v: &str| -> (u32, u32, u32) {
        let parts: Vec<&str> = v.split('.').collect();
        let major = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
        let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        let patch = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
        (major, minor, patch)
    };

    let (cur_major, cur_minor, cur_patch) = parse_version(current);
    let (lat_major, lat_minor, lat_patch) = parse_version(latest);

    if lat_major > cur_major {
        return true;
    }
    if lat_major == cur_major && lat_minor > cur_minor {
        return true;
    }
    if lat_major == cur_major && lat_minor == cur_minor && lat_patch > cur_patch {
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "network")]
    fn test_version_comparison() {
        assert!(is_newer_version("0.1.0", "0.2.0"));
        assert!(is_newer_version("0.1.0", "1.0.0"));
        assert!(is_newer_version("0.1.0", "0.1.1"));
        assert!(!is_newer_version("0.2.0", "0.1.0"));
        assert!(!is_newer_version("1.0.0", "0.9.0"));
        assert!(!is_newer_version("0.1.0", "0.1.0"));
    }
}
