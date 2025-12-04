// ===== keyforge/crates/keyforge-node/src/models.rs =====
use keyforge_core::config::{ScoringWeights, SearchParams};
use keyforge_core::geometry::KeyboardDefinition;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct JobQueueResponse {
    pub job_id: Option<String>,
    pub config: Option<JobConfig>,
}

#[derive(Deserialize)]
pub struct JobConfig {
    pub definition: KeyboardDefinition, // Updated to match core
    pub weights: ScoringWeights,
    pub params: SearchParams,
    pub pinned_keys: String,
    pub corpus_name: String,
    #[serde(default = "default_cost_matrix")]
    pub cost_matrix: String,
}

fn default_cost_matrix() -> String {
    "cost_matrix.csv".to_string()
}

#[derive(Deserialize)]
pub struct PopulationResponse {
    pub layouts: Vec<String>,
}

#[derive(Serialize)]
pub struct SubmitResultRequest {
    pub job_id: String,
    pub layout: String,
    pub score: f32,
    pub node_id: String,
}
