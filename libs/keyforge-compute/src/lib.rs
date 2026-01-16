// libs/keyforge-compute/src/lib.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # KeyForge Compute
//!
//! High-level computation runtime for KeyForge. This crate orchestrates the 
//! physics and evolution engines to provide a unified runtime for 
//! applications.

use keyforge_core::{ProgressCallback, EvolutionError, ScoringSession};
/// Builder for constructing computation sessions.
pub mod builder;
pub use builder::SessionBuilder;
use keyforge_model::{AnalysisReport, Layout, OptimizationResult, SwapSuggestion, SearchConfig};
use keyforge_model::keycodes::KeycodeRegistry;
use keyforge_physics::ScoringEngine;
use std::sync::Arc;
use tracing::instrument;

/// The pure computation runtime.
#[derive(Clone, Debug)]
pub struct Runtime {
    /// The underlying physical scoring engine.
    pub engine: Arc<ScoringEngine>,
    /// Registry of all valid keycodes.
    pub registry: Arc<KeycodeRegistry>,
    /// Global configuration for search and optimization.
    pub search_config: SearchConfig,
}

impl Runtime {
    /// Creates a new `Runtime` from initialized components.
    pub fn new(engine: Arc<ScoringEngine>, registry: Arc<KeycodeRegistry>, search_config: SearchConfig) -> Self {
        Self { engine, registry, search_config }
    }

    /// Evaluates the physical cost of a layout.
    #[instrument(skip(self, layout))]
    pub fn score(&self, layout: &Layout) -> Result<f32, keyforge_physics::PhysicsError> {
        self.engine.score(layout)
    }

    /// Generates a comprehensive ergonomics analysis for a layout.
    #[instrument(skip(self, layout))]
    pub fn analyze(&self, layout: &Layout) -> Result<AnalysisReport, keyforge_physics::PhysicsError> {
        self.engine.analyze(layout)
    }

    /// Suggests layout improvements based on the current scoring model.
    #[instrument(skip(self, layout))]
    pub fn suggest_improvements(&self, layout: &Layout) -> Result<Vec<SwapSuggestion>, keyforge_physics::PhysicsError> {
        Ok(self.engine.suggest_improvements(layout, self.search_config.include_thumbs()))
    }

    /// Runs the evolution optimizer on the current runtime context.
    #[instrument(skip(self, callback))]
    pub fn optimize(&self, callback: impl ProgressCallback, initial_layout: Option<Layout>, pinned_keys: Option<&[Option<keyforge_model::KeyCode>]>) -> Result<OptimizationResult, EvolutionError> {
        keyforge_core::optimize_with_engine(
            self.engine.clone(),
            &self.search_config,
            callback,
            initial_layout,
            pinned_keys,
        )
    }
}

impl From<ScoringSession> for Runtime {
    fn from(s: ScoringSession) -> Self {
        Self {
            engine: s.engine,
            registry: s.registry,
            search_config: s.search_config,
        }
    }
}
