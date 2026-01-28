// libs/keyforge-protocol/src/community.rs

use crate::assets::LayoutDto;
use crate::types::{CorpusIdDto, KeyboardIdDto, LayoutIdDto, ScoreDto, UserIdDto};
use keyforge_model::community::{AnalysisSession, LayoutSubmission};
use serde::{Deserialize, Serialize};
#[cfg(feature = "ts_bindings")]
use ts_rs::TS;
use utoipa::ToSchema;

/// DTO for `LayoutSubmission`.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct LayoutSubmissionDto {
    /// Unique submission ID.
    pub id: String,
    /// Original author.
    pub author_id: UserIdDto,
    /// Physical keyboard ID.
    pub keyboard_id: KeyboardIdDto,
    /// The character mapping.
    pub layout: LayoutDto,
    /// Ergonomic score.
    pub score: ScoreDto,
    /// Community-provided tags.
    pub tags: Vec<String>,
    /// Creation timestamp.
    pub created_at: u64,
}

impl From<LayoutSubmission> for LayoutSubmissionDto {
    fn from(val: LayoutSubmission) -> Self {
        Self {
            id: val.id,
            author_id: val.author_id.into(),
            keyboard_id: val.keyboard_id.into(),
            layout: val.layout.into(),
            score: val.score.into(),
            tags: val.tags,
            created_at: val.created_at,
        }
    }
}

/// DTO for `AnalysisSession`.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct AnalysisSessionDto {
    /// Session ID.
    pub id: String,
    /// Target user.
    pub user_id: UserIdDto,
    /// Physical target.
    pub keyboard_id: KeyboardIdDto,
    /// Language target.
    pub corpus_id: CorpusIdDto,
    /// History of layout revisions.
    pub history: Vec<AnalysisSessionEntryDto>,
}

impl From<AnalysisSession> for AnalysisSessionDto {
    fn from(val: AnalysisSession) -> Self {
        Self {
            id: val.id,
            user_id: val.user_id.into(),
            keyboard_id: val.keyboard_id.into(),
            corpus_id: val.corpus_id.into(),
            history: val.history.into_iter().map(Into::into).collect(),
        }
    }
}

/// DTO for `AnalysisSessionEntry`.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct AnalysisSessionEntryDto {
    /// Layout revision ID.
    pub layout_id: LayoutIdDto,
    /// Performance score.
    pub score: ScoreDto,
    /// Occurrence timestamp.
    pub timestamp: u64,
}

impl From<keyforge_model::community::AnalysisSessionEntry> for AnalysisSessionEntryDto {
    fn from(val: keyforge_model::community::AnalysisSessionEntry) -> Self {
        Self {
            layout_id: val.layout_id.into(),
            score: val.score.into(),
            timestamp: val.timestamp,
        }
    }
}
