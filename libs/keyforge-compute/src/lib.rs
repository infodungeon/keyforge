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
use keyforge_model::{AnalysisReport, Layout, OptimizationResult, SearchConfig, SwapSuggestion};
use keyforge_physics::ScoringEngine;
use std::sync::Arc;
use tracing::instrument;

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
        Ok(self.engine.analyze(layout)?)
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

#[cfg(test)]
mod tests {
    use super::*;
    use keyforge_model::{KeyNode, Keyboard, Corpus, Rubric, CostModel};
    use keyforge_physics::EngineFactory;

    fn setup_runtime() -> Runtime {
        let kb = Keyboard::new(vec![KeyNode::default()], 0).unwrap();
        let mut cm = CostModel::default();
        cm.models.insert("model_a_row_staggered".into(), keyforge_model::cost_model::ModelDefinition {
            description: "test".into(),
            static_costs: std::collections::HashMap::new(),
        });
        let engine = EngineFactory::new_exact(&kb, &Corpus::default(), &Rubric::default(), &cm).unwrap();
        let registry = Arc::new(KeycodeRegistry::new_with_defaults());
        Runtime::new(Arc::from(engine), registry, SearchConfig::default())
    }

    #[test]
    fn test_runtime_methods() {
        struct NoOpCallback;
        impl keyforge_core::ProgressCallback for NoOpCallback {
            fn on_progress(&self, _epoch: usize, _score: f32, _layout: &[keyforge_model::KeyCode], _ips: f32) -> bool { true }
        }

        let mut rt = setup_runtime();
        let layout = Layout::new_unchecked(vec![keyforge_model::KeyCode(0)]);
        
        assert!(rt.score(&layout).is_ok());
        assert!(rt.analyze(&layout).is_ok());
        assert!(rt.suggest_improvements(&layout).is_ok());
        
        // Trigger include_thumbs branch
        rt.search_config = SearchConfig::Annealing { 
            steps: 10, start_temp: 1.0, end_temp: 0.1, seed: 0,
            patience: 10, reheats: 0, reheat_factor: 0.5, include_thumbs: true 
        };
        assert!(rt.suggest_improvements(&layout).is_ok());

        assert!(rt.optimize(NoOpCallback, None, None).is_ok());
    }

    #[test]
    fn test_runtime_from_session() {
        let kb = Keyboard::new(vec![KeyNode::default()], 0).unwrap();
        let mut cm = CostModel::default();
        cm.models.insert("model_a_row_staggered".into(), keyforge_model::cost_model::ModelDefinition {
            description: "test".into(),
            static_costs: std::collections::HashMap::new(),
        });
        let engine = EngineFactory::new_exact(&kb, &Corpus::default(), &Rubric::default(), &cm).unwrap();
        let session = ScoringSession::new(Arc::from(engine), Arc::new(KeycodeRegistry::default()), SearchConfig::default());
        
        let rt = Runtime::from(session);
        assert_eq!(rt.registry.definitions.len(), 0);
    }
}