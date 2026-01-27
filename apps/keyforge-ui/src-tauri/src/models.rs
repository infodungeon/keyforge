use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// RE-EXPORTS from Protocol
pub use keyforge_protocol::{
    AnalysisReportDto, BiometricSample, ConfigAggregateDto, DerivedStatsDto, JobConfig,
    JobRequest as RegisterJobRequest, JobResponse as RegisterJobResponse, KeyNodeDto, LayoutDto,
    NodeRequest as RegisterNodeRequest, NodeResponse as RegisterNodeResponse, PopulationResponse,
    ResultSubmission as SubmitResultRequest, ScoringWeightsDto, SearchParamsDto, SwapSuggestionDto,
    TuningProfile, UserStatsStore, ValidationResultDto,
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
    pub search_params: SearchParamsDto,
    /// Ergonomic weights used to calculate the physical score.
    pub weights: ScoringWeightsDto,
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

/// Result of a layout validation and analysis operation (Legacy alias).
pub type ValidationResult = ValidationResultDto;

/// Statistics derived from raw analysis data for UI display (Legacy alias).
pub type DerivedStats = DerivedStatsDto;
