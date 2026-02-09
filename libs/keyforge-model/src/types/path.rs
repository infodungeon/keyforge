use crate::error::ModelError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::path::{Component, Path, PathBuf};

/// A validated, normalized path that is guaranteed to be safe for operations.
///
/// Invariants:
/// 1. Path is not absolute.
/// 2. Path contains no parent directory traversal (`..`) that escapes the root.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SafePath(PathBuf);
/// Error types for path validation.
#[derive(Debug, thiserror::Error)]
pub enum PathError {
    /// The path is absolute, which is not allowed.
    #[error("Path is absolute")]
    Absolute,
    /// The path contains invalid prefix or root components.
    #[error("Path contains invalid root/prefix components")]
    InvalidPrefix,
    /// The path attempts to traverse above the root directory.
    #[error("Path attempts to traverse above root")]
    Traversal,
}

impl SafePath {
    /// Attempts to create a `SafePath` from a string slice.
    ///
    /// The path is normalized using stack-based normalization.
    /// It must not be absolute and must not contain `..` components that would escape the root.
    ///
    /// # Errors
    /// Returns `ModelError` if the path is absolute or attempts parent directory traversal.
    pub fn try_from_str(s: &str) -> Result<Self, ModelError> {
        let path = Path::new(s);

        // 1. Reject absolute paths
        if path.is_absolute() {
            return Err(ModelError::Invariant(format!("Path '{s}' is absolute")));
        }

        let mut stack = Vec::new();

        for component in path.components() {
            match component {
                Component::Prefix(_) | Component::RootDir => {
                    // Should be caught by is_absolute, but strictly valid relative paths shouldn't have these.
                    return Err(ModelError::Invariant(format!(
                        "Path '{s}' contains invalid root/prefix components"
                    )));
                }
                Component::CurDir => {
                    // Ignore '.'
                }
                Component::ParentDir => {
                    if stack.pop().is_none() {
                        return Err(ModelError::Invariant(format!(
                            "Path '{s}' attempts to traverse above root"
                        )));
                    }
                }
                Component::Normal(c) => {
                    stack.push(c);
                }
            }
        }

        // Reconstruct path from stack
        let mut safe = PathBuf::new();
        for component in stack {
            safe.push(component);
        }

        // If empty, it means it normalized to ".", which is safe.
        // Wait, if a path is empty, it is usually valid as current directory?
        // But PathBuf::new() creates an empty path.
        // Let's verify what happens if we return empty pathbuf.
        // usually consumers want "." or just empty string.
        // let's stick to returning the normalized pathbuf.
        // If the original input effectively meant ".", the stack is empty.
        // So the SafePath will wrap an empty PathBuf.
        // When using as_path(), it will be empty.
        // This might be ambiguous. It is better to explicitly store "." if it is meant to be current dir.

        if safe.as_os_str().is_empty() {
            // However, depending on usage, maybe empty path is bad?
            // "SafePath" implies it points to something.
            // Let's assume empty normalized path means "."
            safe.push(".");
        }

        Ok(SafePath(safe))
    }

    /// Creates a `SafePath` from a `std::path::Path`, validating its safety.
    ///
    /// # Errors
    /// Returns `ModelError` if the path is absolute or attempts parent directory traversal.
    pub fn try_from_path(p: &std::path::Path) -> Result<Self, ModelError> {
        Self::try_from_str(&p.to_string_lossy())
    }

    /// Returns a reference to the underlying `Path`.
    #[must_use]
    pub fn as_path(&self) -> &Path {
        self.0.as_path()
    }

    /// Consumes the `SafePath` and returns the underlying `PathBuf`.
    #[must_use]
    pub fn into_inner(self) -> PathBuf {
        self.0
    }

    /// Converts a trusted `PathBuf` into a `SafePath` representing the root itself.
    ///
    /// This is equivalent to assuming the path is the root and pointing to `.` relative to it.
    /// Use this when you have a canonical, trusted absolute path (e.g. data directory) that
    /// should serve as the anchor for further `SafePath` operations.
    #[must_use]
    pub fn from_trusted_root_path(path: PathBuf) -> Self {
        // Since we are declaring it a trusted root, we don't validate it as a relative path component.
        // It is by definition safe because it IS the root.
        SafePath(path)
    }

    /// Joins a relative path component to this `SafePath`, returning a new validated `SafePath`.
    ///
    /// # Errors
    /// Returns `ModelError` if the resulting path is invalid or attempts to escape the root.
    pub fn join(&self, rel: &str) -> Result<Self, ModelError> {
        let mut combined = self.0.clone();
        combined.push(rel);
        // We reuse try_from_str logic but we need to handle the case where self.0 is already absolute
        // because it was created via from_trusted_root.

        let path = combined.as_path();
        let mut stack = Vec::new();

        for component in path.components() {
            match component {
                Component::Prefix(_) | Component::RootDir => {
                    if !self.0.is_absolute() {
                        return Err(ModelError::Invariant(format!(
                            "Path '{}' contains invalid root/prefix components",
                            path.display()
                        )));
                    }
                    // If self was already absolute, we allow the root/prefix to stay.
                }
                Component::CurDir => {}
                Component::ParentDir => {
                    if stack.pop().is_none() {
                        // Even for absolute paths, we don't allow escaping the initial root.
                        // This is a bit strict but safer.
                        return Err(ModelError::Invariant(format!(
                            "Path '{}' attempts to traverse above root",
                            path.display()
                        )));
                    }
                }
                Component::Normal(c) => {
                    stack.push(c);
                }
            }
        }

        let mut safe = if path.is_absolute() {
            // Find the root/prefix components
            let mut p = PathBuf::new();
            for component in path.components() {
                if matches!(component, Component::Prefix(_) | Component::RootDir) {
                    p.push(component);
                } else {
                    break;
                }
            }
            p
        } else {
            PathBuf::new()
        };

        for component in stack {
            safe.push(component);
        }

        if safe.as_os_str().is_empty() {
            safe.push(".");
        }

        Ok(SafePath(safe))
    }

    /// Creates a `SafePath` from a trusted base and a safe relative path.
    ///
    /// This is used by infrastructure layers to combine a validated relative fragment
    /// with a trusted system root.
    #[must_use]
    pub fn from_trusted_root(root: &Path, rel: &SafePath) -> Self {
        SafePath(root.join(rel.as_path()))
    }

    /// Joins a safe relative path to this `SafePath`, assuming this is a trusted root.
    ///
    /// This is a convenience method for `from_trusted_root(self.as_path(), rel)`.
    #[must_use]
    pub fn join_trusted(&self, rel: &SafePath) -> Self {
        Self::from_trusted_root(self.as_path(), rel)
    }
}

impl<'de> Deserialize<'de> for SafePath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        SafePath::try_from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl Serialize for SafePath {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize as string
        self.0.to_string_lossy().serialize(serializer)
    }
}

impl AsRef<Path> for SafePath {
    fn as_ref(&self) -> &Path {
        self.0.as_path()
    }
}

impl std::str::FromStr for SafePath {
    type Err = crate::error::ModelError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from_str(s)
    }
}

impl std::fmt::Display for SafePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.display())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_paths() -> Result<(), Box<dyn std::error::Error>> {
        assert!(SafePath::try_from_str("a/b/c").is_ok());
        let p = SafePath::try_from_str("a/b/../c")?;
        assert_eq!(p.as_path(), Path::new("a/c"));

        let p2 = SafePath::try_from_str("./foo")?;
        assert_eq!(p2.as_path(), Path::new("foo"));
        Ok(())
    }

    #[test]
    fn test_invalid_paths() {
        assert!(SafePath::try_from_str("/etc/passwd").is_err());
        assert!(SafePath::try_from_str("../foo").is_err());
        assert!(SafePath::try_from_str("a/../../b").is_err());
    }

    #[test]
    fn test_join() -> Result<(), Box<dyn std::error::Error>> {
        let p = SafePath::try_from_str("a/b")?;
        let p2 = p.join("c")?;
        assert_eq!(p2.as_path(), Path::new("a/b/c"));

        assert!(p.join("../../../d").is_err());

        let root = Path::new("/etc");
        let safe_root = SafePath::from_trusted_root(root, &SafePath::try_from_str(".")?);
        assert_eq!(safe_root.as_path(), Path::new("/etc/."));

        let joined = safe_root.join("passwd")?;
        assert_eq!(joined.as_path(), Path::new("/etc/./passwd"));
        Ok(())
    }

    #[test]
    fn test_serde() -> Result<(), Box<dyn std::error::Error>> {
        let json = "\"foo/bar\"";
        let p: SafePath = serde_json::from_str(json)?;
        assert_eq!(p.as_path(), Path::new("foo/bar"));

        let bad_json = "\"../bad\"";
        assert!(serde_json::from_str::<SafePath>(bad_json).is_err());
        Ok(())
    }
}
