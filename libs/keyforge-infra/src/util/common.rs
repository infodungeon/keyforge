// libs/keyforge-infra/src/util/common.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::error::{InfraError, InfraResult};
use keyforge_model::constants::MAX_INPUT_FILE_SIZE;
use keyforge_model::keycodes::KeycodeRegistry;
use keyforge_model::types::path::SafePath;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

/// Computes the SHA-256 hash of a file's contents.
///
/// # Errors
/// Returns `InfraError::Io` if the file cannot be read.
pub fn calculate_file_hash(path: &SafePath) -> InfraResult<String> {
    let file = File::open(path.as_path()).map_err(InfraError::Io)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];

    loop {
        let count = reader.read(&mut buffer).map_err(InfraError::Io)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// Helper for testing: calculates hash of a string.
#[must_use]
pub fn calculate_file_hash_str(s: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    hex::encode(hasher.finalize())
}

/// Loads a keycode registry from a JSON file.
///
/// # Errors
/// Returns `InfraError` if the file cannot be read or parsed.
pub fn load_keycode_registry(path: &SafePath) -> InfraResult<KeycodeRegistry> {
    let content = crate::fs::io::read_to_string_limited(path, MAX_INPUT_FILE_SIZE)?;
    let dto: keyforge_protocol::KeycodeRegistryDto =
        serde_json::from_str(&content).map_err(InfraError::Serde)?;
    Ok(dto.into())
}

/// Aggregately sanitizes filenames to prevent traversal or shell issues.
/// Allowlist: Alphanumeric, dot, underscore, hyphen.
/// Replaces everything else with underscore.
#[must_use]
pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Normalizes a path to prevent traversal and ensure consistent format (forward slashes).
/// Returns None if the path attempts to step above its root.
#[must_use]
#[deprecated(
    since = "0.9.0",
    note = "Use keyforge_model::types::path::SafePath instead"
)]
pub fn normalize_path(raw: &str) -> Option<String> {
    let p = Path::new(raw);
    let mut stack = Vec::new();

    for comp in p.components() {
        match comp {
            std::path::Component::Normal(s) => stack.push(s.to_string_lossy()),
            std::path::Component::ParentDir => {
                if stack.is_empty() {
                    return None;
                }
                stack.pop();
            }
            std::path::Component::RootDir | std::path::Component::Prefix(_) => return None,
            std::path::Component::CurDir => {}
        }
    }

    if stack.is_empty() {
        None
    } else {
        Some(stack.join("/"))
    }
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_calculate_file_hash() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let path = temp.path().join("test.txt");
        fs::write(&path, "hello")?;

        let safe_path = SafePath::from_trusted_root_path(path);
        let hash = calculate_file_hash(&safe_path)?;
        assert!(!hash.is_empty());

        let nonexistent = SafePath::from_trusted_root_path(std::path::PathBuf::from("nonexistent"));
        assert!(calculate_file_hash(&nonexistent).is_err());
        Ok(())
    }

    #[test]
    fn test_load_keycode_registry() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let path = temp.path().join("keycodes.json");
        fs::write(
            &path,
            r#"{"definitions": [{"code": 97, "id": "KC_A", "label": "a", "aliases": []}]}"#,
        )?;

        let rel = SafePath::try_from_str("keycodes.json")?;
        let safe_path = SafePath::from_trusted_root(temp.path(), &rel);
        let reg = load_keycode_registry(&safe_path)?;
        assert_eq!(reg.definitions.len(), 1);

        let missing_rel = SafePath::try_from_str("missing")?;
        let missing = SafePath::from_trusted_root(temp.path(), &missing_rel);
        assert!(load_keycode_registry(&missing).is_err());

        // Invalid JSON
        fs::write(&path, "invalid")?;
        assert!(load_keycode_registry(&safe_path).is_err());
        Ok(())
    }

    #[test]
    fn test_sanitize_filename() -> anyhow::Result<()> {
        assert_eq!(sanitize_filename("valid.txt"), "valid.txt");
        assert_eq!(sanitize_filename("invalid/path"), "invalid_path");
        Ok(())
    }

    #[test]
    #[allow(deprecated)]
    fn test_normalize_path() -> anyhow::Result<()> {
        assert_eq!(normalize_path("a/b/c"), Some("a/b/c".into()));
        assert_eq!(normalize_path("a/../b"), Some("b".into()));
        assert_eq!(normalize_path("../outside"), None);
        assert_eq!(normalize_path("/absolute"), None);
        assert_eq!(normalize_path(""), None);
        Ok(())
    }
}
