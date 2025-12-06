use crate::config::{ScoringWeights, SearchParams};
use crate::geometry::KeyboardDefinition;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RegisterJobRequest {
    pub definition: KeyboardDefinition,
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JobConfig {
    pub definition: KeyboardDefinition,
    pub weights: ScoringWeights,
    pub params: SearchParams,
    pub pinned_keys: String,
    pub corpus_name: String,
    #[serde(default = "default_cost_matrix")]
    pub cost_matrix: String,
}

impl From<RegisterJobRequest> for JobConfig {
    fn from(req: RegisterJobRequest) -> Self {
        Self {
            definition: req.definition,
            weights: req.weights,
            params: req.params,
            pinned_keys: req.pinned_keys,
            corpus_name: req.corpus_name,
            cost_matrix: req.cost_matrix,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterJobResponse {
    pub job_id: String,
    pub is_new: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JobQueueResponse {
    pub job_id: Option<String>,
    pub config: Option<JobConfig>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PopulationResponse {
    pub layouts: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubmitResultRequest {
    pub job_id: String,
    pub layout: String,
    pub score: f32,
    pub node_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterNodeRequest {
    pub node_id: String,
    pub cpu_model: String,
    pub cores: i32,
    pub l2_cache_kb: Option<i32>,
    pub ops_per_sec: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TuningProfile {
    pub strategy: String,
    pub batch_size: usize,
    pub thread_count: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterNodeResponse {
    pub status: String,
    pub tuning: TuningProfile,
}
