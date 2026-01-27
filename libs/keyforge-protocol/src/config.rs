// libs/keyforge-protocol/src/config.rs

use crate::types::KeyIndexDto;
use crate::assets::KeyboardGeometryDto;
use keyforge_model::config as model;
use keyforge_model::geometry;
use keyforge_model::Validator;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[cfg(feature = "ts_bindings")]
use ts_rs::TS;
use utoipa::ToSchema;

/// DTO for `SearchParams`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS))]
#[cfg_attr(feature = "ts_bindings", ts(export))]
pub struct SearchParamsDto {
    /// Number of total iterations to perform.
    pub iterations: usize,
    /// Number of reheating cycles.
    pub reheats: Option<usize>,
    /// Number of threads to use.
    pub threads: Option<usize>,
    /// Optional seed for reproducibility.
    pub seed: Option<u64>,
    /// Whether to include thumb keys in the search.
    pub include_thumbs: bool,
}

impl Validator for SearchParamsDto {
    fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

impl From<model::SearchParams> for SearchParamsDto {
    fn from(val: model::SearchParams) -> Self {
        Self {
            iterations: val.get_search_steps(),
            reheats: Some(val.get_reheats()),
            threads: None,
            seed: val.seed,
            include_thumbs: val.include_thumbs,
        }
    }
}

impl SearchParamsDto {
    /// Gets the number of search steps.
    #[must_use]
    pub fn get_search_steps(&self) -> usize {
        self.iterations
    }
    /// Gets the maximum temperature.
    #[must_use]
    pub fn get_temp_max(&self) -> f32 {
        20.0
    }
    /// Gets the minimum temperature.
    #[must_use]
    pub fn get_temp_min(&self) -> f32 {
        0.005
    }
    /// Gets the search patience (iterations before reheating).
    #[must_use]
    pub fn get_search_patience(&self) -> usize {
        500
    }
    /// Gets the number of reheats.
    #[must_use]
    pub fn get_reheats(&self) -> usize {
        self.reheats.unwrap_or(3)
    }
    /// Gets the reheat factor.
    #[must_use]
    pub fn get_reheat_factor(&self) -> f32 {
        0.5
    }
}

/// DTO for `EngineConfig`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS))]
#[cfg_attr(feature = "ts_bindings", ts(export))]
pub struct EngineConfigDto {
    /// L1 Data cache size.
    pub l1d_size: usize,
    /// L2 cache size.
    pub l2_size: usize,
    /// L3 cache size.
    pub l3_size: usize,
    /// Whether to use SIMD prefetching.
    pub use_prefetch: bool,
}

impl From<model::EngineConfig> for EngineConfigDto {
    fn from(val: model::EngineConfig) -> Self {
        Self {
            l1d_size: val.l1d_size,
            l2_size: val.l2_size,
            l3_size: val.l3_size,
            use_prefetch: val.use_prefetch,
        }
    }
}

/// DTO for `ScoringWeights`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS))]
#[cfg_attr(feature = "ts_bindings", ts(export))]
pub struct ScoringWeightsDto {
    /// Map of penalty keys to weights.
    #[serde(flatten)]
    pub weights: HashMap<String, f32>,
    /// Array of scaling factors for each finger.
    pub finger_penalty_scale: [f32; 5],
    /// String representation of comfortable scissor pairs.
    pub comfortable_scissors: String,
}

impl Validator for ScoringWeightsDto {
    fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

impl From<model::ScoringWeights> for ScoringWeightsDto {
    fn from(val: model::ScoringWeights) -> Self {
        Self {
            weights: val.weights,
            finger_penalty_scale: val.finger_penalty_scale,
            comfortable_scissors: val.comfortable_scissors,
        }
    }
}

/// DTO for `KeyConstraint`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS))]
#[cfg_attr(feature = "ts_bindings", ts(export))]
pub struct KeyConstraintDto {
    /// The physical index of the key.
    pub index: KeyIndexDto,
    /// The label of the key allowed at this position.
    pub key: String,
}

impl From<keyforge_model::config::KeyConstraint> for KeyConstraintDto {
    fn from(val: keyforge_model::config::KeyConstraint) -> Self {
        Self {
            index: val.index.into(),
            key: val.key,
        }
    }
}

/// DTO for `CorpusSource`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS))]
#[cfg_attr(feature = "ts_bindings", ts(export))]
pub struct CorpusSourceDto {
    /// Identifier for the corpus asset.
    pub id: String,
    /// Weight multiplier for this corpus.
    pub weight: f32,
    /// Optional hash for verification.
    pub hash: Option<String>,
}

impl Validator for CorpusSourceDto {
    fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("Corpus ID cannot be empty".into());
        }
        Ok(())
    }
}

impl From<keyforge_model::config::CorpusSource> for CorpusSourceDto {
    fn from(val: keyforge_model::config::CorpusSource) -> Self {
        Self {
            id: val.id,
            weight: val.weight,
            hash: val.hash,
        }
    }
}

/// DTO for `CostMatrixSource`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS))]
#[cfg_attr(feature = "ts_bindings", ts(export))]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum CostMatrixSourceDto {
    /// A predefined model name.
    Predefined(String),
}

impl From<keyforge_model::config::CostMatrixSource> for CostMatrixSourceDto {
    fn from(val: keyforge_model::config::CostMatrixSource) -> Self {
        match val {
            keyforge_model::config::CostMatrixSource::Predefined(s) => Self::Predefined(s),
        }
    }
}

/// DTO for `KeyboardMeta`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS))]
#[cfg_attr(feature = "ts_bindings", ts(export))]
pub struct KeyboardMetaDto {
    /// Display name.
    pub name: String,
    /// Author name.
    pub author: String,
    /// Version string.
    pub version: String,
    /// Design notes.
    pub notes: String,
    /// Keyboard type classification.
    pub kb_type: String,
}

impl From<geometry::KeyboardMeta> for KeyboardMetaDto {
    fn from(val: geometry::KeyboardMeta) -> Self {
        Self {
            name: val.name,
            author: val.author,
            version: val.version,
            notes: val.notes,
            kb_type: val.kb_type,
        }
    }
}



/// DTO for `KeyboardDefinition`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS))]
#[cfg_attr(feature = "ts_bindings", ts(export))]
pub struct KeyboardDefinitionDto {
    /// Metadata.
    pub meta: KeyboardMetaDto,
    /// Geometry.
    pub geometry: KeyboardGeometryDto,
    /// Map of layout names to strings.
    pub layouts: HashMap<String, String>,
}

impl Validator for KeyboardDefinitionDto {
    fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

impl From<geometry::KeyboardDefinition> for KeyboardDefinitionDto {
    fn from(val: geometry::KeyboardDefinition) -> Self {
        Self {
            meta: val.meta.into(),
            geometry: val.geometry.into(),
            layouts: val.layouts,
        }
    }
}

/// DTO for the root `ConfigAggregate`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS))]
#[cfg_attr(feature = "ts_bindings", ts(export))]
pub struct ConfigAggregateDto {
    /// Engine configuration.
    pub engine: EngineConfigDto,
    /// Scoring weights.
    pub weights: ScoringWeightsDto,
}

impl From<keyforge_model::config::Config> for ConfigAggregateDto {
    fn from(val: keyforge_model::config::Config) -> Self {
        Self {
            engine: val.engine.into(),
            weights: val.weights.into(),
        }
    }
}
