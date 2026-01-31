// libs/keyforge-protocol/src/types.rs

use keyforge_model::types as model;
use serde::{Deserialize, Serialize};
#[cfg(feature = "ts_bindings")]
use ts_rs::TS;
use utoipa::ToSchema;

/// DTO for `KeyIndex`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, type = "number"))]
pub struct KeyIndexDto(pub u16);

impl From<model::KeyIndex> for KeyIndexDto {
    fn from(val: model::KeyIndex) -> Self {
        Self(val.raw())
    }
}
impl From<KeyIndexDto> for model::KeyIndex {
    fn from(val: KeyIndexDto) -> Self {
        Self::new(val.0)
    }
}

/// DTO for `KeyCode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, type = "number"))]
pub struct KeyCodeDto(pub u16);

impl From<model::KeyCode> for KeyCodeDto {
    fn from(val: model::KeyCode) -> Self {
        Self(val.raw())
    }
}
impl From<KeyCodeDto> for model::KeyCode {
    fn from(val: KeyCodeDto) -> Self {
        Self::new(val.0)
    }
}

/// DTO for `HandIndex`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, type = "number"))]
pub struct HandIndexDto(pub u8);

impl From<model::HandIndex> for HandIndexDto {
    fn from(val: model::HandIndex) -> Self {
        Self(val.raw())
    }
}
impl From<HandIndexDto> for model::HandIndex {
    fn from(val: HandIndexDto) -> Self {
        Self::new(val.0)
    }
}

/// DTO for `FingerIndex`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, type = "number"))]
pub struct FingerIndexDto(pub u8);

impl From<model::FingerIndex> for FingerIndexDto {
    fn from(val: model::FingerIndex) -> Self {
        Self(val.raw())
    }
}
impl From<FingerIndexDto> for model::FingerIndex {
    fn from(val: FingerIndexDto) -> Self {
        Self::new(val.0)
    }
}

/// DTO for `RowIndex`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, type = "number"))]
pub struct RowIndexDto(pub i8);

impl From<model::RowIndex> for RowIndexDto {
    fn from(val: model::RowIndex) -> Self {
        Self(val.raw())
    }
}
impl From<RowIndexDto> for model::RowIndex {
    fn from(val: RowIndexDto) -> Self {
        Self::new(val.0)
    }
}

/// DTO for `ColIndex`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, type = "number"))]
pub struct ColIndexDto(pub i8);

impl From<model::ColIndex> for ColIndexDto {
    fn from(val: model::ColIndex) -> Self {
        Self(val.raw())
    }
}
impl From<ColIndexDto> for model::ColIndex {
    fn from(val: ColIndexDto) -> Self {
        Self::new(val.0)
    }
}

/// DTO for Score.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, type = "bigint"))]
pub struct ScoreDto(pub i64);

impl From<model::Score> for ScoreDto {
    fn from(val: model::Score) -> Self {
        Self(val.raw())
    }
}
impl From<ScoreDto> for model::Score {
    fn from(val: ScoreDto) -> Self {
        model::Score::from_scaled_i64(val.0)
    }
}

/// DTO for Weight.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct WeightDto(pub f32);

impl From<model::Weight> for WeightDto {
    fn from(val: model::Weight) -> Self {
        Self(val.to_f32())
    }
}
impl From<WeightDto> for model::Weight {
    fn from(val: WeightDto) -> Self {
        Self(val.0)
    }
}

/// Hand preference for space keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
#[serde(rename_all = "lowercase")]
pub enum SpaceHandPreferenceDto {
    /// Left hand only.
    Left,
    /// Right hand only.
    Right,
    /// Both hands.
    Bilateral,
}

impl From<model::SpaceHandPreference> for SpaceHandPreferenceDto {
    fn from(val: model::SpaceHandPreference) -> Self {
        match val {
            model::SpaceHandPreference::Left => Self::Left,
            model::SpaceHandPreference::Right => Self::Right,
            model::SpaceHandPreference::Bilateral => Self::Bilateral,
        }
    }
}
impl From<SpaceHandPreferenceDto> for model::SpaceHandPreference {
    fn from(val: SpaceHandPreferenceDto) -> Self {
        match val {
            SpaceHandPreferenceDto::Left => Self::Left,
            SpaceHandPreferenceDto::Right => Self::Right,
            SpaceHandPreferenceDto::Bilateral => Self::Bilateral,
        }
    }
}

/// DTO for `JobIdentifier`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct JobIdentifierDto {
    /// Unique content-based hash of the job configuration.
    pub hash: String,
}

impl From<keyforge_model::job::JobIdentifier> for JobIdentifierDto {
    fn from(val: keyforge_model::job::JobIdentifier) -> Self {
        Self { hash: val.hash }
    }
}

/// DTO for `JobStatus`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum JobStatusDto {
    /// Job is in the queue, awaiting a worker.
    Pending,
    /// Job is being processed by one or more workers.
    Running {
        /// Number of nodes currently working on this job.
        active_nodes: usize,
        /// Current best score found across all workers.
        current_best: Option<ScoreDto>,
    },
    /// Job has completed processing.
    Completed {
        /// Final best score achieved.
        final_score: ScoreDto,
        /// The optimized layout string.
        #[cfg_attr(feature = "ts_bindings", ts(type = "{ keys: number[] }"))]
        final_layout: crate::assets::LayoutDto,
        /// Total cumulative compute time in seconds.
        total_compute_sec: u64,
    },
}

impl From<keyforge_model::job::JobStatus> for JobStatusDto {
    fn from(val: keyforge_model::job::JobStatus) -> Self {
        match val {
            keyforge_model::job::JobStatus::Pending(_) => Self::Pending,
            keyforge_model::job::JobStatus::Running(r) => Self::Running {
                active_nodes: r.active_nodes,
                current_best: r.current_best.map(Into::into),
            },
            keyforge_model::job::JobStatus::Completed(c) => Self::Completed {
                final_score: c.final_score.into(),
                final_layout: c.final_layout.into(),
                total_compute_sec: c.total_compute_sec,
            },
        }
    }
}
/// Security-bounded collections.
pub mod limited;
pub use limited::LimitedVec;
