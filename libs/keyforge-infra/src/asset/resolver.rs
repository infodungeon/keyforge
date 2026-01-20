use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct PathResolver {
    pub root: PathBuf,
}

impl PathResolver {
    #[must_use]
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    #[must_use]
    pub fn resolve_system_path(&self, category: &str, stem: &str) -> Option<PathBuf> {
        let sub = match category {
            "keyboards" => "keyboards/models",
            "weights" => "weights",
            "config" | "keycodes" => "config",
            "keymap_extras" => "keymap_extras",
            _ => category,
        };

        let p = self.root.join("system").join(sub).join(format!("{stem}.mpk.zst"));
        if p.exists() {
            return Some(p);
        }

        let p_direct = self.root.join("system").join(category).join(format!("{stem}.mpk.zst"));
        if p_direct.exists() {
            return Some(p_direct);
        }

        None
    }

    #[must_use]
    pub fn resolve_user_path(&self, category: &str, stem: &str) -> Option<PathBuf> {
        let sub = match category {
            "keycodes" => "config",
            _ => category,
        };
        let p = self.root.join("user").join(sub).join(format!("{stem}.json"));
        p.exists().then_some(p)
    }

    #[must_use]
    pub fn resolve_direct_path(&self, name: &str) -> Option<PathBuf> {
        let p = PathBuf::from(name);

        // 1. Absolute paths: If it exists, trust the caller provided a direct path
        if p.is_absolute() && p.exists() {
            return Some(p);
        }

        // 2. Explicit relative paths (./ or ../): Resolve relative to process CWD
        if (name.starts_with("./") || name.starts_with("../")) && p.exists() {
            return Some(p);
        }

        // 3. Fallback: Resolve relative to workspace root (sandboxed via safe_join)
        self.safe_join(name).ok().filter(|p| p.exists())
    }

    /// Joins a user-provided path with the root, ensuring it stays within the root directory.
    ///
    /// # Errors
    ///
    /// Returns an error if the root cannot be canonicalized or if path traversal is detected.
    pub fn safe_join(&self, user_path: &str) -> Result<PathBuf, String> {
        let base = std::fs::canonicalize(&self.root)
            .map_err(|e| format!("Failed to canonicalize root: {e}"))?;

        let full = if Path::new(user_path).is_absolute() {
            PathBuf::from(user_path)
        } else {
            self.root.join(user_path)
        };

        let Ok(canonical) = std::fs::canonicalize(&full) else {
            if full
                .components()
                .any(|c| matches!(c, std::path::Component::ParentDir))
            {
                return Err("Path traversal detected (manual check)".into());
            }
            return Ok(full);
        };

        if canonical.starts_with(&base) {
            Ok(canonical)
        } else {
            Err("Path traversal detected (prefix check)".into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_path_resolver_system_user() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        let resolver = PathResolver::new(root.to_path_buf());
        
        let sys_kb = root.join("system/keyboards/models");
        fs::create_dir_all(&sys_kb).unwrap();
        fs::write(sys_kb.join("test.mpk.zst"), "").unwrap();
        
        let user_kb = root.join("user/keyboards");
        fs::create_dir_all(&user_kb).unwrap();
        fs::write(user_kb.join("user_kb.json"), "").unwrap();

        assert!(resolver.resolve_system_path("keyboards", "test").is_some());
        assert!(resolver.resolve_system_path("unknown", "test").is_none());
        assert!(resolver.resolve_user_path("keyboards", "user_kb").is_some());
        assert!(resolver.resolve_user_path("keycodes", "config").is_none());

        // System direct fallback
        let direct_kb = root.join("system/keyboards/direct.mpk.zst");
        fs::create_dir_all(root.join("system/keyboards")).unwrap();
        fs::write(&direct_kb, "").unwrap();
        assert!(resolver.resolve_system_path("keyboards", "direct").is_some());

        // User keycodes path
        let user_cfg = root.join("user/config/mycodes.json");
        fs::create_dir_all(root.join("user/config")).unwrap();
        fs::write(&user_cfg, "").unwrap();
        assert!(resolver.resolve_user_path("keycodes", "mycodes").is_some());
    }

    #[test]
    fn test_path_resolver_direct() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        let resolver = PathResolver::new(root.to_path_buf());
        
        let direct = root.join("direct.json");
        fs::write(&direct, "{}").unwrap();
        
        // Root relative (fallback)
        assert!(resolver.resolve_direct_path("direct.json").is_some());
        
        // Absolute
        assert!(resolver.resolve_direct_path(direct.to_str().unwrap()).is_some());

        // Explicit relative
        fs::write("./test_rel.json", "{}").unwrap();
        assert!(resolver.resolve_direct_path("./test_rel.json").is_some());
        fs::remove_file("./test_rel.json").unwrap();
    }

    #[test]
    fn test_path_resolver_safe_join() {
        let temp = tempfile::tempdir().unwrap();
        let root = fs::canonicalize(temp.path()).unwrap();
        let resolver = PathResolver::new(root.clone());
        
        // Valid join
        assert!(resolver.safe_join("data.json").is_ok());
        
        // Path traversal
        assert!(resolver.safe_join("../outside.json").is_err());
        assert!(resolver.safe_join("/etc/passwd").is_err());
    }
}
