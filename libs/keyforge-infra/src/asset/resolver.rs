use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct PathResolver {
    pub root: PathBuf,
}

impl PathResolver {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn resolve_system_path(&self, category: &str, stem: &str) -> Option<PathBuf> {
        let sub = match category {
            "keyboards" => "keyboards/models",
            "weights" => "weights",
            "config" | "keycodes" => "config",
            "keymap_extras" => "keymap_extras",
            _ => category,
        };

        let p = self.root.join("system").join(sub).join(format!("{}.mpk.zst", stem));
        if p.exists() { return Some(p); }

        let p_direct = self.root.join("system").join(category).join(format!("{}.mpk.zst", stem));
        if p_direct.exists() { return Some(p_direct); }

        None
    }

    pub fn resolve_user_path(&self, category: &str, stem: &str) -> Option<PathBuf> {
        let sub = match category {
            "keycodes" => "config",
            _ => category,
        };
        let p = self.root.join("user").join(sub).join(format!("{}.json", stem));
        p.exists().then_some(p)
    }

    pub fn resolve_direct_path(&self, name: &str) -> Option<PathBuf> {
        let p = PathBuf::from(name);
        if p.is_absolute() {
            return self.safe_join(name).ok().filter(|p| p.exists());
        }
        if name.starts_with("./") || name.starts_with("../") {
            return self.safe_join(name).ok().filter(|p| p.exists());
        }
        None
    }

    pub fn safe_join(&self, user_path: &str) -> Result<PathBuf, String> {
        let base = std::fs::canonicalize(&self.root)
            .map_err(|e| format!("Failed to canonicalize root: {}", e))?;
        
        let full = if Path::new(user_path).is_absolute() {
            PathBuf::from(user_path)
        } else {
            self.root.join(user_path)
        };

        let canonical = match std::fs::canonicalize(&full) {
            Ok(p) => p,
            Err(_) => {
                if full.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
                    return Err("Path traversal detected (manual check)".into());
                }
                return Ok(full);
            }
        };

        if canonical.starts_with(&base) {
            Ok(canonical)
        } else {
            Err("Path traversal detected (prefix check)".into())
        }
    }
}
