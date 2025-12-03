use keyforge_core::config::ScoringWeights;
use keyforge_core::geometry::KeyboardGeometry;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct JobQueueResponse {
    pub job_id: Option<String>,
    pub config: Option<JobConfig>,
}

#[derive(Deserialize)]
pub struct JobConfig {
    pub geometry: KeyboardGeometry,
    pub weights: ScoringWeights,
    pub pinned_keys: String,
    pub corpus_name: String,
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