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

/// High-level ergonomic score summary.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScoreSummary {
    /// Total normalized ergonomic score.
    pub score: f32,
    /// Hand balance (-1.0 Left, +1.0 Right, 0.0 Balanced).
    pub hand_balance: f32,
}

/// Detailed breakdown of biomechanical metrics and violations.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetricBreakdown {
    /// Ratio of Same Finger Bigrams to total bigrams.
    pub sfb_ratio: f32,
    /// Count of scissors (normalized).
    pub scissors: f32,
    /// Count of redirects (normalized).
    pub redirects: f32,
    /// Count of rolls (normalized).
    pub rolls: f32,
    /// Standard metric values (SFB freq, roll freq, etc).
    pub metrics: MetricSet,
    /// Top offenders grouped by metric.
    pub violations: HashMap<MetricId, Vec<MetricViolation>>,
}

/// Movement-related statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TravelStatistics {
    /// Total finger travel distance (normalized).
    pub distance: f32,
    /// Average travel distance per keypress.
    pub travel_per_key: f32,
}

/// Spatial data for visualization.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Heatmaps {
    /// Per-key usage frequency map.
    pub usage: Vec<f32>,
    /// Per-key ergonomic penalty map.
    pub effort: Vec<f32>,
}

/// Detailed breakdown of a layout's performance.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnalysisReport {
    /// High-level summary.
    pub summary: ScoreSummary,
    /// Metric details and violations.
    pub breakdown: MetricBreakdown,
    /// Physical movement stats.
    pub travel: TravelStatistics,
    /// Heatmap visualizations.
    pub visualizations: Heatmaps,
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
