/// The root configuration aggregate.
pub mod aggregate;
/// Key pinning constraints.
pub mod constraints;
/// Layout tier definitions.
pub mod definitions;
/// Hardware-specific optimization parameters.
pub mod engine;
/// Parameter metadata schemas.
pub mod metadata;
/// Search parameters.
pub mod search;
/// Data source definitions.
pub mod source;
/// Utility configuration helpers.
pub mod utils;
/// Scoring weights and penalties.
pub mod weights;

pub use aggregate::Config;
pub use constraints::KeyConstraint;
pub use definitions::LayoutDefinitions;
pub use engine::EngineConfig;
pub use metadata::{ParamType, ParameterMetadata};
pub use search::{SearchConfig, SearchParams};
pub use source::{CorpusSource, CostMatrixSource};
pub use weights::ScoringWeights;
