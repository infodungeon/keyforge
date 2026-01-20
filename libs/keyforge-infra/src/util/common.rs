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

use crate::error::InfraResult;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Calculates the SHA-256 hash of a file on disk.
///
/// # Errors
///
/// Returns `InfraError` if the file cannot be read.
pub fn calculate_file_hash<P: AsRef<Path>>(path: P) -> InfraResult<String> {
    let mut file = File::open(path).map_err(InfraError::Io)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 4096];

    loop {
        let n = file.read(&mut buffer).map_err(InfraError::Io)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    Ok(hex::encode(hasher.finalize()))
}

/// Helper for testing: calculates hash of a string.
#[must_use]
pub fn calculate_file_hash_str(s: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    hex::encode(hasher.finalize())
}

use keyforge_protocol::{BiometricSample, UserStatsStore};

/// Generates a serialized cost matrix based on the user's historical typing statistics.
///
/// **Note**: This is currently a placeholder that returns an empty cost matrix.
/// TODO: Implement statistical analysis of `UserStatsStore` to generate specific K/V pairs.
pub fn generate_cost_profile(_store: &UserStatsStore) -> String {
    tracing::warn!("generate_cost_profile is a STUB - returning empty matrix");
    // Return empty CostModel JSON
    r#"{"meta":{"version":2.0,"description":"Stub","unit":"pts"},"models":{},"dynamic_rules":{"sequence_modifiers":{},"penalties":{},"constraints":{}}}"#.to_string()
}

/// A streaming builder for generating cost profiles from biometric samples.
/// This allows processing large datasets without loading everything into memory.
#[derive(Default, Debug)]
pub struct StreamingProfileBuilder {
    sample_count: usize,
}

impl StreamingProfileBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_sample(&mut self, _sample: &BiometricSample) {
        // TODO: Aggregate stats (mean/variance per bigram)
        self.sample_count += 1;
    }

    pub fn generate(&self) -> String {
        tracing::warn!(
            "StreamingProfileBuilder is a STUB - returning empty matrix (processed {} samples)",
            self.sample_count
        );
        r#"{"meta":{"version":2.0,"description":"Stub","unit":"pts"},"models":{},"dynamic_rules":{"sequence_modifiers":{},"penalties":{},"constraints":{}}}"#.to_string()
    }
}

use crate::error::InfraError;
use keyforge_model::constants::MAX_INPUT_FILE_SIZE;
use keyforge_model::keycodes::{KeycodeDefinition, KeycodeRegistry};

/// Loads a keycode registry from a JSON file.
///
/// # Errors
///
/// Returns `InfraError` if the file cannot be read or parsed.
pub fn load_keycode_registry(path: &Path) -> InfraResult<KeycodeRegistry> {
    let content = crate::fs::io::read_to_string_limited(path, MAX_INPUT_FILE_SIZE)?;

    let defs: Vec<KeycodeDefinition> = serde_json::from_str(&content).map_err(InfraError::Serde)?;
    Ok(KeycodeRegistry::new(defs))
}

use keyforge_model::config::CorpusSource;

/// Generates a deterministic fingerprint for a set of corpora sources.
#[must_use]
pub fn calculate_fingerprint(sources: &[CorpusSource]) -> String {
    // 1. Sort by ID for canonicalization
    let mut sorted = sources.to_vec();
    sorted.sort_by(|a, b| a.id.cmp(&b.id));

    // 2. Hash the sorted list
    let mut hasher = Sha256::new();
    if let Ok(bytes) = serde_json::to_vec(&sorted) {
        hasher.update(bytes);
    } else {
        // Fallback: use raw ID list
        for s in &sorted {
            hasher.update(s.id.as_bytes());
        }
    }
    hex::encode(hasher.finalize())
}

/// Aggressively sanitizes filenames to prevent traversal or shell issues.
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_calculate_file_hash() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("test.txt");
        fs::write(&path, "hello").unwrap();
        
        let hash = calculate_file_hash(&path).unwrap();
        assert!(!hash.is_empty());
        assert!(calculate_file_hash("nonexistent").is_err());
    }

    #[test]
    fn test_stubs() {
        let store = UserStatsStore::default();
        assert!(generate_cost_profile(&store).contains("Stub"));
        
        let mut builder = StreamingProfileBuilder::new();
        builder.add_sample(&BiometricSample { bigram: "th".into(), ms: 10.0, timestamp: 0 });
        assert!(builder.generate().contains("Stub"));
    }

    #[test]
    fn test_load_keycode_registry() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("keycodes.json");
        fs::write(&path, r#"[{"code": 97, "id": "KC_A", "label": "a", "aliases": []}]"#).unwrap();
        
        let reg = load_keycode_registry(&path).unwrap();
        assert_eq!(reg.definitions.len(), 1);
        assert!(load_keycode_registry(&temp.path().join("missing")).is_err());

        // Invalid JSON
        fs::write(&path, "invalid").unwrap();
        assert!(load_keycode_registry(&path).is_err());
    }

    #[test]
    fn test_calculate_fingerprint() {
        let s1 = vec![CorpusSource { id: "a".into(), weight: 1.0, hash: None }];
        let s2 = vec![CorpusSource { id: "a".into(), weight: 1.0, hash: None }];
        assert_eq!(calculate_fingerprint(&s1), calculate_fingerprint(&s2));
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("valid.txt"), "direct.txt".replace("direct", "valid"));
        // Wait, sanitize_filename("valid.txt") -> "valid.txt"
        assert_eq!(sanitize_filename("valid.txt"), "valid.txt");
        assert_eq!(sanitize_filename("invalid/path"), "invalid_path");
    }

    #[test]
    fn test_normalize_path() {
        assert_eq!(normalize_path("a/b/c"), Some("a/b/c".into()));
        assert_eq!(normalize_path("a/../b"), Some("b".into()));
        assert_eq!(normalize_path("../outside"), None);
        assert_eq!(normalize_path("/absolute"), None);
        assert_eq!(normalize_path(""), None);
    }
}
