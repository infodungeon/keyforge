// libs/keyforge-model/src/config/aggregate.rs

use crate::config::definitions::LayoutDefinitions;
use crate::config::search::SearchParams;
use crate::config::weights::ScoringWeights;
use crate::validator::Validator;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_aggregate_validation() {
        let mut config = Config::default();
        assert!(config.validate().is_ok());

        // Trigger failure in one of the children
        config.defs.tier_high_chars = "".into();
        assert!(config.validate().is_err());
    }
}
