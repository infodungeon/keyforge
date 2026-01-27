// libs/keyforge-model/src/types/results.rs

use crate::layout::Layout;
use crate::metrics::{MetricId, MetricSet};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a specific N-gram that violates a metric threshold.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetricViolation {
    /// The keys involved (e.g., "TH").
    pub keys: String,
    /// The cost contribution.
    pub score: f32,
    /// The frequency.
    pub freq: f32,
}

/// Detailed breakdown of a layout's performance.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnalysisReport {
    /// Total weighted score.
    pub score: f32,
    /// Standard metric values.
    #[serde(default)]
    pub metrics: MetricSet,
    /// Top offenders grouped by metric.
    #[serde(default)]
    pub violations: HashMap<MetricId, Vec<MetricViolation>>,

    /// Total finger travel distance.
    pub distance: f32,
    /// Average travel distance per keypress.
    pub travel_per_key: f32,
    /// Total Same Finger Bigram cost.
    pub sfb_total: f32,
    /// Ratio of SFBs to total bigrams.
    pub sfb_ratio: f32,
    /// Hand balance (-1.0 Left, +1.0 Right, 0.0 Balanced).
    pub hand_balance: f32,
    /// Scissor score.
    pub scissors: f32,
    /// Redirect score.
    pub redirects: f32,
    /// Inward roll score.
    pub rolls: f32,
    /// Total SFB penalty.
    #[serde(default)]
    pub sfb_penalty: f32,
    /// Total scissor penalty.
    #[serde(default)]
    pub scissor_penalty: f32,
    /// Total redirect penalty.
    #[serde(default)]
    pub redir_penalty: f32,
    /// Total roll penalty.
    #[serde(default)]
    pub roll_penalty: f32,
    /// Usage heatmap.
    #[serde(default)]
    pub heatmap: Vec<f32>,
    /// Effort heatmap.
    #[serde(default)]
    pub penalty_map: Vec<f32>,
    /// Top SFB offenders.
    #[serde(default)]
    pub top_sfbs: Vec<MetricViolation>,
    /// Top Scissor offenders.
    #[serde(default)]
    pub top_scissors: Vec<MetricViolation>,
    /// Top Redirect offenders.
    #[serde(default)]
    pub top_redirs: Vec<MetricViolation>,
}

/// The final output of an optimization run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    /// The final score achieved (normalized f32).
    pub score: f32,
    /// The raw scaled score (fixed-point i64).
    #[serde(default)]
    pub raw_score: i64,
    /// The optimized layout.
    pub layout: Layout,
}

/// Result of a static scoring operation.
pub type ScoringResult = OptimizationResult;

/// A proposed change to the layout during optimization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapSuggestion {
    /// Index of the first key.
    pub index_a: usize,
    /// Index of the second key.
    pub index_b: usize,
    /// Label of the first key.
    pub key_a: String,
    /// Label of the second key.
    pub key_b: String,
    /// Change in score (negative is improvement).
    pub score_delta: f32,
    /// Percentage improvement.
    pub improvement_pct: f32,
}
