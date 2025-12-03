use keyforge_core::config::{ScoringWeights, SearchParams};
use keyforge_core::geometry::KeyboardGeometry;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ... (Keep all previous structs: RegisterJobRequest, etc.)

#[derive(Serialize, Deserialize)]
pub struct RegisterJobRequest {
    pub geometry: KeyboardGeometry,
    pub weights: ScoringWeights,
    pub pinned_keys: String,
    pub corpus_name: String,
}

#[derive(Serialize, Deserialize)]
pub struct RegisterJobResponse {
    pub job_id: String,
    pub is_new: bool,
}

#[derive(Deserialize)]
pub struct PopulationResponse {
    pub layouts: Vec<String>,
}

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
    pub search_params: SearchParams,
    pub weights: ScoringWeights,
}

// FIXED: Scoped layouts by keyboard name
// HashMap<KeyboardName, HashMap<LayoutName, LayoutString>>
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct UserLayoutStore {
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
