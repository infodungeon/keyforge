// libs/keyforge-model/src/lib.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.


//! # `KeyForge` Model
//!
//! The Domain Nucleus of the `KeyForge` system. This crate defines the "Ubiquitous Language"
//! (Entities, Value Objects, Aggregates) used throughout the application.
//!
//! ## Core Concepts
//!
//! * **Physical Domain:** [`Keyboard`], [`KeyNode`], [`KeyIndex`].
//! * **Logical Domain:** [`Layout`], [`KeyCode`].
//! * **Analysis Domain:** [`Corpus`], [`Rubric`], [`Score`].
//! * **Optimization Domain:** [`SearchConfig`], [`OptimizationResult`].
//!
//! ## Invariants
//!
//! This crate enforces validity through the Type System (Newtypes) and `TryFrom` implementations.
//! It adheres to the "Parse, Don't Validate" philosophy.

#![warn(missing_docs)]

/// Configuration aggregates and parameter definitions.
pub mod config;
/// Global constants and safety limits.
pub mod constants;
/// Text corpus data structures (N-grams, frequencies).
pub mod corpus;
/// Data structures for the external Cost Matrix (Physics Model).
pub mod cost_model;
/// Asset management and loading markers.
pub mod asset;
/// Centralized error types for the domain.
pub mod error;
/// Physical keyboard geometry and spatial definitions.
pub mod geometry;
/// Job identification and hashing logic.
pub mod job;
/// The `Keyboard` aggregate root.
pub mod keyboard;
/// Key code definitions and registry.
pub mod keycodes;
/// The `Layout` entity (logical mapping).
pub mod layout;
/// Parsing logic for keymap formats (QMK/ZMK).
pub mod parsing;
/// Scoring configuration and weights.
pub mod rubric;
/// Core newtypes (`KeyIndex`, `Score`, etc.).
pub mod types;
/// Internal utilities (private).
pub mod utils;
/// Validation traits and helpers.
pub mod validator;


pub use config::{Config, CorpusSource, CostMatrixSource, KeyConstraint, ScoringWeights, SearchParams};
pub use corpus::Corpus;
pub use cost_model::CostModel;
pub use asset::{Asset, AssetCategory};
pub use error::ForgeError;
pub use geometry::{KeyboardDefinition, KeyboardGeometry, KeyNode, KeyboardMeta};
pub use job::JobIdentifier;
pub use keyboard::Keyboard;
pub use keycodes::KeycodeRegistry;
pub use layout::Layout;
pub use rubric::Rubric;
pub use types::{ColIndex, FingerIndex, HandIndex, KeyCode, KeyIndex, RowIndex, Score};
pub use validator::{LayoutValidator, Validator};

use serde::{Deserialize, Serialize};
#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

/// Configuration for the optimization search strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub enum SearchConfig {
    /// Simulated Annealing strategy.
    Annealing {
        /// Total number of mutation steps.
        steps: usize,
        /// Initial temperature (higher = more chaotic).
        start_temp: f32,
        /// Final temperature (lower = more greedy).
        end_temp: f32,
        /// PRNG seed for deterministic replay.
        seed: u64,
        /// Steps without improvement before reheating.
        patience: usize,
        /// Number of times to reheat.
        reheats: usize,
        /// Multiplier for `start_temp` when reheating.
        reheat_factor: f32,
        /// Whether to include thumb keys in swap suggestions.
        #[serde(default)]
        include_thumbs: bool,
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
            include_thumbs: false,
        }
    }
}

impl SearchConfig {
    /// Validates that configuration parameters are within safe bounds.
    /// Validates the search parameters.
    ///
    /// # Errors
    /// Returns a `ForgeError` if the parameters are out of reasonable bounds.
    pub fn validate(&self) -> Result<(), ForgeError> {
        match self {
            SearchConfig::Annealing {
                steps,
                start_temp,
                end_temp,
                reheat_factor,
                ..
            } => {
                if *steps == 0 {
                    return Err(ForgeError::InvalidData("Steps must be > 0".into()));
                }
                if *start_temp < 0.0 {
                    return Err(ForgeError::InvalidData("Start temp must be >= 0".into()));
                }
                if *end_temp < 0.0 {
                    return Err(ForgeError::InvalidData("End temp must be >= 0".into()));
                }
                if *reheat_factor <= 0.0 {
                    return Err(ForgeError::InvalidData("Reheat factor must be > 0".into()));
                }
            }
        }
        Ok(())
    }

    /// Returns whether thumb keys should be included in swap suggestions.
    #[must_use]
    pub fn include_thumbs(&self) -> bool {
        match self {
            SearchConfig::Annealing { include_thumbs, .. } => *include_thumbs,
        }
    }
}

/// Represents a specific N-gram that violates a metric threshold.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct MetricViolation {
    /// The keys involved (e.g., "TH").
    pub keys: String,
    /// The cost contribution of this violation.
    pub score: f32,
    /// The frequency of this N-gram.
    pub freq: f32,
}

/// Detailed breakdown of a layout's performance.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct AnalysisReport {
    /// Total weighted score (lower is better).
    pub score: f32,
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
    /// Scissor (adjacent finger stretch) score.
    pub scissors: f32,
    /// Redirect (direction change) score.
    pub redirects: f32,
    /// Inward roll score.
    pub rolls: f32,
    /// Total penalty contribution from SFBs.
    #[serde(default)]
    pub sfb_penalty: f32,
    /// Total penalty contribution from scissors.
    #[serde(default)]
    pub scissor_penalty: f32,
    /// Total penalty contribution from redirects.
    #[serde(default)]
    pub redir_penalty: f32,
    /// Total penalty contribution from rolls.
    #[serde(default)]
    pub roll_penalty: f32,
    /// Per-key usage heatmap.
    #[serde(default)]
    pub heatmap: Vec<f32>,
    /// Per-key penalty heatmap (Effort).
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
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct OptimizationResult {
    /// The final score achieved.
    pub score: f32,
    /// The optimized layout.
    pub layout: Layout,
}

/// A proposed change to the layout during optimization.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
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
