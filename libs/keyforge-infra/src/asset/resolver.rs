// libs/keyforge-infra/src/asset/resolver.rs

use crate::error::{InfraError, InfraResult};
use keyforge_boundary::SafePath;
use std::path::Path;

/// Utility for resolving asset IDs to filesystem paths within a workspace.
///
/// `PathResolver` understands the hierarchical structure of a `KeyForge` workspace,
/// including the separation between system-provided and user-created assets.
#[derive(Clone, Debug)]
pub struct PathResolver {
    /// The root directory of the `KeyForge` workspace.
    pub root: SafePath,
}

impl PathResolver {
    /// Creates a new `PathResolver` for the specified workspace root.
    #[must_use]
    #[allow(clippy::panic, clippy::missing_panics_doc)]
    pub fn new(root: &Path) -> Self {
        let Ok(dot) = SafePath::try_from_str(".") else {
            // Panic is acceptable here because "." is a hardcoded constant known to be valid.
            panic!("Critical invariant failed: '.' is not a valid SafePath");
        };
        Self {
            root: SafePath::from_trusted_root(root, &dot),
        }
    }

    /// Safely joins a relative path to the root, preventing path traversal.
    ///
    /// # Errors
    ///
    /// Returns `InfraError` if traversal is detected or the path is absolute.
    pub fn safe_join(&self, path: &str) -> InfraResult<SafePath> {
        self.root.join(path).map_err(InfraError::from)
    }

    /// Resolves a system-level asset to its full filesystem path.
    ///
    /// This checks for both compressed (.zst) and uncompressed versions of the asset.
    #[must_use]
    pub fn resolve_system_path(&self, category: &str, stem: &str) -> Option<SafePath> {
        let Ok(cat_dir) = self.root.join("system").and_then(|p| p.join(category)) else {
            return None;
        };

        // 1. Try direct in category folder
        if let Some(p) = Self::try_extensions(&cat_dir, stem) {
            return Some(p);
        }

        // 2. Try in known subdirectories (e.g., keyboards/models/, keyboards/types/)
        if category == "keyboards" {
            for sub in ["models", "types"] {
                if let Ok(sub_dir) = cat_dir.join(sub) {
                    if let Some(p) = Self::try_extensions(&sub_dir, stem) {
                        return Some(p);
                    }
                }
            }
        }

        // 3. Fallback: Recursive search (expensive, but reliable for deep categories like corpora)
        if cat_dir.as_path().exists() {
            if let Ok(walker) = walkdir::WalkDir::new(cat_dir.as_path())
                .into_iter()
                .collect::<Result<Vec<_>, _>>()
            {
                for entry in walker {
                    if entry.file_type().is_file() {
                        if let Some(s) = entry.path().file_stem().and_then(|s| s.to_str()) {
                            // Match either the full stem or the stem without .mpk
                            if s == stem || s == format!("{stem}.mpk") {
                                // If entry.path() is absolute (which it is from walkdir),
                                // we need a way to wrap it as SafePath because it's already "safe"
                                // since it was found within our known cat_dir.
                                // However, FsProvider root is usually absolute too.
                                if let Ok(dot) = SafePath::try_from_str(".") {
                                    return Some(SafePath::from_trusted_root(entry.path(), &dot));
                                }
                            }
                        }
                    }
                }
            }
        }

        None
    }

    fn try_extensions(base_dir: &SafePath, stem: &str) -> Option<SafePath> {
        // Try direct first (e.g., "foo" for "foo")
        if let Ok(base) = base_dir.join(stem) {
            if base.as_path().exists() {
                return Some(base);
            }
        }

        // Try with extensions (e.g., "foo.json", "foo.mpk", "foo.mpk.zst")
        for ext in ["json", "mpk", "mpk.zst"] {
            if let Ok(p) = base_dir.join(&format!("{stem}.{ext}")) {
                if p.as_path().exists() {
                    return Some(p);
                }
            }
        }

        None
    }

    /// Resolves a user-level asset to its full filesystem path.
    #[must_use]
    pub fn resolve_user_path(&self, category: &str, stem: &str) -> Option<SafePath> {
        if let Ok(user_cat_dir) = self.root.join("user").and_then(|p| p.join(category)) {
            return Self::try_extensions(&user_cat_dir, stem);
        }
        None
    }

    /// Resolves a path that might be absolute or relative to the workspace root.
    #[must_use]
    pub fn resolve_direct_path(&self, name: &str) -> Option<SafePath> {
        let p = Path::new(name);
        if p.exists() {
            // If absolute and exists, wrap it as a trusted root.
            // This assumes that if the user provides an absolute path that exists,
            // it's implicitly "safe" for direct access.
            if let Ok(dot) = SafePath::try_from_str(".") {
                return Some(SafePath::from_trusted_root(p, &dot));
            }
        }
        if let Ok(relative) = self.root.join(name) {
            if relative.as_path().exists() {
                return Some(relative);
            }
        }
        None
    }
}
