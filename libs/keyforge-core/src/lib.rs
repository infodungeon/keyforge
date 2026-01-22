// libs/keyforge-core/src/lib.rs

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

//! # `KeyForge` Core
//!
//! Pure orchestration and domain-agnostic helpers. This crate provides
//! the glue between physics and evolution without being tied to IO or
//! specific protocols.

/// Traits and types for loading external assets (keyboards, corpora, etc.).
pub mod loader;
/// High-level session management for optimization runs.
pub mod session;
pub use keyforge_evolution::{EvolutionError, OptimizationControl, ProgressCallback, NoOpCallback};
pub use session::ScoringSession;

pub use keyforge_physics::{
    verify::DeterministicScorer, LayoutIdentity, PhysicsError, ScoringEngine, EngineFactory
};

use keyforge_model::{AnalysisReport, EngineRequest, Layout, OptimizationResult, SwapSuggestion};
use std::sync::Arc;

/// Build a compiled `ScoringEngine` from an `EngineRequest`.
///
/// # Errors
/// Returns `PhysicsError` if the engine building fails.
pub fn build_engine(req: &EngineRequest) -> Result<Box<dyn ScoringEngine>, PhysicsError> {
    keyforge_physics::EngineFactory::new_generic(
        &req.keyboard,
        &req.corpus,
        &req.rubric,
        &req.cost_model,
    )
}

/// Analyze a layout using a compiled engine.
///
/// # Errors
/// Returns `PhysicsError` if analysis fails.
pub fn analyze_with_engine(
    engine: &dyn ScoringEngine,
    layout: &Layout,
) -> Result<AnalysisReport, PhysicsError> {
    engine.analyze(layout)
}

/// Score a layout using a compiled engine.
///
/// # Errors
/// Returns `PhysicsError` if scoring fails.
pub fn score_with_engine(engine: &dyn ScoringEngine, layout: &Layout) -> Result<f32, PhysicsError> {
    Ok(engine.score(layout)?.to_f32())
}

/// Suggest swaps using a compiled engine.
///
/// # Errors
/// Returns `PhysicsError` if suggestion logic fails.
pub fn suggest_with_engine(
    engine: &dyn ScoringEngine,
    layout: &Layout,
) -> Result<Vec<SwapSuggestion>, PhysicsError> {
    Ok(engine.suggest_improvements(layout, true))
}

/// Identify a layout fingerprint.
#[must_use]
pub fn identify(layout: &Layout) -> Option<LayoutIdentity> {
    keyforge_physics::identify(layout)
}

/// Optimize using a precompiled engine.
///
/// # Errors
/// Returns `EvolutionError` if optimization fails.
pub fn optimize_with_engine<CB: ProgressCallback>(
    engine: &Arc<dyn ScoringEngine>,
    config: &keyforge_model::SearchConfig,
    callback: CB,
    initial_layout: Option<Layout>,
    pinned_keys: Option<&[Option<keyforge_model::KeyCode>]>,
) -> Result<OptimizationResult, EvolutionError> {
    keyforge_evolution::evolve(engine, config, callback, initial_layout, pinned_keys)
}