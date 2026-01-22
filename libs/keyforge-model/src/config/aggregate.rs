// libs/keyforge-model/src/config/aggregate.rs

use crate::config::definitions::LayoutDefinitions;
use crate::config::search::{SearchParams, SearchConfig};
use crate::config::weights::ScoringWeights;
use crate::validator::Validator;
use crate::corpus::Corpus;
use crate::keyboard::Keyboard;
use crate::rubric::Rubric;
use crate::cost_model::CostModel;
use crate::types::{KeyCode, Layout};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
#[cfg(feature = "ts_bindings")]
use ts_rs::TS;
use utoipa::ToSchema;

/// The root configuration aggregate for a `KeyForge` session.
#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct Config {
    /// Search parameters for the optimization engine.
    pub search: SearchParams,
    /// Weights for the physics scoring engine.
    pub weights: ScoringWeights,
    /// Definitions for layout tiers and critical bigrams.
    pub defs: LayoutDefinitions,
    /// Keys pinned to specific positions.
    #[serde(default)]
    pub pinned_keys: Vec<crate::config::constraints::KeyConstraint>,
}

impl Validator for Config {
    fn validate(&self) -> Result<(), String> {
        self.search.validate()?;
        self.weights.validate()?;
        self.defs.validate()?;
        for p in &self.pinned_keys {
            p.validate()?;
        }
        Ok(())
    }
}

/// A request structure for performing common engine operations.
#[derive(Clone, Debug)]
pub struct EngineRequest {
    /// The physical keyboard geometry.
    pub keyboard: Arc<Keyboard>,
    /// The language statistics to use.
    pub corpus: Arc<Corpus>,
    /// The ergonomic weights to apply.
    pub rubric: Arc<Rubric>,
    /// The cost model to use.
    pub cost_model: Arc<CostModel>,
    /// Optimization and search parameters.
    pub config: SearchConfig,
    /// The starting layout for the operation.
    pub initial_layout: Option<Layout>,
    /// Keys that must remain in their initial positions.
    pub pinned_keys: Vec<Option<KeyCode>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_aggregate_validation() {
        let mut config = Config::default();
        assert!(config.validate().is_ok());

        // Trigger failure in one of the children
        config.defs.tier_high_chars = String::new();
        assert!(config.validate().is_err());

        // Test invalid pinned keys
        let mut config = Config::default();
        config
            .pinned_keys
            .push(crate::config::constraints::KeyConstraint {
                index: crate::types::KeyIndex(0),
                key: String::new(),
            });
        assert!(config.validate().is_err());
    }
}
