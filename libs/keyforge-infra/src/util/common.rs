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

use std::collections::HashMap;
use keyforge_protocol::{BiometricSample, UserStatsStore};

/// Statistics for a specific bigram.
#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BigramStats {
    /// Number of samples collected.
    pub count: usize,
    /// Arithmetic mean of latency (ms).
    pub mean: f64,
    /// Sum of squared differences from the mean (for variance calculation).
    pub m2: f64,
}

impl BigramStats {
    /// Updates the stats with a new sample using Welford's online algorithm.
    pub fn add_sample(&mut self, ms: f64) {
        self.count += 1;
        let delta = ms - self.mean;
        #[allow(clippy::cast_precision_loss)]
        {
            self.mean += delta / self.count as f64;
        }
        let delta2 = ms - self.mean;
        self.m2 += delta * delta2;
    }

    /// Returns the variance of the samples.
    #[must_use]
    pub fn variance(&self) -> f64 {
        if self.count < 2 {
            0.0
        } else {
            #[allow(clippy::cast_precision_loss)]
            {
                self.m2 / self.count as f64
            }
        }
    }

    /// Returns the standard deviation of the samples.
    #[must_use]
    pub fn std_dev(&self) -> f64 {
        self.variance().sqrt()
    }
}

/// Generates a serialized cost matrix based on the user's historical typing statistics.
#[must_use]
pub fn generate_cost_profile(store: &UserStatsStore) -> String {
    let mut builder = StreamingProfileBuilder::new();
    for sample in &store.biometrics {
        builder.add_sample(sample);
    }
    builder.generate()
}

/// A streaming builder for generating cost profiles from biometric samples.
/// This allows processing large datasets without loading everything into memory.
#[derive(Default, Debug)]
pub struct StreamingProfileBuilder {
    /// Map of bigram strings to their running statistics.
    pub stats: HashMap<String, BigramStats>,
    /// Total number of samples processed.
    pub sample_count: usize,
}

impl StreamingProfileBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a single biometric sample to the running statistics.
    pub fn add_sample(&mut self, sample: &BiometricSample) {
        // Outlier detection: ignore extreme latencies (> 5s)
        if sample.ms > 0.0 && sample.ms < 5000.0 {
            self.stats
                .entry(sample.bigram.clone())
                .or_default()
                .add_sample(sample.ms);
            self.sample_count += 1;
        }
    }

    /// Generates a `CostModel` JSON string from the accumulated statistics.
    #[must_use]
    pub fn generate(&self) -> String {
        use serde_json::json;

        // Task-infra-031: Convert stats to sequence modifiers
        let mut modifiers = HashMap::new();
        
        // Calculate global baseline (mean of all bigrams)
        let total_mean: f64 = self.stats.values().map(|s| s.mean).sum();
        let global_avg = if self.stats.is_empty() { 
            1.0 
        } else { 
            #[allow(clippy::cast_precision_loss)]
            {
                total_mean / self.stats.len() as f64 
            }
        };

        for (bigram, s) in &self.stats {
            // Only use bigrams with sufficient sample size
            if s.count >= 5 {
                // Ratio relative to average: > 1.0 means slower than average
                let ratio = s.mean / global_avg;
                #[allow(clippy::cast_possible_truncation)]
                {
                    modifiers.insert(bigram.clone(), ratio as f32);
                }
            }
        }

        let model = json!({
            "meta": {
                "version": 2.0,
                "description": format!("Generated from {} biometric samples", self.sample_count),
                "unit": "pts"
            },
            "models": {},
            "dynamic_rules": {
                "sequence_modifiers": modifiers,
                "penalties": {},
                "constraints": {}
            }
        });

        serde_json::to_string(&model).unwrap_or_else(|_| "{}".to_string())
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
    fn test_biometric_aggregation() {
        let mut builder = StreamingProfileBuilder::new();
        // Add 5 samples for "th" to trigger inclusion in modifiers
        for _ in 0..5 {
            builder.add_sample(&BiometricSample {
                bigram: "th".into(),
                ms: 100.0,
                timestamp: 0,
            });
        }
        // Add 5 samples for "he" with different latency
        for _ in 0..5 {
            builder.add_sample(&BiometricSample {
                bigram: "he".into(),
                ms: 200.0,
                timestamp: 0,
            });
        }

        let json = builder.generate();
        assert!(json.contains("Generated from 10 biometric samples"));
        assert!(json.contains("\"th\":0.666")); // 100 / 150
        assert!(json.contains("\"he\":1.333")); // 200 / 150
    }

    #[test]
    fn test_load_keycode_registry() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("keycodes.json");
        fs::write(
            &path,
            r#"[{"code": 97, "id": "KC_A", "label": "a", "aliases": []}]"#,
        )
        .unwrap();

        let reg = load_keycode_registry(&path).unwrap();
        assert_eq!(reg.definitions.len(), 1);
        assert!(load_keycode_registry(&temp.path().join("missing")).is_err());

        // Invalid JSON
        fs::write(&path, "invalid").unwrap();
        assert!(load_keycode_registry(&path).is_err());
    }

    #[test]
    fn test_calculate_fingerprint() {
        let s1 = vec![CorpusSource {
            id: "a".into(),
            weight: 1.0,
            hash: None,
        }];
        let s2 = vec![CorpusSource {
            id: "a".into(),
            weight: 1.0,
            hash: None,
        }];
        assert_eq!(calculate_fingerprint(&s1), calculate_fingerprint(&s2));
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(
            sanitize_filename("valid.txt"),
            "direct.txt".replace("direct", "valid")
        );
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
