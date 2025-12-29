pub mod config;
pub mod constants;
pub mod corpus;
pub mod error;
pub mod geometry;
pub mod job;
pub mod keyboard;
pub mod keycodes;
pub mod layout;
pub mod loader;
pub mod parsing;
pub mod rubric;
pub mod serde_utils;
pub mod types;
pub mod validator;

pub use config::{Config, CorpusSource, CostMatrixSource, KeyConstraint, ScoringWeights, SearchParams};
pub use corpus::Corpus;
pub use error::ForgeError;
pub use geometry::{KeyboardDefinition, KeyboardGeometry, KeyNode};
pub use job::{JobIdentifier, JobIdError};
pub use keyboard::Keyboard;
pub use keycodes::KeycodeRegistry;
pub use layout::Layout;
pub use rubric::Rubric;
pub use types::{KeyIndex, HandIndex, FingerIndex, RowIndex, ColIndex, Score, KeyCode};
pub use validator::Validator;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchConfig {
    Annealing {
        steps: usize,
        start_temp: f32,
        end_temp: f32,
        seed: u64,
        patience: usize,
        reheats: usize,
        reheat_factor: f32,
    },
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self::Annealing {
            steps: 100_000,
            start_temp: 100.0,
            end_temp: 0.01,
            seed: 42,
            patience: 500,
            reheats: 3,
            reheat_factor: 0.5,
        }
    }
}

impl SearchConfig {
    pub fn validate(&self) -> Result<(), ForgeError> {
        match self {
            SearchConfig::Annealing {
                steps,
                start_temp,
                end_temp,
                reheat_factor,
                ..
            } => {
                if *steps == 0 {
                    return Err(ForgeError::InvalidData("Steps must be > 0".into()));
                }
                if *start_temp < 0.0 {
                    return Err(ForgeError::InvalidData("Start temp must be >= 0".into()));
                }
                if *end_temp < 0.0 {
                    return Err(ForgeError::InvalidData("End temp must be >= 0".into()));
                }
                if *reheat_factor <= 0.0 {
                    return Err(ForgeError::InvalidData("Reheat factor must be > 0".into()));
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetricViolation {
    pub keys: String,
    pub score: f32,
    pub freq: f32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnalysisReport {
    pub score: f32,
    pub distance: f32,
    pub sfb_total: f32,
    pub sfb_ratio: f32,
    pub hand_balance: f32,
    pub scissors: f32,
    pub redirects: f32,
    pub rolls: f32,
    #[serde(default)]
    pub heatmap: Vec<f32>,
    #[serde(default)]
    pub top_sfbs: Vec<MetricViolation>,
    #[serde(default)]
    pub top_scissors: Vec<MetricViolation>,
    #[serde(default)]
    pub top_redirs: Vec<MetricViolation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    pub score: f32,
    pub layout: Layout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapSuggestion {
    pub index_a: usize,
    pub index_b: usize,
    pub key_a: String,
    pub key_b: String,
    pub score_delta: f32,
    pub improvement_pct: f32,
}
