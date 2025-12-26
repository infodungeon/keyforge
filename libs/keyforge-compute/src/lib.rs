use keyforge_core::ProgressCallback;
use keyforge_core::{LayoutIdentity, ScoringEngine};
use keyforge_model::{AnalysisReport, Layout, OptimizationResult, SearchConfig, SwapSuggestion};
use keyforge_protocol::keycodes::KeycodeRegistry;
use std::sync::Arc;
use tracing::instrument;

/// The pure computation runtime.
/// This struct holds the compiled physics engine and configuration necessary
/// to perform scoring, analysis, and optimization.
/// It is strictly decoupled from file I/O.
#[derive(Clone)]
pub struct Runtime {
    pub engine: Arc<ScoringEngine>,
    pub registry: Arc<KeycodeRegistry>,
    pub search_config: SearchConfig,
}

impl Runtime {
    pub fn new(
        engine: Arc<ScoringEngine>,
        registry: Arc<KeycodeRegistry>,
        search_config: SearchConfig,
    ) -> Self {
        Self {
            engine,
            registry,
            search_config,
        }
    }

    #[instrument(skip(self, layout))]
    pub fn score(&self, layout: &Layout) -> f32 {
        self.engine.score(layout)
    }

    #[instrument(skip(self, layout))]
    pub fn analyze(&self, layout: &Layout) -> AnalysisReport {
        self.engine.analyze(layout)
    }

    #[instrument(skip(self, layout))]
    pub fn identify(&self, layout: &Layout) -> Option<LayoutIdentity> {
        keyforge_core::identify(layout)
    }

    #[instrument(skip(self, layout))]
    pub fn suggest_improvements(&self, layout: &Layout) -> Vec<SwapSuggestion> {
        self.engine.suggest_improvements(layout)
    }

    #[instrument(skip(self, callback))]
    pub fn optimize(&self, callback: impl ProgressCallback) -> OptimizationResult {
        keyforge_core::optimize_with_engine(self.engine.clone(), &self.search_config, callback)
    }
}
