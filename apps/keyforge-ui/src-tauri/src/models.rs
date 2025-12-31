use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// RE-EXPORTS from Protocol
pub use keyforge_protocol::{
    BiometricSample, JobConfig, JobRequest as RegisterJobRequest,
    JobResponse as RegisterJobResponse, NodeRequest as RegisterNodeRequest,
    NodeResponse as RegisterNodeResponse, PopulationResponse,
    ResultSubmission as SubmitResultRequest, TuningProfile, UserStatsStore,
};

#[derive(Clone, Serialize)]
pub struct JobStatusUpdate {
    pub active_nodes: usize,
    pub best_score: f32,
    pub best_layout: String,
}

#[derive(Clone, Serialize)]
pub struct SearchUpdate {
    pub epoch: usize,
    pub score: f32,
    pub layout: String,
    pub ips: f32,
}

#[derive(Deserialize)]
pub struct StartSearchRequest {
    pub pinned_keys: String,
    pub search_params: keyforge_model::config::SearchParams,
    pub weights: keyforge_model::config::ScoringWeights,
}

#[derive(Deserialize)]
pub struct ServerManifest {
    pub files: HashMap<String, String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct SyncStats {
    pub downloaded: usize,
    pub merged: usize,
    pub skipped: usize,
    pub errors: Vec<String>,
}

#[derive(Serialize, Clone)]
pub struct ValidationResult {
    pub layout_name: String,
    pub score: keyforge_model::AnalysisReport,
    pub geometry: keyforge_model::geometry::KeyboardGeometry,
    pub heatmap: Vec<f32>,
    pub penalty_map: Vec<f32>,
}

#[derive(Serialize, Clone)]
pub struct DerivedStats {
    pub hand_balance: f32,
}
