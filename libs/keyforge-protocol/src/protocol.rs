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
use ts_rs::TS;

fn default_version() -> u32 { PROTOCOL_VERSION }
fn default_cost_matrix() -> CostMatrixSource { CostMatrixSource::default() }
fn default_corpora() -> Vec<CorpusSource> { vec![CorpusSource::default()] }

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema, TS)]
#[ts(export)]
pub struct BiometricSample {
    pub bigram: String,
    pub ms: f64,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, ToSchema, TS)]
#[ts(export)]
pub struct UserStatsStore {
    pub sessions: u64,
    pub total_keystrokes: u64,
    pub biometrics: Vec<BiometricSample>,
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema, TS)]
#[ts(export)]
pub struct JobRequest {
    #[serde(default = "default_version")]
    pub version: u32,
    pub definition: KeyboardDefinition,
    pub weights: ScoringWeights,
    pub params: SearchParams,
    #[serde(default)]
    pub pinned_keys: Vec<KeyConstraint>,
    #[serde(default = "default_corpora")]
    pub corpora: Vec<CorpusSource>,
    #[serde(default = "default_cost_matrix")]
    pub cost_matrix: CostMatrixSource,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub biometrics: Vec<BiometricSample>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_job_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub baseline_score: Option<f32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
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

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema, TS)]
#[ts(export)]
pub struct JobConfig {
    pub definition: KeyboardDefinition,
    pub weights: ScoringWeights,
    pub params: SearchParams,
    #[serde(default)]
    pub pinned_keys: Vec<KeyConstraint>,
    #[serde(default = "default_corpora")]
    pub corpora: Vec<CorpusSource>,
    #[serde(default = "default_cost_matrix")]
    pub cost_matrix: CostMatrixSource,
    #[serde(default)]
    pub biometrics: Vec<BiometricSample>,
    #[serde(default)]
    pub parent_job_id: Option<String>,
    #[serde(default)]
    pub baseline_score: Option<f32>,
    #[serde(default)]
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

#[derive(Serialize, Deserialize, Debug, ToSchema, TS)]
#[ts(export)]
pub struct JobResponse {
    pub job_id: String,
    pub is_new: bool,
}

#[derive(Serialize, Deserialize, Debug, ToSchema, TS)]
#[ts(export)]
pub struct JobQueueResponse {
    pub job_id: Option<String>,
    pub config: Option<JobConfig>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema, TS)]
#[ts(export)]
pub struct PopulationResponse {
    pub layouts: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema, TS)]
#[ts(export)]
pub struct ResultSubmission {
    #[serde(default = "default_version")]
    pub version: u32,
    pub job_id: String,
    pub layout: String,
    pub score: f32,
    pub timestamp: u64,
    pub nonce: u64,
    pub node_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema, TS)]
#[ts(export)]
pub struct NodeRequest {
    #[serde(default = "default_version")]
    pub version: u32,
    pub node_id: String,
    pub cpu_model: String,
    pub cores: i32,
    pub l2_cache_kb: Option<i32>,
    pub ops_per_sec: f32,
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

#[derive(Serialize, Deserialize, Debug, ToSchema, TS)]
#[ts(export)]
pub struct TuningProfile {
    pub strategy: String,
    pub batch_size: usize,
    pub thread_count: usize,
}

#[derive(Serialize, Deserialize, Debug, ToSchema, TS)]
#[ts(export)]
pub struct NodeResponse {
    pub status: String,
    pub tuning: TuningProfile,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, ToSchema, TS)]
#[ts(export)]
pub struct SystemMetrics {
    pub uptime_secs: u64,
    pub active_jobs: i64,
    pub total_results: i64,
    pub nodes_online: i64,
    pub total_ops_per_sec: f32,
    pub server_memory_used: u64,
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

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema, TS)]
#[ts(export)]
pub struct JobStatus {
    pub job_id: String,
    pub status: String,
    pub active_nodes: usize,
    pub best_score: Option<f32>,
    pub best_layout: Option<String>,
    pub total_samples: usize,
}
