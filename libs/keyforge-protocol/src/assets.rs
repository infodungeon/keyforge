// libs/keyforge-protocol/src/assets.rs

use crate::constants;
use crate::types::{
    ColIndexDto, FingerIndexDto, HandIndexDto, KeyCodeDto, LimitedVec, RowIndexDto,
};
use keyforge_model::Validator;
use serde::{Deserialize, Serialize};
#[cfg(feature = "ts_bindings")]
use ts_rs::TS;
use utoipa::ToSchema;

/// DTO for `KeyNode` geometry.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct KeyNodeDto {
    /// Physical index of the key.
    pub index: u16,
    /// Default label for the key.
    pub label: String,
    /// X coordinate in millimeters.
    pub x: f32,
    /// Y coordinate in millimeters.
    pub y: f32,
    /// Width of the key cap.
    pub w: f32,
    /// Height of the key cap.
    pub h: f32,
    /// Assigned hand.
    pub hand: HandIndexDto,
    /// Assigned finger.
    pub finger: FingerIndexDto,
    /// Assigned row.
    pub row: RowIndexDto,
    /// Assigned column.
    pub col: ColIndexDto,
    /// True if this is a home-row key.
    pub is_home: bool,
    /// True if this is a reach stretch key.
    pub is_stretch: bool,
    /// Rotation angle in degrees.
    pub r: f32,
    /// Rotation center X.
    pub rx: f32,
    /// Rotation center Y.
    pub ry: f32,
}

impl From<keyforge_model::KeyNode> for KeyNodeDto {
    fn from(val: keyforge_model::KeyNode) -> Self {
        Self {
            #[allow(clippy::cast_possible_truncation)]
            index: val.index as u16,
            label: val.label,
            x: val.x,
            y: val.y,
            w: val.w,
            h: val.h,
            hand: val.hand.into(),
            finger: val.finger.into(),
            row: val.row.into(),
            col: val.col.into(),
            is_home: val.is_home,
            is_stretch: val.is_stretch,
            r: val.r,
            rx: val.rx,
            ry: val.ry,
        }
    }
}

/// DTO for `KeyboardGeometry`.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema, Default)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct KeyboardGeometryDto {
    /// List of key nodes in the geometry.
    pub keys: Vec<KeyNodeDto>,
    /// Indices of keys considered high-value (prime).
    pub prime_slots: Vec<u16>,
    /// Indices of keys considered medium-value.
    pub med_slots: Vec<u16>,
    /// Indices of keys considered low-value.
    pub low_slots: Vec<u16>,
    /// Y-coordinate or index of the home row.
    pub home_row: i8,
}

/// DTO for `Layout`.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema, Default)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct LayoutDto {
    /// List of keycodes assigned to positions (matching keys in geometry).
    pub keys: Vec<KeyCodeDto>,
}

impl From<keyforge_model::Layout> for LayoutDto {
    fn from(val: keyforge_model::Layout) -> Self {
        Self {
            keys: val.keys.iter().map(|k| (*k).into()).collect(),
        }
    }
}

/// Unique identifier for a biomechanical metric (DTO).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
#[serde(rename_all = "snake_case")]
pub enum MetricIdDto {
    /// Total finger travel distance.
    TravelDistance,
    /// Same Finger Bigram frequency.
    Sfb,
    /// Weighted Same Finger Bigram penalty.
    SfbPenalty,
    /// Disjoint Same Finger Bigram frequency.
    Dsfb,
    /// Lateral Stretch Bigram frequency.
    Lsb,
    /// High Frequency Scissors.
    Hfs,
    /// Direct Scissor movement frequency.
    Scissor,
    /// Weighted Scissor penalty.
    ScissorPenalty,
    /// Redirect movement frequency.
    Redirect,
    /// Weighted Redirect penalty.
    RedirectPenalty,
    /// Inward roll frequency.
    RollIn,
    /// Outward roll frequency.
    RollOut,
    /// Total roll-related penalties.
    RollPenalty,
    /// Percent deviation from perfect hand balance.
    HandBalance,
    /// Per-finger usage distribution.
    FingerUsage,
    /// Per-row usage distribution.
    RowBalance,
}

impl From<keyforge_model::metrics::MetricId> for MetricIdDto {
    fn from(val: keyforge_model::metrics::MetricId) -> Self {
        match val {
            keyforge_model::metrics::MetricId::TravelDistance => Self::TravelDistance,
            keyforge_model::metrics::MetricId::Sfb => Self::Sfb,
            keyforge_model::metrics::MetricId::SfbPenalty => Self::SfbPenalty,
            keyforge_model::metrics::MetricId::Dsfb => Self::Dsfb,
            keyforge_model::metrics::MetricId::Lsb => Self::Lsb,
            keyforge_model::metrics::MetricId::Hfs => Self::Hfs,
            keyforge_model::metrics::MetricId::Scissor => Self::Scissor,
            keyforge_model::metrics::MetricId::ScissorPenalty => Self::ScissorPenalty,
            keyforge_model::metrics::MetricId::Redirect => Self::Redirect,
            keyforge_model::metrics::MetricId::RedirectPenalty => Self::RedirectPenalty,
            keyforge_model::metrics::MetricId::RollIn => Self::RollIn,
            keyforge_model::metrics::MetricId::RollOut => Self::RollOut,
            keyforge_model::metrics::MetricId::RollPenalty => Self::RollPenalty,
            keyforge_model::metrics::MetricId::HandBalance => Self::HandBalance,
            keyforge_model::metrics::MetricId::FingerUsage => Self::FingerUsage,
            keyforge_model::metrics::MetricId::RowBalance => Self::RowBalance,
        }
    }
}

/// DTO for a metric violation.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct MetricViolationDto {
    /// String representation of the keys involved (e.g. "QU").
    pub keys: String,
    /// Raw cost contribution.
    pub score: f32,
    /// Frequency of occurrence in the corpus.
    pub freq: f32,
}

impl From<&keyforge_model::types::MetricViolation> for MetricViolationDto {
    fn from(val: &keyforge_model::types::MetricViolation) -> Self {
        Self {
            keys: val.keys.clone(),
            score: val.score,
            freq: val.freq,
        }
    }
}

/// DTO for an analysis report.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct AnalysisReportDto {
    /// Final normalized ergonomic score.
    pub score: f32,
    /// List of computed metric values.
    pub metrics: Vec<(MetricIdDto, f32)>,
    /// List of top violations grouped by metric.
    pub violations: Vec<(MetricIdDto, Vec<MetricViolationDto>)>,
    /// Total travel distance.
    pub distance: f32,
    /// Travel per keypress.
    pub travel_per_key: f32,
    /// Total SFB cost.
    pub sfb_total: f32,
    /// SFB percentage of total bigrams.
    pub sfb_ratio: f32,
    /// Hand balance (-1.0 to 1.0).
    pub hand_balance: f32,
    /// Per-key usage frequency map.
    pub heatmap: Vec<f32>,
    /// Per-key ergonomic penalty map.
    pub penalty_map: Vec<f32>,
}

impl From<keyforge_model::AnalysisReport> for AnalysisReportDto {
    fn from(val: keyforge_model::AnalysisReport) -> Self {
        Self {
            score: val.score,
            metrics: val
                .metrics
                .values
                .iter()
                .map(|(id, v)| (MetricIdDto::from(*id), *v))
                .collect(),
            violations: val
                .violations
                .iter()
                .map(|(id, v)| (MetricIdDto::from(*id), v.iter().map(Into::into).collect()))
                .collect(),
            distance: val.distance,
            travel_per_key: val.travel_per_key,
            sfb_total: val.sfb_total,
            sfb_ratio: val.sfb_ratio,
            hand_balance: val.hand_balance,
            heatmap: val.heatmap,
            penalty_map: val.penalty_map,
        }
    }
}

/// DTO for a swap suggestion.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct SwapSuggestionDto {
    /// First key index.
    pub index_a: usize,
    /// Second key index.
    pub index_b: usize,
    /// Label of the first key.
    pub key_a: String,
    /// Label of the second key.
    pub key_b: String,
    /// Score change (negative is better).
    pub score_delta: f32,
    /// Percentage relative improvement.
    pub improvement_pct: f32,
}

impl From<keyforge_model::types::SwapSuggestion> for SwapSuggestionDto {
    fn from(val: keyforge_model::types::SwapSuggestion) -> Self {
        Self {
            index_a: val.index_a,
            index_b: val.index_b,
            key_a: val.key_a,
            key_b: val.key_b,
            score_delta: val.score_delta,
            improvement_pct: val.improvement_pct,
        }
    }
}

/// Manifest entry for a system asset.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct AssetManifestEntry {
    /// Unique asset ID.
    pub id: String,
    /// Content-addressable hash.
    pub hash: String,
    /// Size in bytes.
    pub size_bytes: u64,
    /// Last modification timestamp (Unix).
    pub updated_at: u64,
}

/// A biometric timing sample.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema, Default)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct BiometricSample {
    /// Keycode of the first key.
    pub key_a: u16,
    /// Keycode of the second key.
    pub key_b: u16,
    /// Duration in milliseconds.
    pub duration_ms: u32,
}

/// Collection of user typing statistics and biometric data.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema, Default)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct UserStatsStore {
    /// Number of distinct typing sessions recorded.
    pub sessions: u64,
    /// Cumulative keystroke count.
    pub total_keystrokes: u64,
    /// List of collected biometric samples.
    pub biometrics: LimitedVec<BiometricSample>,
}

impl Validator for UserStatsStore {
    fn validate(&self) -> Result<(), String> {
        if self.biometrics.len() > constants::MAX_BIOMETRIC_SAMPLES {
            return Err(format!(
                "Too many biometric samples: {} (max: {})",
                self.biometrics.len(),
                constants::MAX_BIOMETRIC_SAMPLES
            ));
        }
        Ok(())
    }
}

/// Response containing a population of layouts.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct PopulationResponse {
    /// List of layout strings in the current population.
    pub layouts: LimitedVec<String>,
}
