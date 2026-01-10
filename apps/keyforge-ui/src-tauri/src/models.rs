use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// RE-EXPORTS from Protocol
pub use keyforge_protocol::{
    BiometricSample, JobConfig, JobRequest as RegisterJobRequest,
    JobResponse as RegisterJobResponse, NodeRequest as RegisterNodeRequest,
    NodeResponse as RegisterNodeResponse, PopulationResponse,
    ResultSubmission as SubmitResultRequest, TuningProfile, UserStatsStore,
};

/// Periodic status update for a remote job running on the Hive.
#[derive(Clone, Serialize, Debug)]
pub struct JobStatusUpdate {
    /// Number of active compute nodes contributing to this job.
    pub active_nodes: usize,
    /// The best (lowest) physical cost score found so far.
    pub best_score: f32,
    /// The serialized layout string of the current best result.
    pub best_layout: String,
}

/// Real-time search update from a local worker or optimizer.
#[derive(Clone, Serialize, Debug)]
pub struct SearchUpdate {
    /// The current optimization epoch (iteration count).
    pub epoch: usize,
    /// The current best score achieved in this search.
    pub score: f32,
    /// The current best layout found by this search.
    pub layout: String,
    /// Search throughput in "Items Per Second".
    pub ips: f32,
}

/// Request to start a new local search operation.
#[derive(Deserialize, Debug)]
pub struct StartSearchRequest {
    /// Encoded string representing pinned keys (positions that shouldn't move).
    pub pinned_keys: String,
    /// Configuration parameters for the optimization algorithm.
    pub search_params: keyforge_model::config::SearchParams,
    /// Ergonomic weights used to calculate the physical score.
    pub weights: keyforge_model::config::ScoringWeights,
}

/// Manifest of files available on a remote server.
#[derive(Deserialize, Debug)]
pub struct ServerManifest {
    /// Map of relative file paths to their content hashes or metadata.
    pub files: HashMap<String, String>,
}

/// Statistics gathered during a data synchronization operation.
#[derive(Serialize, Clone, Debug)]
pub struct SyncStats {
    /// Number of new files downloaded.
    pub downloaded: usize,
    /// Number of files merged into the local repository.
    pub merged: usize,
    /// Number of files skipped (already up-to-date).
    pub skipped: usize,
    /// List of error messages encountered during sync.
    pub errors: Vec<String>,
}

/// Result of a layout validation and analysis operation.
#[derive(Serialize, Clone, Debug)]
pub struct ValidationResult {
    /// Name of the layout that was validated.
    pub layout_name: String,
    /// Comprehensive analysis report including ergonomics metrics.
    pub score: keyforge_model::AnalysisReport,
    /// Physical geometry of the keyboard used for analysis.
    pub geometry: keyforge_model::geometry::KeyboardGeometry,
    /// Frequency heatmap indicating which keys are used most relative to their position.
    pub heatmap: Vec<f32>,
    /// Map of ergonomic penalties applied across the layout.
    pub penalty_map: Vec<f32>,
}

/// Statistics derived from raw analysis data for UI display.
#[derive(Serialize, Clone, Debug)]
pub struct DerivedStats {
    /// Calculated balance between left and right hand usage (0.0 to 1.0).
    pub hand_balance: f32,
}
