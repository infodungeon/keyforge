// Copyright (c) 2025 KeyForge Contributors
//
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
//! Core orchestration helpers.
//!
//! This crate is intentionally IO-free. It provides pure helper functions for:
//! - building physics engines from fully-loaded domain inputs
//! - running analysis
//! - running optimization

pub mod loader;
pub mod session;
pub use session::ScoringSession;
pub use keyforge_evolution::{ProgressCallback, EvolutionError};

pub use keyforge_physics::{
    verify::DeterministicScorer, EngineRequest, LayoutIdentity, ScoringEngine, PhysicsError,
};

use keyforge_model::{AnalysisReport, Layout, OptimizationResult, SwapSuggestion};
use std::sync::Arc;

/// Build a compiled `ScoringEngine` from an `EngineRequest`.
///
/// This is a convenience wrapper around `ScoringEngine::new`.
pub fn build_engine(req: &EngineRequest) -> Result<ScoringEngine, PhysicsError> {
    ScoringEngine::new(&req.keyboard, &req.corpus, &req.rubric, &req.cost_overrides)
}

/// Analyze a layout using a compiled engine.
pub fn analyze_with_engine(engine: &ScoringEngine, layout: &Layout) -> Result<AnalysisReport, PhysicsError> {
    engine.analyze(layout)
}

/// Score a layout using a compiled engine.
pub fn score_with_engine(engine: &ScoringEngine, layout: &Layout) -> Result<f32, PhysicsError> {
    engine.score(layout)
}

/// Suggest swaps using a compiled engine.
pub fn suggest_with_engine(engine: &ScoringEngine, layout: &Layout) -> Result<Vec<SwapSuggestion>, PhysicsError> {
    Ok(engine.suggest_improvements(layout))
}

/// Legacy-style analysis: compiles an engine from the request and analyzes the request layout.
/// If no layout is provided, uses a default 0-filled layout.
pub fn analyze(req: &EngineRequest) -> Result<AnalysisReport, PhysicsError> {
    keyforge_physics::analyze(req)
}

/// Legacy-style score: compiles an engine from the request and scores the request layout.
pub fn score(req: &EngineRequest) -> Result<OptimizationResult, PhysicsError> {
    keyforge_physics::score(req)
}

/// Legacy-style swap suggestions: compiles an engine from the request and suggests improvements.
pub fn suggest(req: &EngineRequest) -> Result<Vec<SwapSuggestion>, PhysicsError> {
    keyforge_physics::suggest_improvements(req)
}

/// Identify a layout fingerprint.
pub fn identify(layout: &Layout) -> Option<LayoutIdentity> {
    keyforge_physics::identify(layout)
}

/// Optimize using the legacy request style (engine compiled internally).
///
/// Prefer `optimize_with_engine` when you already have a compiled engine.
pub fn optimize(req: &EngineRequest) -> Result<OptimizationResult, EvolutionError> {
    keyforge_evolution::optimize(req)
}

/// Optimize using the legacy request style, reporting progress via callback.
pub fn optimize_with_callback<CB: keyforge_evolution::ProgressCallback>(
    req: &EngineRequest,
    callback: CB,
) -> Result<OptimizationResult, EvolutionError> {
    keyforge_evolution::optimize_with_callback(req, callback)
}

/// Optimize using a precompiled engine.
///
/// This is the most efficient path for environments that repeatedly optimize against
/// the same keyboard/corpus/rubric.
pub fn optimize_with_engine<CB: keyforge_evolution::ProgressCallback>(
    engine: Arc<ScoringEngine>,
    config: &keyforge_model::SearchConfig,
    callback: CB,
) -> Result<OptimizationResult, EvolutionError> {
    keyforge_evolution::evolve(engine, config, callback)
}
