// libs/keyforge-infra/src/hardware/mod.rs

use std::path::PathBuf;

/// Abstraction for hardware-specific information providers.
pub trait HardwareProvider: Send + Sync + std::fmt::Debug {
    /// Reads content from a hardware-specific system file (e.g., /proc/cpuinfo).
    ///
    /// # Errors
    /// Returns `std::io::Result` if the file cannot be read.
    fn read_system_file(&self, path: &str) -> std::io::Result<String>;

    /// Reads cache-specific information from the system.
    ///
    /// # Errors
    /// Returns `std::io::Result` if the information cannot be retrieved.
    fn read_cache_info(&self, index: u8, field: &str) -> std::io::Result<String>;
}

/// Implementation of `HardwareProvider` that uses the real filesystem.
#[derive(Debug, Clone, Default)]
pub struct FsHardwareProvider {
    root: PathBuf,
}

impl FsHardwareProvider {
    /// Creates a new `FsHardwareProvider` using the specified root directory for storage.
    #[must_use]
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }
}

impl HardwareProvider for FsHardwareProvider {
    fn read_system_file(&self, path: &str) -> std::io::Result<String> {
        let p = path.strip_prefix('/').unwrap_or(path);
        std::fs::read_to_string(self.root.join(p))
    }

    fn read_cache_info(&self, index: u8, field: &str) -> std::io::Result<String> {
        let path = format!("sys/devices/system/cpu/cpu0/cache/index{index}/{field}");
        self.read_system_file(&path)
    }
}
