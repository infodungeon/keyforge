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

use crate::PROTOCOL_VERSION;
use crate::constants;
use crate::assets::BiometricSample;
use keyforge_model::{
    CorpusSource, CostMatrixSource, KeyConstraint, KeyboardDefinition, ScoringWeights, SearchParams,
    Validator, LayoutValidator,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

fn default_version() -> u32 { PROTOCOL_VERSION }
fn default_cost_matrix() -> CostMatrixSource { CostMatrixSource::default() }
fn default_corpora() -> Vec<CorpusSource> { vec![CorpusSource::default()] }

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
    #[serde(default, deserialize_with = "crate::serde_utils::deserialize_limited_vec")]
    pub pinned_keys: Vec<KeyConstraint>,
    /// Text corpora to use.
    #[serde(default = "default_corpora", deserialize_with = "crate::serde_utils::deserialize_limited_vec")]
    pub corpora: Vec<CorpusSource>,
    /// Cost matrix source.
    #[serde(default = "default_cost_matrix")]
    pub cost_matrix: CostMatrixSource,
    /// User biometric data.
    #[serde(default, skip_serializing_if = "Vec::is_empty", deserialize_with = "crate::serde_utils::deserialize_limited_vec")]
    pub biometrics: Vec<BiometricSample>,
    /// Parent job ID.
    #[serde(default)]
    pub parent_job_id: Option<String>,
    /// Baseline score.
    #[serde(default)]
    pub baseline_score: Option<f32>,
    /// Parent job IDs.
    #[serde(default, deserialize_with = "crate::serde_utils::deserialize_limited_vec")]
    pub parents: Vec<String>,
}

impl JobConfig {
    /// Generates a unique Job ID by hashing the configuration.
    pub fn id(&self) -> Result<String, String> {
        let primary_corpus = self.corpora.first()
            .map(|c| c.id.as_str())
            .unwrap_or(keyforge_model::constants::DEFAULT_CORPUS_ID);

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
            corpus.validate().map_err(|e| format!("Corpus #{}: {}", i, e))?;
        }

        if self.definition.geometry.keys.len() > keyforge_model::constants::MAX_KEYBOARD_KEYS {
            return Err(format!("Geometry exceeds maximum key limit ({})", keyforge_model::constants::MAX_KEYBOARD_KEYS));
        }
        if self.pinned_keys.len() > keyforge_model::constants::MAX_PINNED_KEYS_COUNT {
            return Err("Pinned keys configuration too large".to_string());
        }
        if self.biometrics.len() > constants::MAX_BIOMETRIC_SAMPLES {
            return Err(format!("Too many biometric samples (Limit: {})", constants::MAX_BIOMETRIC_SAMPLES));
        }
        for (i, sample) in self.biometrics.iter().enumerate() {
            sample.validate().map_err(|e| format!("Biometric #{}: {}", i, e))?;
        }

        match &self.cost_matrix {
            CostMatrixSource::Predefined(s) => {
                if s.trim().is_empty() { return Err("Predefined cost matrix filename cannot be empty".to_string()); }
            }
        }

        let key_count = self.definition.geometry.keys.len();
        for (i, constraint) in self.pinned_keys.iter().enumerate() {
            if (constraint.index.0 as usize) >= key_count {
                return Err(format!("Constraint #{} index {} out of bounds (max {})", i, constraint.index, key_count - 1));
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
        if self.job_id.trim().is_empty() { return Err("job_id cannot be empty".into()); }
        if self.node_id.trim().is_empty() { return Err("node_id cannot be empty".into()); }
        if self.score.is_nan() || self.score < 0.0 { return Err("Invalid score".into()); }
        
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
