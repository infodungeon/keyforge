use crate::error::BoundaryError;
use std::path::{Component, Path, PathBuf};

/// A validated, normalized path that is guaranteed to be safe for operations.
///
/// Invariants:
/// 1. Path is not absolute.
/// 2. Path contains no parent directory traversal (`..`) that escapes the root.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SafePath(PathBuf);

impl SafePath {
    /// Attempts to create a `SafePath` from a string slice.
    ///
    /// The path is normalized using stack-based normalization.
    /// It must not be absolute and must not contain `..` components that would escape the root.
    ///
    /// # Errors
    /// Returns `BoundaryError` if the path is absolute or attempts parent directory traversal.
    pub fn try_from_str(s: &str) -> Result<Self, BoundaryError> {
        let path = Path::new(s);

        // 1. Reject absolute paths
        if path.is_absolute() {
            return Err(BoundaryError::Invariant(format!("Path '{s}' is absolute")));
        }

        let mut stack = Vec::new();

        for component in path.components() {
            match component {
                Component::Prefix(_) | Component::RootDir => {
                    // Should be caught by is_absolute, but strictly valid relative paths shouldn't have these.
                    return Err(BoundaryError::Invariant(format!(
                        "Path '{s}' contains invalid root/prefix components"
                    )));
                }
                Component::CurDir => {
                    // Ignore '.'
                }
                Component::ParentDir => {
                    if stack.pop().is_none() {
                        return Err(BoundaryError::Invariant(format!(
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

        if safe.as_os_str().is_empty() {
            safe.push(".");
        }

        Ok(SafePath(safe))
    }

    /// Creates a `SafePath` from a `std::path::Path`, validating its safety.
    ///
    /// # Errors
    /// Returns `BoundaryError` if the path is absolute or attempts parent directory traversal.
    pub fn try_from_path(p: &std::path::Path) -> Result<Self, BoundaryError> {
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
    #[must_use]
    pub fn from_trusted_root_path(path: PathBuf) -> Self {
        SafePath(path)
    }

    /// Joins a relative path component to this `SafePath`, returning a new validated `SafePath`.
    ///
    /// # Errors
    /// Returns `BoundaryError` if the resulting path is invalid or attempts to escape the root.
    pub fn join(&self, rel: &str) -> Result<Self, BoundaryError> {
        let mut combined = self.0.clone();
        combined.push(rel);

        let path = combined.as_path();
        let mut stack = Vec::new();

        for component in path.components() {
            match component {
                Component::Prefix(_) | Component::RootDir => {
                    if !self.0.is_absolute() {
                        return Err(BoundaryError::Invariant(format!(
                            "Path '{}' contains invalid root/prefix components",
                            path.display()
                        )));
                    }
                }
                Component::CurDir => {}
                Component::ParentDir => {
                    if stack.pop().is_none() {
                        return Err(BoundaryError::Invariant(format!(
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
    #[must_use]
    pub fn from_trusted_root(root: &Path, rel: &SafePath) -> Self {
        SafePath(root.join(rel.as_path()))
    }

    /// Joins a safe relative path to this `SafePath`, assuming this is a trusted root.
    #[must_use]
    pub fn join_trusted(&self, rel: &SafePath) -> Self {
        Self::from_trusted_root(self.as_path(), rel)
    }
}

impl AsRef<Path> for SafePath {
    fn as_ref(&self) -> &Path {
        self.0.as_path()
    }
}

impl std::str::FromStr for SafePath {
    type Err = BoundaryError;

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
}
