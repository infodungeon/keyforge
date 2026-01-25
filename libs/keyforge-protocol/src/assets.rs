// libs/keyforge-protocol/src/assets.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You    may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::constants;
use keyforge_model::Validator;
use serde::{Deserialize, Serialize};
#[cfg(feature = "ts_bindings")]
use ts_rs::TS;
use utoipa::ToSchema;

/// A manifest entry for the Global Asset Cache.
/// Stored in Valkey to ensure all nodes agree on asset versions.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct AssetManifestEntry {
    /// Unique identifier for the asset (e.g., "keyboard:corne" or "corpus:english").
    pub id: String,
    /// BLAKE3 or SHA256 hash of the asset content for integrity verification.
    pub hash: String,
    /// Total size of the asset in bytes.
    pub size_bytes: u64,
    /// UNIX timestamp of when the asset was last modified or synchronized.
    pub last_updated: u64,
}

impl Validator for AssetManifestEntry {
    fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("Asset ID cannot be empty".into());
        }
        if self.hash.is_empty() {
            return Err("Asset hash cannot be empty".into());
        }
        Ok(())
    }
}

/// Represents a single timing sample for a bigram.
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct BiometricSample {
    /// The bigram string (e.g., "th").
    pub bigram: String,
    /// The time in milliseconds.
    pub ms: f64,
    /// Timestamp of the sample.
    pub timestamp: u64,
}

impl Validator for BiometricSample {
    fn validate(&self) -> Result<(), String> {
        if self.bigram.len() != 2 {
            return Err(format!("Invalid bigram length: '{}'", self.bigram));
        }
        if !self.bigram.is_ascii() {
            return Err(format!(
                "Bigram contains non-ASCII characters: '{}'",
                self.bigram
            ));
        }
        if self.ms <= 0.0 || self.ms > constants::MAX_BIOMETRIC_MS {
            return Err(format!(
                "Biometric sample out of realistic range: {}ms",
                self.ms
            ));
        }
        Ok(())
    }
}

/// Aggregated statistics for a user.
#[derive(Serialize, Deserialize, Clone, Default, Debug, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct UserStatsStore {
    /// Total number of sessions.
    pub sessions: u64,
    /// Total keystrokes typed.
    pub total_keystrokes: u64,
    /// Collection of biometric samples.
    #[serde(deserialize_with = "crate::serde_utils::deserialize_limited_vec")]
    #[cfg_attr(feature = "ts_bindings", ts(type = "Array<BiometricSample>"))]
    pub biometrics: Vec<BiometricSample>,
}

impl Validator for UserStatsStore {
    fn validate(&self) -> Result<(), String> {
        for s in &self.biometrics {
            s.validate()?;
        }
        Ok(())
    }
}

/// Response containing available layouts.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct PopulationResponse {
    /// List of layout strings.
    #[serde(deserialize_with = "crate::serde_utils::deserialize_limited_vec")]
    #[cfg_attr(feature = "ts_bindings", ts(type = "Array<string>"))]
    pub layouts: Vec<String>,
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;

    #[test]
    fn test_biometric_sample_validation() {
        let valid = BiometricSample {
            bigram: "th".into(),
            ms: 100.0,
            timestamp: 0,
        };
        assert!(valid.validate().is_ok());

        let invalid_len = BiometricSample {
            bigram: "abc".into(),
            ms: 100.0,
            timestamp: 0,
        };
        assert!(invalid_len.validate().is_err());

        let invalid_ms = BiometricSample {
            bigram: "th".into(),
            ms: -1.0,
            timestamp: 0,
        };
        assert!(invalid_ms.validate().is_err());

        let extreme_ms = BiometricSample {
            bigram: "th".into(),
            ms: constants::MAX_BIOMETRIC_MS + 1.0,
            timestamp: 0,
        };
        assert!(extreme_ms.validate().is_err());

        let invalid_ascii = BiometricSample {
            bigram: "t\u{00E9}".into(), // t + é
            ms: 100.0,
            timestamp: 0,
        };
        assert!(invalid_ascii.validate().is_err());
    }
}
