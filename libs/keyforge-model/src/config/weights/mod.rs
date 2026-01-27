// libs/keyforge-model/src/config/weights/mod.rs

/// Weight configuration structures.
pub mod config;
/// Default constants for scoring weights.
pub mod constants;
/// Weight accessors.
pub mod accessors;
/// CLI configuration for weights.
pub mod cli;

pub use config::ScoringWeights;
#[cfg(feature = "cli")]
pub use cli::ScoringWeightsConfig;
pub use constants::*;
