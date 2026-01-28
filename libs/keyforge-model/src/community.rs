// libs/keyforge-model/src/community.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You    may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Community-driven entities and stateful analysis sessions.

use crate::layout::Layout;
use crate::types::{CorpusId, KeyboardId, LayoutId, Score, UserId};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// A layout shared with the community.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LayoutSubmission {
    /// Unique identifier for the submission.
    pub id: String,
    /// The user who submitted the layout.
    pub author_id: UserId,
    /// The physical keyboard this layout targets.
    pub keyboard_id: KeyboardId,
    /// The actual character mapping.
    pub layout: Layout,
    /// Performance score at the time of submission.
    pub score: Score,
    /// Community tags (e.g., "ergonomic", "gaming").
    pub tags: Vec<String>,
    /// When the submission was created.
    pub created_at: u64,
}

/// A registry of physical keyboards available to the user or system.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Default)]
pub struct KeyboardInventory {
    /// List of keyboard IDs in the inventory.
    pub keyboards: Vec<KeyboardId>,
    /// Last sync timestamp with the system repository.
    pub last_sync: u64,
}

/// A stateful session for comparing and refining layouts.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AnalysisSession {
    /// Unique identifier for the session.
    pub id: String,
    /// The user running the session.
    pub user_id: UserId,
    /// The target keyboard.
    pub keyboard_id: KeyboardId,
    /// The target corpus.
    pub corpus_id: CorpusId,
    /// History of layouts analyzed in this session.
    pub history: Vec<AnalysisSessionEntry>,
}

/// A single entry in an analysis session's history.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AnalysisSessionEntry {
    /// The layout identifier.
    pub layout_id: LayoutId,
    /// The score achieved.
    pub score: Score,
    /// Timestamp of the analysis.
    pub timestamp: u64,
}
