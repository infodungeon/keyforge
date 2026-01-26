// libs/keyforge-model/src/config/weights/mod.rs

/// Weight configuration structures.
pub mod config;
/// Default constants for scoring weights.
pub mod constants;

pub use config::ScoringWeights;
#[cfg(feature = "cli")]
pub use config::ScoringWeightsConfig;
pub use constants::*;
