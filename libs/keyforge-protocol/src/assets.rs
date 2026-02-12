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
            index: val.index().raw(),
            label: val.label().to_string(),
            x: val.x().to_f32(),
            y: val.y().to_f32(),
            w: val.w(),
            h: val.h(),
            hand: val.hand().into(),
            finger: val.finger().into(),
            row: val.row().into(),
            col: val.col().into(),
            is_home: val.is_home(),
            is_stretch: val.is_stretch(),
            r: val.r(),
            rx: val.rx().to_f32(),
            ry: val.ry().to_f32(),
        }
    }
}

impl From<KeyNodeDto> for keyforge_model::KeyNode {
    fn from(val: KeyNodeDto) -> Self {
        keyforge_model::KeyNode::builder()
            .index(val.index.into())
            .label(val.label)
            .x(keyforge_model::types::SpatialUnit::from_f32(val.x))
            .y(keyforge_model::types::SpatialUnit::from_f32(val.y))
            .w(val.w)
            .h(val.h)
            .hand(val.hand.into())
            .finger(val.finger.into())
            .row(val.row.into())
            .col(val.col.into())
            .is_home(val.is_home)
            .is_stretch(val.is_stretch)
            .r(val.r)
            .rx(keyforge_model::types::SpatialUnit::from_f32(val.rx))
            .ry(keyforge_model::types::SpatialUnit::from_f32(val.ry))
            .build()
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
            keys: val.keys().iter().map(|k| (*k).into()).collect(),
        }
    }
}

impl From<LayoutDto> for keyforge_model::Layout {
    fn from(val: LayoutDto) -> Self {
        Self::new_unchecked(val.keys.iter().map(|k| (*k).into()).collect())
    }
}

/// Unique identifier for a biomechanical metric (DTO).
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    ToSchema,
    strum::Display,
    strum::EnumString,
)]
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
    /// Home Finger Substitution frequency.
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

impl From<MetricIdDto> for keyforge_model::metrics::MetricId {
    fn from(val: MetricIdDto) -> Self {
        match val {
            MetricIdDto::TravelDistance => Self::TravelDistance,
            MetricIdDto::Sfb => Self::Sfb,
            MetricIdDto::SfbPenalty => Self::SfbPenalty,
            MetricIdDto::Dsfb => Self::Dsfb,
            MetricIdDto::Lsb => Self::Lsb,
            MetricIdDto::Hfs => Self::Hfs,
            MetricIdDto::Scissor => Self::Scissor,
            MetricIdDto::ScissorPenalty => Self::ScissorPenalty,
            MetricIdDto::Redirect => Self::Redirect,
            MetricIdDto::RedirectPenalty => Self::RedirectPenalty,
            MetricIdDto::RollIn => Self::RollIn,
            MetricIdDto::RollOut => Self::RollOut,
            MetricIdDto::RollPenalty => Self::RollPenalty,
            MetricIdDto::HandBalance => Self::HandBalance,
            MetricIdDto::FingerUsage => Self::FingerUsage,
            MetricIdDto::RowBalance => Self::RowBalance,
        }
    }
}

use std::sync::Arc;

/// DTO for a collection of metric values.
#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct MetricSetDto {
    /// Map of metric ID to its calculated value.
    pub values: std::collections::HashMap<MetricIdDto, crate::types::ScoreDto>,
}

impl From<keyforge_model::metrics::MetricSet> for MetricSetDto {
    fn from(val: keyforge_model::metrics::MetricSet) -> Self {
        Self {
            values: val
                .iter()
                .map(|(id, score)| (MetricIdDto::from(*id), (*score).into()))
                .collect(),
        }
    }
}

impl From<MetricSetDto> for keyforge_model::metrics::MetricSet {
    fn from(val: MetricSetDto) -> Self {
        let mut set = Self::new();
        for (id, score) in val.values {
            set.set(id.into(), score.into());
        }
        set
    }
}

/// DTO for a metric violation.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct MetricViolationDto {
    /// String representation of the keys involved (e.g. "QU").
    pub keys: String,
    /// Raw cost contribution.
    pub score: crate::types::ScoreDto,
    /// Frequency of occurrence in the corpus.
    pub freq: crate::types::ScoreDto,
}

impl From<keyforge_model::types::MetricViolation> for MetricViolationDto {
    fn from(val: keyforge_model::types::MetricViolation) -> Self {
        Self {
            keys: val.keys,
            score: val.score.into(),
            freq: val.freq.into(),
        }
    }
}

impl From<&keyforge_model::types::MetricViolation> for MetricViolationDto {
    fn from(val: &keyforge_model::types::MetricViolation) -> Self {
        Self {
            keys: val.keys.clone(),
            score: val.score.into(),
            freq: val.freq.into(),
        }
    }
}

impl From<MetricViolationDto> for keyforge_model::types::MetricViolation {
    fn from(val: MetricViolationDto) -> Self {
        Self {
            keys: val.keys,
            score: val.score.into(),
            freq: val.freq.into(),
        }
    }
}

/// DTO for an analysis report.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct AnalysisReportDto {
    /// Final normalized ergonomic score.
    pub score: crate::types::ScoreDto,
    /// Raw unnormalized score (for verification).
    pub raw_score: crate::types::ScoreDto,
    /// Standard metric values.
    pub metrics: MetricSetDto,
    /// Top offenders grouped by metric.
    pub violations: std::collections::HashMap<MetricIdDto, Vec<MetricViolationDto>>,
    /// Total travel distance.
    pub distance: crate::types::ScoreDto,
    /// Travel per keypress.
    pub travel_per_key: crate::types::ScoreDto,
    /// Total SFB cost.
    pub sfb_total: crate::types::ScoreDto,
    /// SFB percentage of total bigrams.
    pub sfb_ratio: crate::types::ScoreDto,
    /// Hand balance (-1.0 to 1.0).
    pub hand_balance: crate::types::ScoreDto,
    /// Scissor score.
    pub scissors: crate::types::ScoreDto,
    /// Redirect score.
    pub redirects: crate::types::ScoreDto,
    /// Inward roll score.
    pub rolls: crate::types::ScoreDto,
    /// Total SFB penalty.
    pub sfb_penalty: crate::types::ScoreDto,
    /// Total scissor penalty.
    pub scissor_penalty: crate::types::ScoreDto,
    /// Total redirect penalty.
    pub redir_penalty: crate::types::ScoreDto,
    /// Total roll penalty.
    pub roll_penalty: crate::types::ScoreDto,
    /// Per-key usage frequency map.
    pub heatmap: Vec<crate::types::ScoreDto>,
    /// Per-key ergonomic penalty map.
    pub penalty_map: Vec<crate::types::ScoreDto>,
    /// Top SFB offenders.
    pub top_sfbs: Vec<MetricViolationDto>,
    /// Top Scissor offenders.
    pub top_scissors: Vec<MetricViolationDto>,
    /// Top Redirect offenders.
    pub top_redirs: Vec<MetricViolationDto>,
}

impl From<keyforge_model::AnalysisReport> for AnalysisReportDto {
    fn from(val: keyforge_model::AnalysisReport) -> Self {
        Self {
            score: val.score.into(),
            raw_score: val.raw_score.into(),
            metrics: val.metrics.into(),
            violations: val
                .violations
                .into_iter()
                .map(|(k, v)| (k.into(), v.into_iter().map(Into::into).collect()))
                .collect(),
            distance: val.distance.into(),
            travel_per_key: val.travel_per_key.into(),
            sfb_total: val.sfb_total.into(),
            sfb_ratio: val.sfb_ratio.into(),
            hand_balance: val.hand_balance.into(),
            scissors: val.scissors.into(),
            redirects: val.redirects.into(),
            rolls: val.rolls.into(),
            sfb_penalty: val.sfb_penalty.into(),
            scissor_penalty: val.scissor_penalty.into(),
            redir_penalty: val.redir_penalty.into(),
            roll_penalty: val.roll_penalty.into(),
            heatmap: val.heatmap.into_iter().map(Into::into).collect(),
            penalty_map: val.penalty_map.into_iter().map(Into::into).collect(),
            top_sfbs: val.top_sfbs.into_iter().map(Into::into).collect(),
            top_scissors: val.top_scissors.into_iter().map(Into::into).collect(),
            top_redirs: val.top_redirs.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<AnalysisReportDto> for keyforge_model::AnalysisReport {
    fn from(val: AnalysisReportDto) -> Self {
        Self {
            score: val.score.into(),
            raw_score: val.raw_score.into(),
            metrics: val.metrics.into(),
            violations: val
                .violations
                .into_iter()
                .map(|(k, v)| (k.into(), v.into_iter().map(Into::into).collect()))
                .collect(),
            distance: val.distance.into(),
            travel_per_key: val.travel_per_key.into(),
            sfb_total: val.sfb_total.into(),
            sfb_ratio: val.sfb_ratio.into(),
            hand_balance: val.hand_balance.into(),
            scissors: val.scissors.into(),
            redirects: val.redirects.into(),
            rolls: val.rolls.into(),
            sfb_penalty: val.sfb_penalty.into(),
            scissor_penalty: val.scissor_penalty.into(),
            redir_penalty: val.redir_penalty.into(),
            roll_penalty: val.roll_penalty.into(),
            heatmap: val.heatmap.into_iter().map(Into::into).collect(),
            penalty_map: val.penalty_map.into_iter().map(Into::into).collect(),
            top_sfbs: val.top_sfbs.into_iter().map(Into::into).collect(),
            top_scissors: val.top_scissors.into_iter().map(Into::into).collect(),
            top_redirs: val.top_redirs.into_iter().map(Into::into).collect(),
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
    pub score_delta: crate::types::ScoreDto,
    /// Percentage relative improvement.
    pub improvement_pct: crate::types::ScoreDto,
}

impl From<keyforge_model::types::SwapSuggestion> for SwapSuggestionDto {
    fn from(val: keyforge_model::types::SwapSuggestion) -> Self {
        Self {
            index_a: val.index_a,
            index_b: val.index_b,
            key_a: val.key_a,
            key_b: val.key_b,
            score_delta: val.score_delta.into(),
            improvement_pct: val.improvement_pct.into(),
        }
    }
}

impl From<SwapSuggestionDto> for keyforge_model::types::SwapSuggestion {
    fn from(val: SwapSuggestionDto) -> Self {
        Self {
            index_a: val.index_a,
            index_b: val.index_b,
            key_a: val.key_a,
            key_b: val.key_b,
            score_delta: val.score_delta.into(),
            improvement_pct: val.improvement_pct.into(),
        }
    }
}

/// DTO for an optimization result.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct OptimizationResultDto {
    /// Final score achieved.
    pub score: crate::types::ScoreDto,
    /// Optimized layout.
    pub layout: LayoutDto,
}

impl From<keyforge_model::types::OptimizationResult> for OptimizationResultDto {
    fn from(val: keyforge_model::types::OptimizationResult) -> Self {
        Self {
            score: val.score.into(),
            layout: val.layout.into(),
        }
    }
}

impl From<OptimizationResultDto> for keyforge_model::types::OptimizationResult {
    fn from(val: OptimizationResultDto) -> Self {
        Self {
            score: val.score.into(),
            layout: val.layout.into(),
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

/// DTO for `CostModelMeta`.
#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct CostModelMetaDto {
    /// Schema version of the cost model.
    pub version: String,
    /// Human-readable description of the model.
    pub description: String,
    /// Measurement unit for costs (e.g., 'pts').
    pub unit: String,
}

impl From<keyforge_model::cost_model::CostModelMeta> for CostModelMetaDto {
    fn from(val: keyforge_model::cost_model::CostModelMeta) -> Self {
        Self {
            version: val.version,
            description: val.description,
            unit: val.unit,
        }
    }
}

impl From<CostModelMetaDto> for keyforge_model::cost_model::CostModelMeta {
    fn from(val: CostModelMetaDto) -> Self {
        Self {
            version: val.version,
            description: val.description,
            unit: val.unit,
        }
    }
}

/// DTO for `FingerReach`.
#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct FingerReachDto {
    /// Costs for basic row movements.
    pub base: std::collections::HashMap<RowIndexDto, f32>,
    /// Costs for inner column stretches.
    pub inner: std::collections::HashMap<RowIndexDto, f32>,
    /// Costs for outer column stretches.
    pub outer: std::collections::HashMap<RowIndexDto, f32>,
}

impl From<keyforge_model::cost_model::FingerReach> for FingerReachDto {
    fn from(val: keyforge_model::cost_model::FingerReach) -> Self {
        Self {
            base: val
                .base
                .into_iter()
                .map(|(k, v)| (k.into(), v.to_f32()))
                .collect(),
            inner: val
                .inner
                .into_iter()
                .map(|(k, v)| (k.into(), v.to_f32()))
                .collect(),
            outer: val
                .outer
                .into_iter()
                .map(|(k, v)| (k.into(), v.to_f32()))
                .collect(),
        }
    }
}

impl From<FingerReachDto> for keyforge_model::cost_model::FingerReach {
    fn from(val: FingerReachDto) -> Self {
        let sc = |v: f32| keyforge_model::types::Score::from_f32(v).unwrap_or_default();
        Self {
            base: val
                .base
                .into_iter()
                .map(|(k, v)| (k.into(), sc(v)))
                .collect(),
            inner: val
                .inner
                .into_iter()
                .map(|(k, v)| (k.into(), sc(v)))
                .collect(),
            outer: val
                .outer
                .into_iter()
                .map(|(k, v)| (k.into(), sc(v)))
                .collect(),
        }
    }
}

/// DTO for `FingerDefinition`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
#[serde(untagged)]
pub enum FingerDefinitionDto {
    /// Reach-based costs for standard fingers.
    Standard(FingerReachDto),
    /// Coordinate-based costs for thumb keys.
    Thumb(std::collections::HashMap<String, f32>),
    /// Unknown or complex finger definitions.
    #[cfg_attr(feature = "ts_bindings", ts(type = "any"))]
    Fallback(serde_json::Value),
}

impl From<keyforge_model::cost_model::FingerDefinition> for FingerDefinitionDto {
    fn from(val: keyforge_model::cost_model::FingerDefinition) -> Self {
        match val {
            keyforge_model::cost_model::FingerDefinition::Standard(reach) => {
                Self::Standard(reach.into())
            }
            keyforge_model::cost_model::FingerDefinition::Thumb(map) => {
                Self::Thumb(map.into_iter().map(|(k, v)| (k, v.to_f32())).collect())
            }
            keyforge_model::cost_model::FingerDefinition::Fallback => {
                Self::Fallback(serde_json::Value::Null)
            }
        }
    }
}

impl From<FingerDefinitionDto> for keyforge_model::cost_model::FingerDefinition {
    fn from(val: FingerDefinitionDto) -> Self {
        let sc = |v: f32| keyforge_model::types::Score::from_f32(v).unwrap_or_default();
        match val {
            FingerDefinitionDto::Standard(reach) => Self::Standard(reach.into()),
            FingerDefinitionDto::Thumb(map) => {
                Self::Thumb(map.into_iter().map(|(k, v)| (k, sc(v))).collect())
            }
            FingerDefinitionDto::Fallback(_) => Self::Fallback,
        }
    }
}

/// DTO for `HandDefinition`.
#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct HandDefinitionDto {
    /// Mapping of finger IDs to their definitions.
    #[serde(flatten)]
    pub fingers: std::collections::HashMap<String, FingerDefinitionDto>,
}

impl From<keyforge_model::cost_model::HandDefinition> for HandDefinitionDto {
    fn from(val: keyforge_model::cost_model::HandDefinition) -> Self {
        Self {
            fingers: val
                .fingers
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
        }
    }
}

impl From<HandDefinitionDto> for keyforge_model::cost_model::HandDefinition {
    fn from(val: HandDefinitionDto) -> Self {
        Self {
            fingers: val
                .fingers
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
        }
    }
}

/// DTO for `ModelDefinition`.
#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct ModelDefinitionDto {
    /// Description of the physical layout model.
    pub description: String,
    /// Static costs mapped by hand ID.
    pub static_costs: std::collections::HashMap<String, HandDefinitionDto>,
}

impl From<keyforge_model::cost_model::ModelDefinition> for ModelDefinitionDto {
    fn from(val: keyforge_model::cost_model::ModelDefinition) -> Self {
        Self {
            description: val.description,
            static_costs: val
                .static_costs
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
        }
    }
}

impl From<ModelDefinitionDto> for keyforge_model::cost_model::ModelDefinition {
    fn from(val: ModelDefinitionDto) -> Self {
        Self {
            description: val.description,
            static_costs: val
                .static_costs
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
        }
    }
}

/// DTO for `DynamicRules`.
#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct DynamicRulesDto {
    /// Modifiers for specific key sequences.
    pub sequence_modifiers: std::collections::HashMap<String, f32>,
    /// Penalties for general ergonomic violations.
    pub penalties: std::collections::HashMap<String, f32>,
    /// Hard constraints on layout properties.
    pub constraints: std::collections::HashMap<String, f32>,
}

impl From<keyforge_model::cost_model::DynamicRules> for DynamicRulesDto {
    fn from(val: keyforge_model::cost_model::DynamicRules) -> Self {
        Self {
            sequence_modifiers: val
                .sequence_modifiers
                .into_iter()
                .map(|(k, v)| (k, v.to_f32()))
                .collect(),
            penalties: val
                .penalties
                .into_iter()
                .map(|(k, v)| (k, v.to_f32()))
                .collect(),
            constraints: val
                .constraints
                .into_iter()
                .map(|(k, v)| (k, v.to_f32()))
                .collect(),
        }
    }
}

impl From<DynamicRulesDto> for keyforge_model::cost_model::DynamicRules {
    fn from(val: DynamicRulesDto) -> Self {
        let sc = |v: f32| keyforge_model::types::Score::from_f32(v).unwrap_or_default();
        Self {
            sequence_modifiers: val
                .sequence_modifiers
                .into_iter()
                .map(|(k, v)| (k, sc(v)))
                .collect(),
            penalties: val.penalties.into_iter().map(|(k, v)| (k, sc(v))).collect(),
            constraints: val
                .constraints
                .into_iter()
                .map(|(k, v)| (k, sc(v)))
                .collect(),
        }
    }
}

/// DTO for `CostModel` (Raw representation).
#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct CostModelDto {
    /// Model metadata.
    pub meta: CostModelMetaDto,
    /// Definitions for different physical layouts.
    pub models: std::collections::HashMap<String, ModelDefinitionDto>,
    /// Global dynamic rules and penalties.
    pub dynamic_rules: DynamicRulesDto,
}

impl From<keyforge_model::cost_model::CostModel> for CostModelDto {
    fn from(val: keyforge_model::cost_model::CostModel) -> Self {
        Self {
            meta: val.meta.into(),
            models: val.models.into_iter().map(|(k, v)| (k, v.into())).collect(),
            dynamic_rules: val.dynamic_rules.into(),
        }
    }
}

impl From<CostModelDto> for keyforge_model::cost_model::CostModel {
    fn from(val: CostModelDto) -> Self {
        Self {
            meta: val.meta.into(),
            models: val.models.into_iter().map(|(k, v)| (k, v.into())).collect(),
            dynamic_rules: val.dynamic_rules.into(),
        }
    }
}

impl keyforge_model::Asset for CostModelDto {
    fn category() -> keyforge_model::AssetCategory {
        keyforge_model::AssetCategory::CostModel
    }

    fn post_load(&mut self) -> Result<(), keyforge_model::error::ForgeError> {
        // We can perform validation on the DTO or convert to domain and validate there
        let domain: keyforge_model::cost_model::CostModel = self.clone().into();
        domain
            .validate()
            .map_err(keyforge_model::error::ForgeError::InvalidData)
    }
}

/// Result of a layout validation and analysis operation.
#[derive(Serialize, Clone, Debug, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct ValidationResultDto {
    /// Name of the layout that was validated.
    pub layout_name: String,
    /// Comprehensive analysis report including ergonomics metrics.
    pub score: AnalysisReportDto,
    /// Physical geometry of the keyboard used for analysis.
    pub geometry: Vec<KeyNodeDto>,
    /// Frequency heatmap indicating which keys are used most relative to their position.
    pub heatmap: Vec<f32>,
    /// Map of ergonomic penalties applied across the layout.
    pub penalty_map: Vec<f32>,
}

/// DTO for `CorpusMetadata`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Default)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct CorpusMetadataDto {
    /// Whether this is a standard corpus.
    pub is_std: bool,
}

/// DTO for `Corpus`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct CorpusDto {
    /// Metadata.
    pub meta: CorpusMetadataDto,
    /// Character frequencies.
    pub char_freqs: Vec<u64>,
    /// Bigram frequencies.
    pub bigrams: Vec<(u16, u16, u32)>,
    /// Trigram frequencies.
    pub trigrams: Vec<(u16, u16, u16, u32)>,
    /// Word frequencies.
    pub words: Vec<(String, u32)>,
}

impl From<keyforge_model::Corpus> for CorpusDto {
    fn from(val: keyforge_model::Corpus) -> Self {
        Self {
            meta: CorpusMetadataDto {
                is_std: val.meta.is_std,
            },
            char_freqs: val.char_freqs.to_vec(),
            bigrams: val.bigrams.to_vec(),
            trigrams: val.trigrams.to_vec(),
            words: val.words.to_vec(),
        }
    }
}

impl From<CorpusDto> for keyforge_model::Corpus {
    fn from(val: CorpusDto) -> Self {
        Self {
            meta: keyforge_model::corpus::CorpusMetadata {
                is_std: val.meta.is_std,
            },
            char_freqs: Arc::from(val.char_freqs),
            bigrams: Arc::from(val.bigrams),
            trigrams: Arc::from(val.trigrams),
            words: Arc::from(val.words),
        }
    }
}

impl keyforge_model::Asset for CorpusDto {
    fn category() -> keyforge_model::AssetCategory {
        keyforge_model::AssetCategory::Corpus
    }
}

/// Statistics derived from raw analysis data for UI display.
#[derive(Serialize, Clone, Debug, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct DerivedStatsDto {
    /// Calculated balance between left and right hand usage (-1.0 to 1.0).
    pub hand_balance: f32,
}

/// DTO for a single keycode definition.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct KeycodeDefinitionDto {
    /// The numeric code.
    pub code: crate::types::KeyCodeDto,
    /// The canonical ID (e.g., "`KC_A`").
    pub id: String,
    /// The display label (e.g., "A").
    pub label: String,
    /// Alternative names.
    pub aliases: Vec<String>,
}

impl From<keyforge_model::keycodes::KeycodeDefinition> for KeycodeDefinitionDto {
    fn from(val: keyforge_model::keycodes::KeycodeDefinition) -> Self {
        Self {
            code: val.code.into(),
            id: val.id,
            label: val.label,
            aliases: val.aliases,
        }
    }
}

impl From<KeycodeDefinitionDto> for keyforge_model::keycodes::KeycodeDefinition {
    fn from(val: KeycodeDefinitionDto) -> Self {
        Self {
            code: val.code.into(),
            id: val.id,
            label: val.label,
            aliases: val.aliases,
        }
    }
}

/// DTO for `KeycodeRegistry`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Default)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct KeycodeRegistryDto {
    /// List of all definitions.
    pub definitions: Vec<KeycodeDefinitionDto>,
}

impl From<keyforge_model::keycodes::KeycodeRegistry> for KeycodeRegistryDto {
    fn from(val: keyforge_model::keycodes::KeycodeRegistry) -> Self {
        Self {
            definitions: val.definitions.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<KeycodeRegistryDto> for keyforge_model::keycodes::KeycodeRegistry {
    fn from(val: KeycodeRegistryDto) -> Self {
        Self::new(val.definitions.into_iter().map(Into::into).collect())
    }
}

impl keyforge_model::Asset for KeycodeRegistryDto {
    fn category() -> keyforge_model::AssetCategory {
        keyforge_model::AssetCategory::Keycodes
    }

    fn post_load(&mut self) -> Result<(), keyforge_model::error::ForgeError> {
        let mut reg: keyforge_model::keycodes::KeycodeRegistry = self.clone().into();
        reg.rebuild_maps();
        reg.validate()
            .map_err(keyforge_model::error::ForgeError::InvalidData)
    }
}

/// DTO for `Rubric`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct RubricDto {
    /// Relative effort for each of the 5 fingers.
    pub finger_effort: [crate::types::ScoreDto; 5],
    /// Penalty for lateral movement.
    pub travel_lat: crate::types::ScoreDto,
    /// Penalty for vertical movement.
    pub travel_vert: crate::types::ScoreDto,
    /// Base penalty for Same Finger Bigrams.
    pub sfb_base: crate::types::ScoreDto,
    /// Penalty for lateral SFBs.
    pub sfb_lateral: crate::types::ScoreDto,
    /// Penalty for lateral SFBs on weak fingers.
    pub sfb_lateral_weak: crate::types::ScoreDto,
    /// Penalty for diagonal SFBs.
    pub sfb_diagonal: crate::types::ScoreDto,
    /// Penalty for long-distance SFBs.
    pub sfb_long: crate::types::ScoreDto,
    /// Minimum row difference for a bigram to be considered long-distance.
    pub threshold_sfb_long_row_diff: i8,
    /// Penalty for scissor movements.
    pub penalty_scissor: crate::types::ScoreDto,
    /// Minimum row difference for a movement to be considered a scissor.
    pub threshold_scissor_row_diff: i8,
    /// Penalty for redirects.
    pub redirect: crate::types::ScoreDto,
    /// Bonus for comfortable rolls.
    pub roll_bonus: crate::types::ScoreDto,
    /// Bonus for outward rolls.
    pub roll_out_bonus: crate::types::ScoreDto,
    /// Target percentage of trigrams to cover in optimized results.
    pub trigram_coverage: crate::types::ScoreDto,
    /// Maximum number of trigrams to track in analysis.
    pub trigram_limit: usize,
}

impl From<keyforge_model::Rubric> for RubricDto {
    fn from(val: keyforge_model::Rubric) -> Self {
        let raw: keyforge_model::rubric::RawRubric = val.into();
        Self {
            finger_effort: raw.finger_effort.map(Into::into),
            travel_lat: raw.travel_lat.into(),
            travel_vert: raw.travel_vert.into(),
            sfb_base: raw.sfb_base.into(),
            sfb_lateral: raw.sfb_lateral.into(),
            sfb_lateral_weak: raw.sfb_lateral_weak.into(),
            sfb_diagonal: raw.sfb_diagonal.into(),
            sfb_long: raw.sfb_long.into(),
            threshold_sfb_long_row_diff: raw.threshold_sfb_long_row_diff,
            penalty_scissor: raw.penalty_scissor.into(),
            threshold_scissor_row_diff: raw.threshold_scissor_row_diff,
            redirect: raw.redirect.into(),
            roll_bonus: raw.roll_bonus.into(),
            roll_out_bonus: raw.roll_out_bonus.into(),
            trigram_coverage: raw.trigram_coverage.into(),
            trigram_limit: raw.trigram_limit,
        }
    }
}

impl From<RubricDto> for keyforge_model::Rubric {
    fn from(val: RubricDto) -> Self {
        let raw = keyforge_model::rubric::RawRubric {
            finger_effort: val.finger_effort.map(Into::into),
            travel_lat: val.travel_lat.into(),
            travel_vert: val.travel_vert.into(),
            sfb_base: val.sfb_base.into(),
            sfb_lateral: val.sfb_lateral.into(),
            sfb_lateral_weak: val.sfb_lateral_weak.into(),
            sfb_diagonal: val.sfb_diagonal.into(),
            sfb_long: val.sfb_long.into(),
            threshold_sfb_long_row_diff: val.threshold_sfb_long_row_diff,
            penalty_scissor: val.penalty_scissor.into(),
            threshold_scissor_row_diff: val.threshold_scissor_row_diff,
            redirect: val.redirect.into(),
            roll_bonus: val.roll_bonus.into(),
            roll_out_bonus: val.roll_out_bonus.into(),
            trigram_coverage: val.trigram_coverage.into(),
            trigram_limit: val.trigram_limit,
        };
        raw.into()
    }
}

impl keyforge_model::Asset for RubricDto {
    fn category() -> keyforge_model::AssetCategory {
        keyforge_model::AssetCategory::Rubric
    }

    fn post_load(&mut self) -> Result<(), keyforge_model::error::ForgeError> {
        Ok(())
    }
}
