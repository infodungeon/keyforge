// libs/keyforge-model/src/config/weights/mod.rs

/// Weight accessors.
pub mod accessors;
/// CLI configuration for weights.
pub mod cli;
/// Weight configuration structures.
pub mod config;
/// Default constants for scoring weights.
pub mod constants;

#[cfg(feature = "cli")]
pub use cli::ScoringWeightsConfig;
pub use config::ScoringWeights;
pub use constants::*;
