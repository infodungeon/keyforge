// libs/keyforge-infra/src/asset/resolver.rs

use crate::error::{InfraError, InfraResult};
use std::path::{Path, PathBuf};

/// Utility for resolving asset IDs to filesystem paths within a workspace.
///
/// `PathResolver` understands the hierarchical structure of a `KeyForge` workspace,
/// including the separation between system-provided and user-created assets.
#[derive(Clone, Debug)]
pub struct PathResolver {
    /// The root directory of the `KeyForge` workspace.
    pub root: PathBuf,
}

impl PathResolver {
    /// Creates a new `PathResolver` for the specified workspace root.
    #[must_use]
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Safely joins a relative path to the root, preventing path traversal.
    ///
    /// # Errors
    ///
    /// Returns `InfraError::Config` if traversal is detected or the path is absolute.
    pub fn safe_join(&self, path: &str) -> InfraResult<PathBuf> {
        let p = Path::new(path);
        if p.is_absolute() || path.contains("..") {
            return Err(InfraError::Config(format!(
                "Invalid or unsafe path: {path}"
            )));
        }
        Ok(self.root.join(p))
    }

    /// Resolves a system-level asset to its full filesystem path.
    ///
    /// This checks for both compressed (.zst) and uncompressed versions of the asset.
    #[must_use]
    pub fn resolve_system_path(&self, category: &str, stem: &str) -> Option<PathBuf> {
        let base = self.root.join("system").join(category).join(stem);

        // Try direct first
        if base.exists() {
            return Some(base);
        }

        // Try with extensions
        let with_json = base.with_extension("json");
        if with_json.exists() {
            return Some(with_json);
        }

        let with_mpk = base.with_extension("mpk");
        if with_mpk.exists() {
            return Some(with_mpk);
        }

        let with_zst = base.with_extension("mpk.zst");
        if with_zst.exists() {
            return Some(with_zst);
        }

        None
    }

    /// Resolves a user-level asset to its full filesystem path.
    #[must_use]
    pub fn resolve_user_path(&self, category: &str, stem: &str) -> Option<PathBuf> {
        let base = self.root.join("user").join(category).join(stem);
        let with_json = base.with_extension("json");
        if with_json.exists() {
            return Some(with_json);
        }
        if base.exists() {
            return Some(base);
        }
        None
    }

    /// Resolves a path that might be absolute or relative to the workspace root.
    #[must_use]
    pub fn resolve_direct_path(&self, name: &str) -> Option<PathBuf> {
        let p = PathBuf::from(name);
        if p.exists() {
            return Some(p);
        }
        let relative = self.root.join(name);
        if relative.exists() {
            return Some(relative);
        }
        None
    }
}
