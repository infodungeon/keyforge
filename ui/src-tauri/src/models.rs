use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// RE-EXPORTS from Protocol
pub use keyforge_protocol::{
    JobConfig, PopulationResponse, RegisterJobRequest, RegisterJobResponse, RegisterNodeRequest,
    RegisterNodeResponse, SubmitResultRequest, TuningProfile,
};

// UI-Specific Models (Not shared with Hive/Node)

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
    // We use Protocol types for params/weights now
    pub search_params: keyforge_protocol::config::SearchParams,
    pub weights: keyforge_protocol::config::ScoringWeights,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct UserLayoutStore {
    // HashMap<KeyboardID, HashMap<LayoutName, LayoutString>>
    pub layouts: HashMap<String, HashMap<String, String>>,
}

#[derive(Deserialize)]
pub struct ServerManifest {
    pub files: HashMap<String, String>,
}

#[derive(Serialize, Clone)]
pub struct SyncStats {
    pub downloaded: usize,
    pub merged: usize,
    pub skipped: usize,
    pub errors: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BiometricSample {
    pub bigram: String,
    pub ms: f64,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct UserStatsStore {
    pub sessions: u64,
    pub total_keystrokes: u64,
    pub biometrics: Vec<BiometricSample>,
}
