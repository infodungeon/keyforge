// libs/keyforge-protocol/src/config.rs

use crate::types::KeyIndexDto;
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
    pub iterations: usize,
    pub reheats: Option<usize>,
    pub threads: Option<usize>,
    pub seed: Option<u64>,
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
    #[must_use]
    pub fn get_search_steps(&self) -> usize {
        self.iterations
    }
    #[must_use]
    pub fn get_temp_max(&self) -> f32 {
        20.0
    }
    #[must_use]
    pub fn get_temp_min(&self) -> f32 {
        0.005
    }
    #[must_use]
    pub fn get_search_patience(&self) -> usize {
        500
    }
    #[must_use]
    pub fn get_reheats(&self) -> usize {
        self.reheats.unwrap_or(3)
    }
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
    pub l1d_size: usize,
    pub l2_size: usize,
    pub l3_size: usize,
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
    #[serde(flatten)]
    pub weights: HashMap<String, f32>,
    pub finger_penalty_scale: [f32; 5],
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
    pub index: KeyIndexDto,
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
    pub id: String,
    pub weight: f32,
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
    pub name: String,
    pub author: String,
    pub version: String,
    pub notes: String,
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

/// DTO for `KeyboardGeometry`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS))]
#[cfg_attr(feature = "ts_bindings", ts(export))]
pub struct KeyboardGeometryDto {
    pub keys: Vec<crate::assets::KeyNodeDto>,
    pub prime_slots: Vec<KeyIndexDto>,
    pub med_slots: Vec<KeyIndexDto>,
    pub low_slots: Vec<KeyIndexDto>,
    pub home_row: i8,
}

impl From<geometry::KeyboardGeometry> for KeyboardGeometryDto {
    fn from(val: geometry::KeyboardGeometry) -> Self {
        Self {
            keys: val.keys.into_iter().map(Into::into).collect(),
            prime_slots: val.prime_slots.into_iter().map(Into::into).collect(),
            med_slots: val.med_slots.into_iter().map(Into::into).collect(),
            low_slots: val.low_slots.into_iter().map(Into::into).collect(),
            home_row: val.home_row,
        }
    }
}

/// DTO for `KeyboardDefinition`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS))]
#[cfg_attr(feature = "ts_bindings", ts(export))]
pub struct KeyboardDefinitionDto {
    pub meta: KeyboardMetaDto,
    pub geometry: KeyboardGeometryDto,
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
    pub engine: EngineConfigDto,
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
