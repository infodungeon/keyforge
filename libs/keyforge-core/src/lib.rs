// libs/keyforge-core/src/lib.rs

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

//! # `KeyForge` Core
//!
//! Pure orchestration and domain-agnostic helpers. This crate provides
//! the glue between physics and evolution without being tied to IO or
//! specific protocols.
//!
//! This crate is intentionally IO-free. It provides pure helper functions for:
//! - building physics engines from fully-loaded domain inputs
//! - running analysis
//! - running optimization

/// Traits and types for loading external assets (keyboards, corpora, etc.).
pub mod loader;
/// High-level session management for optimization runs.
pub mod session;
pub use keyforge_evolution::{EvolutionError, ProgressCallback};
pub use session::ScoringSession;

pub use keyforge_physics::{
    verify::DeterministicScorer, EngineRequest, LayoutIdentity, PhysicsError, ScoringEngine,
};

use keyforge_model::{AnalysisReport, Layout, OptimizationResult, SwapSuggestion};
use std::sync::Arc;

/// Build a compiled `ScoringEngine` from an `EngineRequest`.
///
/// Builds a scoring engine.
///
/// # Errors
///
/// Returns `PhysicsError` if the engine building fails.
pub fn build_engine(req: &EngineRequest) -> Result<Box<dyn ScoringEngine>, PhysicsError> {
    keyforge_physics::EngineFactory::new_generic(&req.keyboard, &req.corpus, &req.rubric, &req.cost_model)
}

/// Analyze a layout using a compiled engine.
/// Analyzes a layout with a pre-built engine.
///
/// # Errors
///
/// Returns `PhysicsError` if analysis fails.
pub fn analyze_with_engine(
    engine: &dyn ScoringEngine,
    layout: &Layout,
) -> Result<AnalysisReport, PhysicsError> {
    engine.analyze(layout)
}

/// Score a layout using a compiled engine.
/// Scores a layout with a pre-built engine.
///
/// # Errors
///
/// Returns `PhysicsError` if scoring fails.
pub fn score_with_engine(engine: &dyn ScoringEngine, layout: &Layout) -> Result<f32, PhysicsError> {
    Ok(engine.score(layout)?.to_f32())
}

/// Suggest swaps using a compiled engine.
/// Suggests swaps for a layout.
///
/// # Errors
///
/// Returns `PhysicsError` if suggestion logic fails.
pub fn suggest_with_engine(
    engine: &dyn ScoringEngine,
    layout: &Layout,
) -> Result<Vec<SwapSuggestion>, PhysicsError> {
    Ok(engine.suggest_improvements(layout, true))
}

/// Legacy-style analysis: compiles an engine from the request and analyzes the request layout.
/// If no layout is provided, uses a default 0-filled layout.
/// Analyzes a request.
///
/// # Errors
///
/// Returns `PhysicsError` if analysis fails.
pub fn analyze(req: &EngineRequest) -> Result<AnalysisReport, PhysicsError> {
    keyforge_physics::analyze(req)
}

/// Legacy-style score: compiles an engine from the request and scores the request layout.
/// Scores a request.
///
/// # Errors
///
/// Returns `PhysicsError` if scoring fails.
pub fn score(req: &EngineRequest) -> Result<OptimizationResult, PhysicsError> {
    keyforge_physics::score(req)
}

/// Legacy-style swap suggestions: compiles an engine from the request and suggests improvements.
/// Suggests swaps for a request.
///
/// # Errors
///
/// Returns `PhysicsError` if suggestion fails.
pub fn suggest(req: &EngineRequest) -> Result<Vec<SwapSuggestion>, PhysicsError> {
    keyforge_physics::suggest_improvements(req)
}

/// Identify a layout fingerprint.
#[must_use]
pub fn identify(layout: &Layout) -> Option<LayoutIdentity> {
    keyforge_physics::identify(layout)
}

/// Optimize using the legacy request style (engine compiled internally).
///
/// Prefer `optimize_with_engine` when you already have a compiled engine.
/// Optimizes a layout.
///
/// # Errors
///
/// Returns `EvolutionError` if optimization fails.
pub fn optimize(req: &EngineRequest) -> Result<OptimizationResult, EvolutionError> {
    keyforge_evolution::optimize(req)
}

/// Optimize using the legacy request style, reporting progress via callback.
/// Optimize using the legacy request style, reporting progress via callback.
/// Optimizes with a callback.
///
/// # Errors
///
/// Returns `EvolutionError` if optimization fails.
pub fn optimize_with_callback<CB: ProgressCallback>(
    req: &EngineRequest,
    callback: CB,
) -> Result<OptimizationResult, EvolutionError> {
    keyforge_evolution::optimize_with_callback(req, callback)
}

/// Optimize using a precompiled engine.
/// Optimizes with a pre-built engine.
///
/// # Errors
///
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

#[cfg(test)]
mod tests {
    use super::*;
    use keyforge_model::types::{FingerIndex, HandIndex, KeyCode};
    use keyforge_model::{Corpus, CostModel, KeyNode, Keyboard, Rubric};

    fn setup_minimal_req() -> EngineRequest {
        let keys = vec![KeyNode {
            index: 0,
            hand: HandIndex(0),
            finger: FingerIndex(1),
            ..Default::default()
        }];
        let kb = Arc::new(Keyboard::new(keys, 0).unwrap());
        let cp = Arc::new(Corpus::default());
        let rb = Arc::new(Rubric::default());

        let mut cost_model: CostModel = serde_json::from_str(
            r#"{
            "meta": {"version":"1", "description":"test", "unit":"pts"},
            "models": {},
            "dynamic_rules": {"sequence_modifiers": {}, "penalties": {}, "constraints": {}}
        }"#,
        )
        .unwrap();

        cost_model.models.insert(
            "model_a_row_staggered".into(),
            keyforge_model::cost_model::ModelDefinition {
                description: "test".into(),
                static_costs: std::collections::HashMap::new(),
            },
        );

        EngineRequest {
            keyboard: kb,
            corpus: cp,
            rubric: rb,
            cost_model: Arc::new(cost_model),
            config: keyforge_model::SearchConfig::default(),
            initial_layout: Some(Layout::new_unchecked(vec![KeyCode(0)])),
            pinned_keys: vec![],
        }
    }

    #[test]
    fn test_build_engine_success() {
        let req = setup_minimal_req();
        let engine = build_engine(&req).unwrap();
        assert_eq!(engine.key_count(), 1);
    }

    #[test]
    fn test_analysis_wrappers() {
        let req = setup_minimal_req();
        let engine = build_engine(&req).unwrap();
        let layout = req.initial_layout.as_ref().unwrap();

        assert!(analyze_with_engine(engine.as_ref(), layout).is_ok());
        assert!(score_with_engine(engine.as_ref(), layout).is_ok());
        assert!(suggest_with_engine(engine.as_ref(), layout).is_ok());
        
        assert!(analyze(&req).is_ok());
        assert!(score(&req).is_ok());
        assert!(suggest(&req).is_ok());
    }

    #[test]
    fn test_optimization_wrappers() {
        let req = setup_minimal_req();
        let engine: Arc<dyn ScoringEngine> = build_engine(&req).unwrap().into();
        
        assert!(optimize(&req).is_ok());
        
        struct NoOpCallback;
        impl ProgressCallback for NoOpCallback {
            fn on_progress(&self, _epoch: usize, _score: f32, _layout: &[KeyCode], _ips: f32) -> bool { true }
        }
        
        assert!(optimize_with_callback(&req, NoOpCallback).is_ok());
        assert!(optimize_with_engine(&engine, &req.config, NoOpCallback, None, None).is_ok());
    }

    #[test]
    fn test_identity_wrapper() {
        let layout = Layout::new_unchecked(vec![KeyCode(0)]);
        // identify might return None if layout doesn't match any known fingerprint
        let _ = identify(&layout);
    }
}
