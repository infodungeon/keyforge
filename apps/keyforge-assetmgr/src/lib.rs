// apps/keyforge-assetmgr/src/lib.rs

pub mod ops;

use std::path::Path;

/// Checks if a file is hidden (starts with a dot).
#[must_use]
pub fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|s| s.to_str())
        .is_some_and(|s| s.starts_with('.'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_hidden() {
        assert!(is_hidden(Path::new(".hidden")));
        assert!(!is_hidden(Path::new("visible.json")));
    }
}
