//! RNR configuration file management (.rnr/config.yaml)

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::platform::Platform;

/// The rnr configuration directory name
pub const RNR_DIR: &str = ".rnr";
/// The rnr configuration file name
pub const CONFIG_FILE: &str = "config.yaml";
/// The binary directory name
pub const BIN_DIR: &str = "bin";

/// RNR configuration stored in .rnr/config.yaml
#[derive(Debug, Serialize, Deserialize)]
pub struct RnrConfig {
    /// Version of rnr that created this config
    pub version: String,
    /// List of configured platform identifiers
    pub platforms: Vec<String>,
}

impl RnrConfig {
    /// Create a new config with the given platforms
    pub fn new(version: &str, platforms: &[Platform]) -> Self {
        Self {
            version: version.to_string(),
            platforms: platforms.iter().map(|p| p.id().to_string()).collect(),
        }
    }

    /// Load config from the default location
    pub fn load() -> Result<Self> {
        let path = config_path()?;
        Self::load_from(&path)
    }

    /// Load config from a specific path
    pub fn load_from(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config: {}", path.display()))?;
        let config: Self = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse config: {}", path.display()))?;
        Ok(config)
    }

    /// Save config to the default location
    pub fn save(&self) -> Result<()> {
        let path = config_path()?;
        self.save_to(&path)
    }

    /// Save config to a specific path
    pub fn save_to(&self, path: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }

        let content = serde_yaml::to_string(self).context("Failed to serialize config")?;
        fs::write(path, content)
            .with_context(|| format!("Failed to write config: {}", path.display()))?;
        Ok(())
    }

    /// Get the configured platforms
    pub fn get_platforms(&self) -> Vec<Platform> {
        self.platforms
            .iter()
            .filter_map(|id| Platform::from_id(id))
            .collect()
    }

    /// Add a platform to the config
    pub fn add_platform(&mut self, platform: Platform) {
        let id = platform.id().to_string();
        if !self.platforms.contains(&id) {
            self.platforms.push(id);
            self.platforms.sort();
        }
    }

    /// Remove a platform from the config
    pub fn remove_platform(&mut self, platform: Platform) {
        let id = platform.id();
        self.platforms.retain(|p| p != id);
    }

    /// Check if a platform is configured
    pub fn has_platform(&self, platform: Platform) -> bool {
        self.platforms.contains(&platform.id().to_string())
    }
}

/// Get the path to .rnr directory
pub fn rnr_dir() -> Result<PathBuf> {
    let current = std::env::current_dir().context("Failed to get current directory")?;
    Ok(current.join(RNR_DIR))
}

/// Get the path to .rnr/config.yaml
pub fn config_path() -> Result<PathBuf> {
    Ok(rnr_dir()?.join(CONFIG_FILE))
}

/// Get the path to .rnr/bin
pub fn bin_dir() -> Result<PathBuf> {
    Ok(rnr_dir()?.join(BIN_DIR))
}

/// Check if rnr is already initialized in the current directory
pub fn is_initialized() -> Result<bool> {
    let path = config_path()?;
    Ok(path.exists())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_roundtrip() {
        let platforms = vec![Platform::LinuxAmd64, Platform::MacosArm64];
        let config = RnrConfig::new("0.1.0", &platforms);

        let yaml = serde_yaml::to_string(&config).unwrap();
        let parsed: RnrConfig = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(parsed.version, "0.1.0");
        assert_eq!(parsed.platforms.len(), 2);
    }

    #[test]
    fn test_add_remove_platform() {
        let mut config = RnrConfig::new("0.1.0", &[Platform::LinuxAmd64]);

        config.add_platform(Platform::MacosArm64);
        assert!(config.has_platform(Platform::MacosArm64));

        config.remove_platform(Platform::LinuxAmd64);
        assert!(!config.has_platform(Platform::LinuxAmd64));
    }
}
