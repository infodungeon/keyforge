// libs/keyforge-model/src/config/aggregate.rs

use crate::config::definitions::LayoutDefinitions;
use crate::config::search::SearchParams;
use crate::config::weights::ScoringWeights;
use crate::validator::Validator;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
#[cfg(feature = "ts_bindings")]
use ts_rs::TS;


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
}

impl Validator for Config {
    fn validate(&self) -> Result<(), String> {
        self.search.validate()?;
        self.weights.validate()?;
        self.defs.validate()?;
        Ok(())
    }
}
