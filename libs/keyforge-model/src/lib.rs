// libs/keyforge-model/src/lib.rs

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

//! # `KeyForge` Model
//!
//! The Domain Nucleus of the `KeyForge` system. This crate defines the "Ubiquitous Language"
//! (Entities, Value Objects, Aggregates) used throughout the application.

#![warn(missing_docs)]

/// Asset management and loading markers.
pub mod asset;
/// Biometric profiling and timing models.
pub mod biometrics;
/// Community-driven entities and stateful analysis sessions.
pub mod community;
/// Configuration aggregates and parameter definitions.
pub mod config;
/// Global constants and safety limits.
pub mod constants;
/// Text corpus data structures (N-grams, frequencies).
pub mod corpus;
/// Data structures for the external Cost Matrix (Physics Model).
pub mod cost_model;
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
/// Standardized mapping between external data and domain models.
pub mod mapping;
pub mod metrics;
/// Scoring configuration and weights.
pub mod rubric;
/// Core domain types and newtypes.
#[path = "types/mod.rs"]
pub mod types;
/// User identity and preference aggregates.
pub mod user;
/// Internal utilities (private).
pub mod utils;
/// Validation traits and helpers.
pub mod validator;

/// Testing utilities and proptest strategies.
#[cfg(any(test, feature = "testing"))]
pub mod testing;

pub use asset::{Asset, AssetCategory};
pub use biometrics::{BiometricProfile, BiometricSample, LatencyStats};
pub use community::{AnalysisSession, AnalysisSessionEntry, KeyboardInventory, LayoutSubmission};
pub use config::aggregate::{Config, EngineRequest};
pub use config::constraints::KeyConstraint;
pub use config::search::{SearchConfig, SearchParams};
pub use config::source::{CorpusSource, CostMatrixSource};
pub use config::weights::ScoringWeights;
pub use corpus::Corpus;
pub use cost_model::CostModel;
pub use error::ForgeError;
pub use geometry::{KeyNode, KeyboardDefinition, KeyboardGeometry};
pub use job::{Completed, Job, JobIdentifier, JobState, JobStatus, Pending, Running};
pub use keyboard::{Keyboard, SpatialIndex};
pub use keycodes::KeycodeRegistry;
pub use layout::Layout;
pub use mapping::{BulkProjection, Projection};
pub use metrics::{MetricId, MetricSet};
pub use rubric::Rubric;
pub use types::{
    AnalysisReport, ColIndex, CorpusId, FingerIndex, HandIndex, Heatmaps, KeyCode, KeyIndex,
    KeyboardId, LayoutId, MetricBreakdown, MetricViolation, OptimizationResult, RowIndex, Score,
    ScoreSummary, ScoringResult, SpaceHandPreference, SwapSuggestion, TravelStatistics, UserId,
};
pub use user::{UserBiometricStatus, UserPreferences, UserProfile};
pub use validator::{LayoutValidator, Validator};
