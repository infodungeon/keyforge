pub mod corpus;
pub mod keyboard;
pub mod layout;
pub mod rubric;
pub mod serde_utils;

pub use corpus::Corpus;
pub use keyboard::{KeyNode, Keyboard};
pub use keyforge_protocol::keycodes::KeycodeRegistry;
pub use layout::Layout;
pub use rubric::Rubric;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
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

    // Visual Data
    #[serde(default)]
    pub heatmap: Vec<f32>,

    // Detailed Metrics
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
pub mod error;
pub mod loader;
