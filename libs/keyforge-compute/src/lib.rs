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
use keyforge_core::{ProgressCallback, EvolutionError, ScoringSession};
pub mod builder;
pub use builder::SessionBuilder;
use keyforge_model::{AnalysisReport, Layout, OptimizationResult, SwapSuggestion, SearchConfig};
use keyforge_model::keycodes::KeycodeRegistry;
use keyforge_physics::ScoringEngine;
use std::sync::Arc;
use tracing::instrument;

/// The pure computation runtime.
#[derive(Clone)]
pub struct Runtime {
    pub engine: Arc<ScoringEngine>,
    pub registry: Arc<KeycodeRegistry>,
    pub search_config: SearchConfig,
}

impl Runtime {
    pub fn new(engine: Arc<ScoringEngine>, registry: Arc<KeycodeRegistry>, search_config: SearchConfig) -> Self {
        Self { engine, registry, search_config }
    }

    #[instrument(skip(self, layout))]
    pub fn score(&self, layout: &Layout) -> Result<f32, keyforge_physics::PhysicsError> {
        self.engine.score(layout)
    }

    #[instrument(skip(self, layout))]
    pub fn analyze(&self, layout: &Layout) -> Result<AnalysisReport, keyforge_physics::PhysicsError> {
        self.engine.analyze(layout)
    }

    #[instrument(skip(self, layout))]
    pub fn suggest_improvements(&self, layout: &Layout) -> Result<Vec<SwapSuggestion>, keyforge_physics::PhysicsError> {
        Ok(self.engine.suggest_improvements(layout))
    }

    #[instrument(skip(self, callback))]
    pub fn optimize(&self, callback: impl ProgressCallback) -> Result<OptimizationResult, EvolutionError> {
        keyforge_core::optimize_with_engine(
            self.engine.clone(),
            &self.search_config,
            callback
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
