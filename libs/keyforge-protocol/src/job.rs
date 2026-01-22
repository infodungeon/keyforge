// libs/keyforge-protocol/src/job.rs

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

use crate::assets::BiometricSample;
use crate::constants;
use crate::PROTOCOL_VERSION;
use keyforge_model::{
    CorpusSource, CostMatrixSource, KeyConstraint, KeyboardDefinition, LayoutValidator,
    ScoringWeights, SearchParams, Validator,
};
use serde::{Deserialize, Serialize};
#[cfg(feature = "ts_bindings")]
use ts_rs::TS;
use utoipa::ToSchema;

fn default_version() -> u32 {
    PROTOCOL_VERSION
}
fn default_cost_matrix() -> CostMatrixSource {
    CostMatrixSource::default()
}
fn default_corpora() -> Vec<CorpusSource> {
    vec![CorpusSource::default()]
}

/// Full configuration for a running job.
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct JobConfig {
    /// Keyboard geometry definition.
    pub definition: KeyboardDefinition,
    /// Scoring weights.
    pub weights: ScoringWeights,
    /// Search parameters.
    pub params: SearchParams,
    /// Keys pinned to specific positions.
    #[serde(
        default,
        deserialize_with = "crate::serde_utils::deserialize_limited_vec"
    )]
    pub pinned_keys: Vec<KeyConstraint>,
    /// Text corpora to use.
    #[serde(
        default = "default_corpora",
        deserialize_with = "crate::serde_utils::deserialize_limited_vec"
    )]
    pub corpora: Vec<CorpusSource>,
    /// Cost matrix source.
    #[serde(default = "default_cost_matrix")]
    pub cost_matrix: CostMatrixSource,
    /// User biometric data.
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        deserialize_with = "crate::serde_utils::deserialize_limited_vec"
    )]
    pub biometrics: Vec<BiometricSample>,
    /// Parent job ID.
    #[serde(default)]
    pub parent_job_id: Option<String>,
    /// Baseline score.
    #[serde(default)]
    pub baseline_score: Option<f32>,
    /// Parent job IDs.
    #[serde(
        default,
        deserialize_with = "crate::serde_utils::deserialize_limited_vec"
    )]
    pub parents: Vec<String>,
}

impl Default for JobConfig {
    fn default() -> Self {
        Self {
            definition: KeyboardDefinition::default(),
            weights: ScoringWeights::default(),
            params: SearchParams::default(),
            pinned_keys: vec![],
            corpora: default_corpora(),
            cost_matrix: default_cost_matrix(),
            biometrics: vec![],
            parent_job_id: None,
            baseline_score: None,
            parents: vec![],
        }
    }
}

impl JobConfig {
    /// Generates a unique Job ID by hashing the configuration.
    ///
    /// # Errors
    /// Returns an error if the layout geometry or configuration parts are invalid.
    pub fn id(&self) -> Result<String, String> {
        let primary_corpus = self
            .corpora
            .first()
            .map_or(keyforge_model::constants::DEFAULT_CORPUS_ID, |c| {
                c.id.as_str()
            });

        keyforge_model::job::JobIdentifier::from_parts(
            &self.definition.geometry,
            &self.weights,
            &self.params,
            &self.pinned_keys,
            primary_corpus,
            &self.cost_matrix,
        )
        .map(|ident| ident.hash)
        .map_err(|e| e.to_string())
    }
}

impl Validator for JobConfig {
    fn validate(&self) -> Result<(), String> {
        self.weights.validate()?;
        self.params.validate()?;
        self.definition.validate()?;

        for (i, corpus) in self.corpora.iter().enumerate() {
            corpus.validate().map_err(|e| format!("Corpus #{i}: {e}"))?;
        }

        if self.pinned_keys.len() > keyforge_model::constants::MAX_PINNED_KEYS_COUNT {
            return Err("Pinned keys configuration too large".to_string());
        }
        if self.biometrics.len() > constants::MAX_BIOMETRIC_SAMPLES {
            return Err(format!(
                "Too many biometric samples (Limit: {})",
                constants::MAX_BIOMETRIC_SAMPLES
            ));
        }
        for (i, sample) in self.biometrics.iter().enumerate() {
            sample
                .validate()
                .map_err(|e| format!("Biometric #{i}: {e}"))?;
        }

        match &self.cost_matrix {
            CostMatrixSource::Predefined(s) => {
                if s.trim().is_empty() {
                    return Err("Predefined cost matrix filename cannot be empty".to_string());
                }
            }
        }

        let key_count = self.definition.geometry.keys.len();
        for (i, constraint) in self.pinned_keys.iter().enumerate() {
            if (constraint.index.0 as usize) >= key_count {
                return Err(format!(
                    "Constraint #{} index {} out of bounds (max {})",
                    i,
                    constraint.index,
                    key_count - 1
                ));
            }
        }
        Ok(())
    }
}

/// Request to initiate a new optimization job.
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct JobRequest {
    /// Protocol version.
    #[serde(default = "default_version")]
    pub version: u32,
    /// The job configuration.
    #[serde(flatten)]
    pub config: JobConfig,
}

impl Default for JobRequest {
    fn default() -> Self {
        Self {
            version: PROTOCOL_VERSION,
            config: JobConfig::default(),
        }
    }
}

impl Validator for JobRequest {
    fn validate(&self) -> Result<(), String> {
        self.config.validate()
    }
}

impl std::ops::Deref for JobRequest {
    type Target = JobConfig;
    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

impl std::ops::DerefMut for JobRequest {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.config
    }
}

impl From<JobRequest> for JobConfig {
    fn from(req: JobRequest) -> Self {
        req.config
    }
}

/// Response confirming job submission.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct JobResponse {
    /// The assigned Job ID.
    pub job_id: String,
    /// Whether this is a new job (true) or existing (false).
    pub is_new: bool,
}

/// Response for a worker polling the queue.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct JobQueueResponse {
    /// The Job ID to process, if any.
    pub job_id: Option<String>,
    /// The configuration for the job, if any.
    pub config: Option<JobConfig>,
}

/// Submission of a result from a worker.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct ResultSubmission {
    /// Protocol version.
    #[serde(default = "default_version")]
    pub version: u32,
    /// The Job ID.
    pub job_id: String,
    /// The layout string.
    pub layout: String,
    /// The score achieved.
    pub score: f32,
    /// Timestamp of the result.
    pub timestamp: u64,
    /// Nonce for cryptographic verification.
    pub nonce: u64,
    /// The Node ID of the worker.
    pub node_id: String,
    /// Cryptographic signature (Mandatory for server-side acceptance).
    pub signature: String,
}

impl Validator for ResultSubmission {
    fn validate(&self) -> Result<(), String> {
        if self.job_id.trim().is_empty() {
            return Err("job_id cannot be empty".into());
        }
        if self.node_id.trim().is_empty() {
            return Err("node_id cannot be empty".into());
        }
        if self.score.is_nan() || self.score < 0.0 {
            return Err("Invalid score".into());
        }

        // Layout structure check
        LayoutValidator::validate_structure(&self.layout)?;

        // Clock skew check
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if self.timestamp > now + constants::MAX_FUTURE_SKEW_SEC {
            return Err("Timestamp too far in the future (Clock skew?)".into());
        }
        if self.timestamp < now - constants::MAX_PAST_SKEW_SEC {
            return Err("Timestamp too old (Stale result)".into());
        }

        Ok(())
    }
}

/// Status of a specific job.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct JobStatus {
    /// The Job ID.
    pub job_id: String,
    /// Current status (e.g., "Running").
    pub status: String,
    /// Number of nodes working on this job.
    pub active_nodes: usize,
    /// Best score found so far.
    pub best_score: Option<f32>,
    /// Best layout found so far.
    pub best_layout: Option<String>,
    /// Total samples processed.
    pub total_samples: usize,
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;
    use crate::PROTOCOL_VERSION;
    use keyforge_model::KeyNode;

    #[test]
    fn test_job_request_serialization() {
        let req = JobRequest {
            version: PROTOCOL_VERSION,
            config: JobConfig {
                definition: KeyboardDefinition::default(),
                weights: ScoringWeights::default(),
                params: SearchParams::default(),
                pinned_keys: vec![],
                corpora: vec![],
                cost_matrix: CostMatrixSource::default(),
                biometrics: vec![],
                parent_job_id: None,
                baseline_score: None,
                parents: vec![],
            },
        };

        let json = serde_json::to_string(&req).expect("Failed to serialize");
        let deserialized: JobRequest = serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(req.version, deserialized.version);
    }

    #[test]
    fn test_biometric_limit_validation() {
        let mut req = JobRequest {
            version: PROTOCOL_VERSION,
            config: JobConfig {
                definition: KeyboardDefinition::default(),
                weights: ScoringWeights::default(),
                params: SearchParams::default(),
                pinned_keys: vec![],
                corpora: vec![],
                cost_matrix: CostMatrixSource::default(),
                biometrics: vec![],
                parent_job_id: None,
                baseline_score: None,
                parents: vec![],
            },
        };
        req.config.definition.geometry.keys.push(KeyNode::default());
        req.config.definition.geometry.home_row = 0;
        req.config
            .definition
            .geometry
            .prime_slots
            .push(keyforge_model::KeyIndex(0));

        // Fill up to limit
        req.config.biometrics = (0..constants::MAX_BIOMETRIC_SAMPLES)
            .map(|i| BiometricSample {
                bigram: "th".to_string(),
                ms: 100.0,
                timestamp: i as u64,
            })
            .collect();
        assert!(req.validate().is_ok());

        // One too many
        req.config.biometrics.push(BiometricSample {
            bigram: "xx".to_string(),
            ms: 0.0,
            timestamp: 0,
        });
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_result_submission_timestamp() {
        let mut sub = ResultSubmission {
            version: PROTOCOL_VERSION,
            job_id: "test".into(),
            layout: "A B C D E F G H I J".into(), // Valid layout
            score: 100.0,
            timestamp: 0,
            nonce: 0,
            node_id: "node".into(),
            signature: "dummy_sig".into(),
        };

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // 1. Current time is valid
        sub.timestamp = now;
        assert!(sub.validate().is_ok());

        // 2. Future check
        sub.timestamp = now + constants::MAX_FUTURE_SKEW_SEC + 100;
        assert!(sub.validate().is_err());

        // 3. Old check
        sub.timestamp = now - constants::MAX_PAST_SKEW_SEC - 100;
        assert!(sub.validate().is_err());
    }

    #[test]
    fn test_job_config_conversion() {
        let req = JobRequest {
            version: 1,
            config: JobConfig {
                definition: KeyboardDefinition::default(),
                weights: ScoringWeights::default(),
                params: SearchParams::default(),
                pinned_keys: vec![KeyConstraint {
                    index: keyforge_model::KeyIndex(0),
                    key: "A".into(),
                }],
                corpora: vec![CorpusSource::default()],
                cost_matrix: CostMatrixSource::default(),
                biometrics: vec![],
                parent_job_id: Some("parent".into()),
                baseline_score: Some(100.0),
                parents: vec!["p1".into()],
            },
        };
        let config: JobConfig = req.clone().into();
        assert_eq!(config.pinned_keys.len(), 1);
        assert_eq!(config.parent_job_id, Some("parent".into()));
    }

    #[test]
    fn test_job_request_validation_logic() {
        let mut req = JobRequest {
            version: 1,
            config: JobConfig {
                definition: KeyboardDefinition::default(),
                weights: ScoringWeights::default(),
                params: SearchParams::default(),
                pinned_keys: vec![],
                corpora: vec![],
                cost_matrix: CostMatrixSource::default(),
                biometrics: vec![],
                parent_job_id: None,
                baseline_score: None,
                parents: vec![],
            },
        };

        req.config.definition.geometry.keys = vec![KeyNode::default()];
        req.config.definition.geometry.home_row = 0;
        req.config.definition.geometry.prime_slots = vec![keyforge_model::KeyIndex(0)];

        // 1. Too many keys
        let original_keys = req.config.definition.geometry.keys.clone();
        req.config.definition.geometry.keys = vec![KeyNode::default(); 201];
        assert!(req.validate().is_err(), "Should reject > 200 keys");
        req.config.definition.geometry.keys = original_keys;

        // 2. Pinned key out of bounds
        req.config.pinned_keys = vec![KeyConstraint {
            index: keyforge_model::KeyIndex(5),
            key: "A".into(),
        }];
        assert!(req.validate().is_err(), "Should reject out of bounds pin");
        req.config.pinned_keys.clear();

        req.config.cost_matrix = CostMatrixSource::Predefined(" ".into());
        assert!(
            req.validate().is_err(),
            "Should reject empty cost matrix name"
        );
    }

    #[test]
    fn test_job_config_id_generation() {
        let config = JobConfig::default();
        let id = config.id().unwrap();
        assert!(!id.is_empty());
    }

    #[test]
    fn test_result_submission_validation_extended() {
        let sub = ResultSubmission {
            version: PROTOCOL_VERSION,
            job_id: "test".into(),
            layout: "A B C D E F G H I J".into(),
            score: 100.0,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            nonce: 0,
            node_id: "node".into(),
            signature: "sig".into(),
        };
        assert!(sub.validate().is_ok());

        // Empty IDs
        let mut invalid = sub.clone();
        invalid.job_id = " ".into();
        assert!(invalid.validate().is_err());

        let mut invalid = sub.clone();
        invalid.node_id = " ".into();
        assert!(invalid.validate().is_err());

        // Invalid score
        let mut invalid = sub.clone();
        invalid.score = f32::NAN;
        assert!(invalid.validate().is_err());
        invalid.score = -1.0;
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_job_request_deref() {
        let mut req = JobRequest::default();
        assert_eq!(req.version, PROTOCOL_VERSION);
        // Deref to JobConfig
        assert!(!req.corpora.is_empty());
        // DerefMut
        req.parent_job_id = Some("p".into());
        assert_eq!(req.config.parent_job_id, Some("p".into()));
    }

    #[test]
    fn test_job_config_validation_extended() {
        let mut config = JobConfig::default();
        config.definition.geometry.keys.push(KeyNode::default());
        config.definition.geometry.home_row = 0;
        config
            .definition
            .geometry
            .prime_slots
            .push(keyforge_model::KeyIndex(0));

        // Invalid corpus
        config.corpora[0].id = " ".into();
        assert!(config.validate().is_err());
        config.corpora[0].id = "en".into();

        // Too many pins
        config.pinned_keys = vec![
            KeyConstraint {
                index: keyforge_model::KeyIndex(0),
                key: "A".into()
            };
            201
        ];
        assert!(config.validate().is_err());
    }
}

#[cfg(test)]
mod fuzz {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn fuzz_json_deserialization(s in "\\PC*") {
            let _ = serde_json::from_str::<JobRequest>(&s);
        }
    }
}
