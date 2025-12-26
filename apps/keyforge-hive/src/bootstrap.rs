use serde::Deserialize;
use std::path::{Path, PathBuf};

/// Minimal bootstrap config that tells Hive where its canonical `data/` root lives.
///
/// This file must live outside of `data/system/config` to avoid a chicken-and-egg
/// dependency. Default location: `/etc/keyforge/hive_bootstrap.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct HiveBootstrapConfig {
    /// Path to the KeyForge data root directory.
    ///
    /// Expected layout:
    /// - `${data_root}/system/...` (read-only system assets)
    /// - `${data_root}/user/...`   (server-side writable workspace, if applicable)
    pub data_root: PathBuf,
}

impl HiveBootstrapConfig {
    pub const DEFAULT_PATH: &'static str = "/etc/keyforge/hive.toml";

    pub fn load(path: &Path) -> Result<Self, String> {
        let raw = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read bootstrap config {:?}: {}", path, e))?;
        toml::from_str(&raw)
            .map_err(|e| format!("Failed to parse bootstrap config {:?}: {}", path, e))
    }
}
