// Copyright (c) 2025 KeyForge Contributors
//
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
use keyforge_model::{
    CorpusSource, CostMatrixSource, KeyConstraint, KeyboardDefinition, ScoringWeights, SearchParams,
    Validator, LayoutValidator, constants,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

fn default_version() -> u32 { PROTOCOL_VERSION }
fn default_cost_matrix() -> CostMatrixSource { CostMatrixSource::default() }
fn default_corpora() -> Vec<CorpusSource> { vec![CorpusSource::default()] }

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

/// Aggregated statistics for a user.
#[derive(Serialize, Deserialize, Clone, Default, Debug, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct UserStatsStore {
    /// Total number of sessions.
    pub sessions: u64,
    /// Total keystrokes typed.
    pub total_keystrokes: u64,
    /// Collection of biometric samples.
    #[serde(deserialize_with = "keyforge_model::serde_utils::deserialize_limited_vec")]
    pub biometrics: Vec<BiometricSample>,
}

/// Request to initiate a new optimization job.
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct JobRequest {
    /// Protocol version.
    #[serde(default = "default_version")]
    pub version: u32,
    /// Keyboard geometry definition.
    pub definition: KeyboardDefinition,
    /// Scoring weights.
    pub weights: ScoringWeights,
    /// Search parameters.
    pub params: SearchParams,
    /// Keys pinned to specific positions.
    #[serde(default, deserialize_with = "keyforge_model::serde_utils::deserialize_limited_vec")]
    pub pinned_keys: Vec<KeyConstraint>,
    /// Text corpora to use.
    #[serde(default = "default_corpora", deserialize_with = "keyforge_model::serde_utils::deserialize_limited_vec")]
    pub corpora: Vec<CorpusSource>,
    /// Cost matrix source.
    #[serde(default = "default_cost_matrix")]
    pub cost_matrix: CostMatrixSource,
    /// User biometric data.
    #[serde(default, skip_serializing_if = "Vec::is_empty", deserialize_with = "keyforge_model::serde_utils::deserialize_limited_vec")]
    pub biometrics: Vec<BiometricSample>,
    /// Parent job ID (for evolution).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_job_id: Option<String>,
    /// Baseline score to beat.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub baseline_score: Option<f32>,
    /// Parent job IDs (for merging).
    #[serde(default, skip_serializing_if = "Vec::is_empty", deserialize_with = "keyforge_model::serde_utils::deserialize_limited_vec")]
    pub parents: Vec<String>,
}

impl Validator for JobRequest {
    fn validate(&self) -> Result<(), String> {
        self.weights.validate()?;
        self.params.validate()?;
        self.definition.geometry.validate()?;

        if self.definition.geometry.keys.len() > constants::MAX_KEYBOARD_KEYS {
            return Err(format!("Geometry exceeds maximum key limit ({})", constants::MAX_KEYBOARD_KEYS));
        }
        if self.pinned_keys.len() > constants::MAX_PINNED_KEYS_COUNT {
            return Err("Pinned keys configuration too large".to_string());
        }
        if self.biometrics.len() > 10_000 {
            return Err("Too many biometric samples (Limit: 10,000)".to_string());
        }

        match &self.cost_matrix {
            CostMatrixSource::Predefined(s) => {
                if s.trim().is_empty() { return Err("Predefined cost matrix filename cannot be empty".to_string()); }
            }
            CostMatrixSource::Custom(s) => {
                if s.trim().is_empty() { return Err("Custom cost matrix content cannot be empty".to_string()); }
                if !s.contains(',') { return Err("Custom cost matrix does not appear to be valid CSV".to_string()); }
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
    #[serde(default, deserialize_with = "keyforge_model::serde_utils::deserialize_limited_vec")]
    pub pinned_keys: Vec<KeyConstraint>,
    /// Text corpora to use.
    #[serde(default = "default_corpora", deserialize_with = "keyforge_model::serde_utils::deserialize_limited_vec")]
    pub corpora: Vec<CorpusSource>,
    /// Cost matrix source.
    #[serde(default = "default_cost_matrix")]
    pub cost_matrix: CostMatrixSource,
    /// User biometric data.
    #[serde(default, deserialize_with = "keyforge_model::serde_utils::deserialize_limited_vec")]
    pub biometrics: Vec<BiometricSample>,
    /// Parent job ID.
    #[serde(default)]
    pub parent_job_id: Option<String>,
    /// Baseline score.
    #[serde(default)]
    pub baseline_score: Option<f32>,
    /// Parent job IDs.
    #[serde(default, deserialize_with = "keyforge_model::serde_utils::deserialize_limited_vec")]
    pub parents: Vec<String>,
}

impl From<JobRequest> for JobConfig {
    fn from(req: JobRequest) -> Self {
        Self {
            definition: req.definition,
            weights: req.weights,
            params: req.params,
            pinned_keys: req.pinned_keys,
            corpora: req.corpora,
            cost_matrix: req.cost_matrix,
            biometrics: req.biometrics,
            parent_job_id: req.parent_job_id,
            baseline_score: req.baseline_score,
            parents: req.parents,
        }
    }
}

/// Response confirming job submission.
#[derive(Serialize, Deserialize, Debug, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct JobResponse {
    /// The assigned Job ID.
    pub job_id: String,
    /// Whether this is a new job (true) or existing (false).
    pub is_new: bool,
}

/// Response for a worker polling the queue.
#[derive(Serialize, Deserialize, Debug, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct JobQueueResponse {
    /// The Job ID to process, if any.
    pub job_id: Option<String>,
    /// The configuration for the job, if any.
    pub config: Option<JobConfig>,
}

/// Response containing available layouts.
#[derive(Serialize, Deserialize, Debug, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct PopulationResponse {
    /// List of layout strings.
    #[serde(deserialize_with = "keyforge_model::serde_utils::deserialize_limited_vec")]
    pub layouts: Vec<String>,
}

/// Submission of a result from a worker.
#[derive(Serialize, Deserialize, Debug, ToSchema)]
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
    /// Cryptographic signature.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

/// Request from a node to register or heartbeat.
#[derive(Serialize, Deserialize, Debug, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct NodeRequest {
    /// Protocol version.
    #[serde(default = "default_version")]
    pub version: u32,
    /// The Node ID.
    pub node_id: String,
    /// CPU model name.
    pub cpu_model: String,
    /// Number of cores.
    pub cores: i32,
    /// L2 cache size in KB.
    pub l2_cache_kb: Option<i32>,
    /// Operations per second benchmark.
    pub ops_per_sec: f32,
    /// Public key for verification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key: Option<String>,
}

impl Validator for NodeRequest {
    fn validate(&self) -> Result<(), String> {
        if self.node_id.trim().is_empty() { return Err("node_id cannot be empty".into()); }
        if self.cores <= 0 { return Err("cores must be > 0".into()); }
        if self.ops_per_sec < 0.0 { return Err("ops_per_sec cannot be negative".into()); }
        Ok(())
    }
}

/// Tuning profile for a worker.
#[derive(Serialize, Deserialize, Debug, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct TuningProfile {
    /// Strategy name.
    pub strategy: String,
    /// Batch size for processing.
    pub batch_size: usize,
    /// Number of threads to use.
    pub thread_count: usize,
}

/// Response to a node heartbeat.
#[derive(Serialize, Deserialize, Debug, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct NodeResponse {
    /// Status of the node (e.g., "Active").
    pub status: String,
    /// Tuning profile to apply.
    pub tuning: TuningProfile,
}

/// System-wide metrics.
#[derive(Serialize, Deserialize, Debug, Default, Clone, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct SystemMetrics {
    /// Uptime in seconds.
    pub uptime_secs: u64,
    /// Number of active jobs.
    pub active_jobs: i64,
    /// Total results processed.
    pub total_results: i64,
    /// Number of nodes online.
    pub nodes_online: i64,
    /// Total operations per second across the cluster.
    pub total_ops_per_sec: f32,
    /// Server memory used in bytes.
    pub server_memory_used: u64,
    /// Server CPU usage percentage.
    pub server_cpu_usage: f32,
}

impl Validator for ResultSubmission {
    fn validate(&self) -> Result<(), String> {
        if self.job_id.trim().is_empty() { return Err("job_id cannot be empty".into()); }
        if self.node_id.trim().is_empty() { return Err("node_id cannot be empty".into()); }
        if self.score.is_nan() || self.score < 0.0 { return Err("Invalid score".into()); }
        
        // Layout structure check
        LayoutValidator::validate_structure(&self.layout)?;

        // Timestamp check
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs();
        if self.timestamp > now + 300 { return Err("Timestamp is in the future".into()); }
        if self.timestamp < now.saturating_sub(1800) { return Err("Timestamp is too old".into()); }
        
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
