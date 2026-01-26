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
    pub index: u16,
    pub label: String,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub hand: HandIndexDto,
    pub finger: FingerIndexDto,
    pub row: RowIndexDto,
    pub col: ColIndexDto,
    pub is_home: bool,
    pub is_stretch: bool,
    pub r: f32,
    pub rx: f32,
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
    pub keys: Vec<KeyNodeDto>,
    pub prime_slots: Vec<u16>,
    pub med_slots: Vec<u16>,
    pub low_slots: Vec<u16>,
    pub home_row: i8,
}

/// DTO for `Layout`.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema, Default)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct LayoutDto {
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
    TravelDistance,
    Sfb,
    SfbPenalty,
    Dsfb,
    Lsb,
    Hfs,
    Scissor,
    ScissorPenalty,
    Redirect,
    RedirectPenalty,
    RollIn,
    RollOut,
    RollPenalty,
    HandBalance,
    FingerUsage,
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
    pub keys: String,
    pub score: f32,
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
    pub score: f32,
    pub metrics: Vec<(MetricIdDto, f32)>,
    pub violations: Vec<(MetricIdDto, Vec<MetricViolationDto>)>,
    pub distance: f32,
    pub travel_per_key: f32,
    pub sfb_total: f32,
    pub sfb_ratio: f32,
    pub hand_balance: f32,
    pub heatmap: Vec<f32>,
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
    pub index_a: usize,
    pub index_b: usize,
    pub key_a: String,
    pub key_b: String,
    pub score_delta: f32,
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
    pub id: String,
    pub hash: String,
    pub size_bytes: u64,
    pub updated_at: u64,
}

/// A biometric timing sample.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema, Default)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct BiometricSample {
    pub key_a: u16,
    pub key_b: u16,
    pub duration_ms: u32,
}

/// Collection of user typing statistics and biometric data.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema, Default)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct UserStatsStore {
    pub sessions: u64,
    pub total_keystrokes: u64,
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
    pub layouts: LimitedVec<String>,
}
