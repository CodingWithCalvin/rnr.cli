//! Platform detection and definitions

use std::fmt;

/// All supported platforms
pub const ALL_PLATFORMS: &[Platform] = &[
    Platform::LinuxAmd64,
    Platform::MacosAmd64,
    Platform::MacosArm64,
    Platform::WindowsAmd64,
    Platform::WindowsArm64,
];

/// A supported platform
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Platform {
    LinuxAmd64,
    MacosAmd64,
    MacosArm64,
    WindowsAmd64,
    WindowsArm64,
}

impl Platform {
    /// Get the platform identifier string (e.g., "linux-amd64")
    pub fn id(&self) -> &'static str {
        match self {
            Platform::LinuxAmd64 => "linux-amd64",
            Platform::MacosAmd64 => "macos-amd64",
            Platform::MacosArm64 => "macos-arm64",
            Platform::WindowsAmd64 => "windows-amd64",
            Platform::WindowsArm64 => "windows-arm64",
        }
    }

    /// Get the binary filename for this platform
    pub fn binary_name(&self) -> &'static str {
        match self {
            Platform::LinuxAmd64 => "rnr-linux-amd64",
            Platform::MacosAmd64 => "rnr-macos-amd64",
            Platform::MacosArm64 => "rnr-macos-arm64",
            Platform::WindowsAmd64 => "rnr-windows-amd64.exe",
            Platform::WindowsArm64 => "rnr-windows-arm64.exe",
        }
    }

    /// Get the approximate binary size in bytes
    pub fn size_bytes(&self) -> u64 {
        match self {
            Platform::LinuxAmd64 => 760 * 1024,
            Platform::MacosAmd64 => 662 * 1024,
            Platform::MacosArm64 => 608 * 1024,
            Platform::WindowsAmd64 => 584 * 1024,
            Platform::WindowsArm64 => 528 * 1024,
        }
    }

    /// Get human-readable size string
    pub fn size_display(&self) -> String {
        let kb = self.size_bytes() / 1024;
        format!("{} KB", kb)
    }

    /// Parse a platform from its identifier string
    pub fn from_id(id: &str) -> Option<Platform> {
        match id {
            "linux-amd64" => Some(Platform::LinuxAmd64),
            "macos-amd64" => Some(Platform::MacosAmd64),
            "macos-arm64" => Some(Platform::MacosArm64),
            "windows-amd64" => Some(Platform::WindowsAmd64),
            "windows-arm64" => Some(Platform::WindowsArm64),
            _ => None,
        }
    }

    /// Detect the current platform
    pub fn current() -> Option<Platform> {
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        return Some(Platform::LinuxAmd64);

        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        return Some(Platform::MacosAmd64);

        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        return Some(Platform::MacosArm64);

        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        return Some(Platform::WindowsAmd64);

        #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
        return Some(Platform::WindowsArm64);

        #[allow(unreachable_code)]
        None
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id())
    }
}

/// Calculate total size for a set of platforms
pub fn total_size(platforms: &[Platform]) -> u64 {
    platforms.iter().map(|p| p.size_bytes()).sum()
}

/// Format total size for display
pub fn format_size(bytes: u64) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{} KB", bytes / 1024)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_id_roundtrip() {
        for platform in ALL_PLATFORMS {
            let id = platform.id();
            let parsed = Platform::from_id(id);
            assert_eq!(parsed, Some(*platform));
        }
    }

    #[test]
    fn test_current_platform_is_known() {
        // This test will pass on supported platforms
        let current = Platform::current();
        if let Some(p) = current {
            assert!(ALL_PLATFORMS.contains(&p));
        }
    }

    #[test]
    fn test_binary_names() {
        assert_eq!(Platform::LinuxAmd64.binary_name(), "rnr-linux-amd64");
        assert_eq!(
            Platform::WindowsAmd64.binary_name(),
            "rnr-windows-amd64.exe"
        );
    }
}
