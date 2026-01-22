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

//! # `KeyForge` Compute
//!
//! High-level computation runtime for `KeyForge`. This crate orchestrates the
//! physics and evolution engines to provide a unified runtime for
//! applications.

use keyforge_core::{EvolutionError, ProgressCallback, ScoringSession};
/// Biometric profiling logic.
pub mod biometrics;
/// Builder for constructing computation sessions.
pub mod builder;
/// Hardware detection and engine selection.
pub mod hardware;
pub use builder::SessionBuilder;
use keyforge_model::keycodes::KeycodeRegistry;
use keyforge_model::{
    AnalysisReport, EngineRequest, KeyCode, Layout, OptimizationResult, ScoringResult, SearchConfig,
    SwapSuggestion,
};
use keyforge_physics::{EngineFactory, ScoringEngine};
use std::sync::Arc;
use tracing::instrument;

/// Performs a one-off scoring operation for the given request.
///
/// # Errors
///
/// Returns a `keyforge_physics::PhysicsError` if the engine initialization or scoring fails.
#[instrument(skip(req))]
pub fn score(req: &EngineRequest) -> Result<ScoringResult, keyforge_physics::PhysicsError> {
    let engine =
        EngineFactory::new_generic(&req.keyboard, &req.corpus, &req.rubric, &req.cost_model)?;
    let layout = req
        .initial_layout
        .clone()
        .unwrap_or_else(|| Layout::new_unchecked(vec![KeyCode::EMPTY; engine.key_count()]));

    Ok(ScoringResult {
        score: engine.score(&layout)?.to_f32(),
        layout,
    })
}

/// Performs a one-off analysis operation for the given request.
///
/// # Errors
///
/// Returns a `keyforge_physics::PhysicsError` if the engine initialization or analysis fails.
#[instrument(skip(req))]
pub fn analyze(req: &EngineRequest) -> Result<AnalysisReport, keyforge_physics::PhysicsError> {
    let engine =
        EngineFactory::new_generic(&req.keyboard, &req.corpus, &req.rubric, &req.cost_model)?;
    let layout = req
        .initial_layout
        .clone()
        .unwrap_or_else(|| Layout::new_unchecked(vec![KeyCode::EMPTY; engine.key_count()]));
    engine.analyze(&layout)
}

/// Suggests improvements for the layout described in the request.
///
/// # Errors
///
/// Returns `keyforge_physics::PhysicsError` if the engine cannot be compiled or if an error occurs.
#[instrument(skip(req))]
pub fn suggest_improvements(
    req: &EngineRequest,
) -> Result<Vec<SwapSuggestion>, keyforge_physics::PhysicsError> {
    let engine =
        EngineFactory::new_generic(&req.keyboard, &req.corpus, &req.rubric, &req.cost_model)?;
    let layout = req
        .initial_layout
        .clone()
        .unwrap_or_else(|| Layout::new_unchecked(vec![KeyCode::EMPTY; engine.key_count()]));

    let raw_suggestions = engine.suggest_improvements(&layout, req.config.include_thumbs());

    // Filter out suggestions that involve pinned keys
    let filtered: Vec<SwapSuggestion> = raw_suggestions
        .into_iter()
        .filter(|s| {
            let pin_a = req
                .pinned_keys
                .get(s.index_a)
                .and_then(|p| p.as_ref())
                .is_some();
            let pin_b = req
                .pinned_keys
                .get(s.index_b)
                .and_then(|p| p.as_ref())
                .is_some();
            !pin_a && !pin_b
        })
        .collect();

    Ok(filtered)
}

/// Performs a one-off optimization operation.
///
/// # Errors
/// Returns `EvolutionError` if optimization fails.
pub fn optimize(req: &EngineRequest) -> Result<OptimizationResult, EvolutionError> {
    let engine = EngineFactory::new_generic(&req.keyboard, &req.corpus, &req.rubric, &req.cost_model)
        .map_err(EvolutionError::Physics)?;
    let engine_arc: Arc<dyn ScoringEngine> = Arc::from(engine);
    keyforge_core::optimize_with_engine(
        &engine_arc,
        &req.config,
        keyforge_core::NoOpCallback,
        req.initial_layout.clone(),
        Some(&req.pinned_keys),
    )
}

/// Performs a one-off optimization operation with a progress callback.
///
/// # Errors
/// Returns `EvolutionError` if optimization fails.
pub fn optimize_with_callback<CB: ProgressCallback>(
    req: &EngineRequest,
    callback: CB,
) -> Result<OptimizationResult, EvolutionError> {
    let engine = EngineFactory::new_generic(&req.keyboard, &req.corpus, &req.rubric, &req.cost_model)
        .map_err(EvolutionError::Physics)?;
    let engine_arc: Arc<dyn ScoringEngine> = Arc::from(engine);
    keyforge_core::optimize_with_engine(
        &engine_arc,
        &req.config,
        callback,
        req.initial_layout.clone(),
        Some(&req.pinned_keys),
    )
}

/// The pure computation runtime.
#[derive(Clone, Debug)]
pub struct Runtime {
    /// The underlying physical scoring engine.
    pub engine: Arc<dyn ScoringEngine>,
    /// Registry of all valid keycodes.
    pub registry: Arc<KeycodeRegistry>,
    /// Global configuration for search and optimization.
    pub search_config: SearchConfig,
}

impl Runtime {
    /// Creates a new `Runtime` from initialized components.
    #[must_use]
    pub fn new(
        engine: Arc<dyn ScoringEngine>,
        registry: Arc<KeycodeRegistry>,
        search_config: SearchConfig,
    ) -> Self {
        Self {
            engine,
            registry,
            search_config,
        }
    }

    /// Evaluates the physical cost of a layout.
    ///
    /// # Errors
    ///
    /// Returns `keyforge_physics::PhysicsError` if evaluation fails.
    #[instrument(skip(self, layout))]
    pub fn score(&self, layout: &Layout) -> Result<f32, keyforge_physics::PhysicsError> {
        Ok(self.engine.score(layout)?.to_f32())
    }

    /// Generates a comprehensive ergonomics analysis for a layout.
    ///
    /// # Errors
    ///
    /// Returns `keyforge_physics::PhysicsError` if analysis fails.
    #[instrument(skip(self, layout))]
    pub fn analyze(
        &self,
        layout: &Layout,
    ) -> Result<AnalysisReport, keyforge_physics::PhysicsError> {
        self.engine.analyze(layout)
    }

    /// Suggests layout improvements based on the current scoring model.
    ///
    /// # Errors
    ///
    /// Returns `keyforge_physics::PhysicsError` if suggestion logic fails.
    #[instrument(skip(self, layout))]
    pub fn suggest_improvements(
        &self,
        layout: &Layout,
    ) -> Result<Vec<SwapSuggestion>, keyforge_physics::PhysicsError> {
        Ok(self
            .engine
            .suggest_improvements(layout, self.search_config.include_thumbs()))
    }

    /// Runs the evolution optimizer on the current runtime context.
    ///
    /// # Errors
    ///
    /// Returns `EvolutionError` if optimization fails.
    #[instrument(skip(self, callback))]
    pub fn optimize(
        &self,
        callback: impl ProgressCallback,
        initial_layout: Option<Layout>,
        pinned_keys: Option<&[Option<keyforge_model::KeyCode>]>,
    ) -> Result<OptimizationResult, EvolutionError> {
        keyforge_core::optimize_with_engine(
            &self.engine,
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
