use keyforge_model::SearchConfig;
use keyforge_model::keycodes::KeycodeRegistry;
use keyforge_physics::ScoringEngine;
use std::sync::Arc;

/// A consolidated environment for scoring and optimization.
/// Holds the compiled physics engine and associated metadata.
#[derive(Clone)]
pub struct ScoringSession {
    pub engine: Arc<ScoringEngine>,
    pub registry: Arc<KeycodeRegistry>,
    pub search_config: SearchConfig,
}

impl ScoringSession {
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
}
