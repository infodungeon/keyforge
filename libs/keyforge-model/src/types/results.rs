// libs/keyforge-model/src/types/results.rs

use crate::layout::Layout;
use crate::metrics::{MetricId, MetricSet};
use crate::types::Score;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a specific N-gram that violates a metric threshold.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetricViolation {
    /// The keys involved (e.g., "TH").
    pub keys: String,
    /// The cost contribution.
    pub score: Score,
    /// The frequency.
    pub freq: Score,
}

/// Detailed breakdown of a layout's performance.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnalysisReport {
    /// Total weighted score.
    pub score: Score,
    /// Raw unnormalized score (for verification).
    #[serde(default)]
    pub raw_score: Score,
    /// Standard metric values.
    #[serde(default)]
    pub metrics: MetricSet,
    /// Top offenders grouped by metric.
    #[serde(default)]
    pub violations: HashMap<MetricId, Vec<MetricViolation>>,

    /// Total finger travel distance.
    pub distance: Score,
    /// Average travel distance per keypress.
    pub travel_per_key: Score,
    /// Total Same Finger Bigram cost.
    pub sfb_total: Score,
    /// Ratio of SFBs to total bigrams.
    pub sfb_ratio: Score,
    /// Hand balance (-1.0 Left, +1.0 Right, 0.0 Balanced).
    pub hand_balance: Score,
    /// Scissor score.
    pub scissors: Score,
    /// Redirect score.
    pub redirects: Score,
    /// Inward roll score.
    pub rolls: Score,
    /// Total SFB penalty.
    #[serde(default)]
    pub sfb_penalty: Score,
    /// Total scissor penalty.
    #[serde(default)]
    pub scissor_penalty: Score,
    /// Total redirect penalty.
    #[serde(default)]
    pub redir_penalty: Score,
    /// Total roll penalty.
    #[serde(default)]
    pub roll_penalty: Score,
    /// Usage heatmap.
    #[serde(default)]
    pub heatmap: Vec<Score>,
    /// Effort heatmap.
    #[serde(default)]
    pub penalty_map: Vec<Score>,
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
    /// The final score achieved.
    pub score: Score,
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
    pub score_delta: Score,
    /// Percentage improvement.
    pub improvement_pct: Score,
}
