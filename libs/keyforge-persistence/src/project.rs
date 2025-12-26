use keyforge_protocol::{
    config::{CorpusSource, ScoringWeights, SearchParams},
    CostMatrixSource, KeyConstraint,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMeta {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub author: String,
}

impl Default for ProjectMeta {
    fn default() -> Self {
        Self {
            name: "Untitled Project".to_string(),
            version: "0.1.0".to_string(),
            author: "Anonymous".to_string(),
        }
    }
}

/// A persistable definition of an optimization experiment.
/// Contains all "ingredients" needed to compile a Runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    #[serde(default)]
    pub meta: ProjectMeta,

    /// Name or Path of the keyboard definition (e.g. "corne", "ansi_104")
    pub keyboard: String,

    /// List of corpora to blend
    pub corpora: Vec<CorpusSource>,

    /// Scoring parameters
    pub weights: ScoringWeights,

    /// Search configuration (annealing steps, etc.)
    pub params: SearchParams,

    /// User-defined constraints (pinned keys)
    #[serde(default)]
    pub constraints: Vec<KeyConstraint>,

    /// Source for the cost matrix (biomechanical profile)
    #[serde(default)]
    pub cost_matrix: CostMatrixSource,

    /// Optional seed for deterministic reproducibility
    #[serde(default)]
    pub seed: Option<u64>,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            meta: ProjectMeta::default(),
            keyboard: "ortho_30".to_string(),
            corpora: vec![CorpusSource::default()],
            weights: ScoringWeights::default(),
            params: SearchParams::default(),
            constraints: Vec::new(),
            cost_matrix: CostMatrixSource::default(),
            seed: None,
        }
    }
}
