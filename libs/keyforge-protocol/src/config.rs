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

impl From<SearchParamsDto> for model::SearchParams {
    #[allow(clippy::cast_precision_loss)]
    fn from(val: SearchParamsDto) -> Self {
        let mut params = std::collections::HashMap::new();
        params.insert("search_steps".to_string(), val.iterations as f32);
        if let Some(r) = val.reheats {
            params.insert("reheats".to_string(), r as f32);
        }

        Self {
            params,
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

impl From<EngineConfigDto> for model::EngineConfig {
    fn from(val: EngineConfigDto) -> Self {
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
        for (key, &val) in &self.weights {
            keyforge_model::types::FixedWeight::from_f32(val)
                .map_err(|e| format!("Weight '{key}' is invalid: {e}"))?;
        }
        for (i, &val) in self.finger_penalty_scale.iter().enumerate() {
            keyforge_model::types::FixedWeight::from_f32(val)
                .map_err(|e| format!("Finger penalty scale #{i} is invalid: {e}"))?;
        }
        Ok(())
    }
}

impl From<model::ScoringWeights> for ScoringWeightsDto {
    fn from(val: model::ScoringWeights) -> Self {
        Self {
            weights: val
                .weights
                .into_iter()
                .map(|(k, v)| (k, v.to_f32()))
                .collect(),
            finger_penalty_scale: val.finger_penalty_scale.map(keyforge_model::Score::to_f32),
            comfortable_scissors: val.comfortable_scissors,
        }
    }
}

impl From<ScoringWeightsDto> for model::ScoringWeights {
    fn from(val: ScoringWeightsDto) -> Self {
        let mut weights_map = HashMap::new();
        for (k, v) in val.weights {
            if let Ok(score) = keyforge_model::types::Score::from_f32(v) {
                weights_map.insert(k, score);
            }
        }

        let mut finger_penalty_scale = [keyforge_model::types::Score::default(); 5];
        for (i, &v) in val.finger_penalty_scale.iter().enumerate() {
            if let Ok(score) = keyforge_model::types::Score::from_f32(v) {
                finger_penalty_scale[i] = score;
            }
        }

        Self {
            weights: weights_map,
            finger_penalty_scale,
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

impl From<KeyConstraintDto> for keyforge_model::config::KeyConstraint {
    fn from(val: KeyConstraintDto) -> Self {
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

impl Default for CorpusSourceDto {
    fn default() -> Self {
        Self {
            id: "en_small.json".to_string(),
            weight: 1.0,
            hash: None,
        }
    }
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

impl From<CorpusSourceDto> for keyforge_model::config::CorpusSource {
    fn from(val: CorpusSourceDto) -> Self {
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
    Predefined {
        /// Identifier.
        id: String,
        /// Optional hash.
        hash: Option<String>,
    },
}

impl Default for CostMatrixSourceDto {
    fn default() -> Self {
        Self::Predefined {
            id: "cost_matrix.json".to_string(),
            hash: None,
        }
    }
}

impl From<keyforge_model::config::CostMatrixSource> for CostMatrixSourceDto {
    fn from(val: keyforge_model::config::CostMatrixSource) -> Self {
        match val {
            keyforge_model::config::CostMatrixSource::Predefined { id, hash } => {
                Self::Predefined { id, hash }
            }
        }
    }
}

impl From<CostMatrixSourceDto> for keyforge_model::config::CostMatrixSource {
    fn from(val: CostMatrixSourceDto) -> Self {
        match val {
            CostMatrixSourceDto::Predefined { id, hash } => Self::Predefined { id, hash },
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

/// DTO for `KeyboardGeometry`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS))]
#[cfg_attr(feature = "ts_bindings", ts(export))]
pub struct KeyboardGeometryDto {
    /// List of key nodes.
    pub keys: Vec<crate::assets::KeyNodeDto>,
    /// Prime slot indices.
    pub prime_slots: Vec<KeyIndexDto>,
    /// Medium slot indices.
    pub med_slots: Vec<KeyIndexDto>,
    /// Low slot indices.
    pub low_slots: Vec<KeyIndexDto>,
    /// Index of the home row.
    pub home_row: i8,
}

impl From<geometry::KeyboardGeometry> for KeyboardGeometryDto {
    fn from(val: geometry::KeyboardGeometry) -> Self {
        Self {
            keys: val.keys().iter().map(|k| k.clone().into()).collect(),
            prime_slots: val.prime_slots().iter().map(|&s| s.into()).collect(),
            med_slots: val.med_slots().iter().map(|&s| s.into()).collect(),
            low_slots: val.low_slots().iter().map(|&s| s.into()).collect(),
            home_row: val.home_row().raw(),
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

impl From<KeyboardDefinitionDto> for geometry::KeyboardDefinition {
    fn from(val: KeyboardDefinitionDto) -> Self {
        Self {
            meta: val.meta.into(),
            geometry: val.geometry.into(),
            layouts: val.layouts,
        }
    }
}

impl From<KeyboardMetaDto> for geometry::KeyboardMeta {
    fn from(val: KeyboardMetaDto) -> Self {
        Self {
            name: val.name,
            author: val.author,
            version: val.version,
            notes: val.notes,
            kb_type: val.kb_type,
        }
    }
}

impl From<KeyboardGeometryDto> for geometry::KeyboardGeometry {
    fn from(val: KeyboardGeometryDto) -> Self {
        let keys = val.keys.into_iter().map(Into::into).collect();
        let prime_slots = val.prime_slots.into_iter().map(Into::into).collect();
        let med_slots = val.med_slots.into_iter().map(Into::into).collect();
        let low_slots = val.low_slots.into_iter().map(Into::into).collect();

        geometry::KeyboardGeometry::new(
            keys,
            prime_slots,
            med_slots,
            low_slots,
            keyforge_model::RowIndex::new(val.home_row),
        )
    }
}

/// DTO for `ProjectMeta`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS))]
#[cfg_attr(feature = "ts_bindings", ts(export))]
pub struct ProjectMetaDto {
    /// The display name of the project.
    pub name: String,
    /// The version string for the project.
    pub version: String,
    /// The author of the project.
    pub author: String,
}

impl Default for ProjectMetaDto {
    fn default() -> Self {
        Self {
            name: "Untitled Project".to_string(),
            version: "0.1.0".to_string(),
            author: "Anonymous".to_string(),
        }
    }
}

impl From<keyforge_model::config::aggregate::ProjectMeta> for ProjectMetaDto {
    fn from(val: keyforge_model::config::aggregate::ProjectMeta) -> Self {
        Self {
            name: val.name,
            version: val.version,
            author: val.author,
        }
    }
}

impl From<ProjectMetaDto> for keyforge_model::config::aggregate::ProjectMeta {
    fn from(val: ProjectMetaDto) -> Self {
        Self {
            name: val.name,
            version: val.version,
            author: val.author,
        }
    }
}

/// DTO for the root `Config` aggregate.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS))]
#[cfg_attr(feature = "ts_bindings", ts(export))]
pub struct ConfigDto {
    /// Metadata about the configuration/project.
    pub meta: ProjectMetaDto,
    /// Name or Path of the keyboard definition.
    pub keyboard: String,
    /// List of corpora to blend.
    pub corpora: Vec<CorpusSourceDto>,
    /// Source for the cost matrix.
    pub cost_matrix: CostMatrixSourceDto,
    /// Optional seed for deterministic reproducibility.
    pub seed: Option<u64>,
    /// Search parameters for the optimization engine.
    pub search: SearchParamsDto,
    /// Hardware-specific engine parameters.
    pub engine: EngineConfigDto,
    /// Weights for the physics scoring engine.
    pub weights: ScoringWeightsDto,
    /// Definitions for layout tiers and critical bigrams.
    pub defs: serde_json::Value,
    /// Keys pinned to specific positions.
    pub pinned_keys: Vec<KeyConstraintDto>,
}

impl Default for ConfigDto {
    fn default() -> Self {
        Self {
            meta: ProjectMetaDto::default(),
            keyboard: "ortho_30".to_string(),
            corpora: vec![CorpusSourceDto::default()],
            cost_matrix: CostMatrixSourceDto::default(),
            seed: None,
            search: SearchParamsDto::from(keyforge_model::config::SearchParams::default()),
            engine: EngineConfigDto::from(keyforge_model::config::EngineConfig::default()),
            weights: ScoringWeightsDto::from(keyforge_model::config::ScoringWeights::default()),
            defs: serde_json::Value::Null,
            pinned_keys: Vec::new(),
        }
    }
}

impl From<keyforge_model::config::Config> for ConfigDto {
    fn from(val: keyforge_model::config::Config) -> Self {
        Self {
            meta: val.meta.into(),
            keyboard: val.keyboard,
            corpora: val.corpora.into_iter().map(Into::into).collect(),
            cost_matrix: val.cost_matrix.into(),
            seed: val.seed,
            search: val.search.into(),
            engine: val.engine.into(),
            weights: val.weights.into(),
            defs: serde_json::Value::Null, // Simplified for now
            pinned_keys: val.pinned_keys.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<ConfigDto> for keyforge_model::config::Config {
    fn from(val: ConfigDto) -> Self {
        Self {
            meta: val.meta.into(),
            keyboard: val.keyboard,
            corpora: val.corpora.into_iter().map(Into::into).collect(),
            cost_matrix: val.cost_matrix.into(),
            seed: val.seed,
            search: val.search.into(),
            engine: val.engine.into(),
            weights: val.weights.into(),
            defs: keyforge_model::config::definitions::LayoutDefinitions::default(), // Simplified
            pinned_keys: val.pinned_keys.into_iter().map(Into::into).collect(),
        }
    }
}

impl keyforge_model::Asset for ConfigDto {
    fn category() -> keyforge_model::AssetCategory {
        keyforge_model::AssetCategory::Keycodes // Mapping root config to config category
    }

    fn post_load(&mut self) -> Result<(), keyforge_model::error::ForgeError> {
        Ok(())
    }
}

impl keyforge_model::Asset for KeyboardDefinitionDto {
    fn category() -> keyforge_model::AssetCategory {
        keyforge_model::AssetCategory::Keyboard
    }

    fn post_load(&mut self) -> Result<(), keyforge_model::error::ForgeError> {
        self.validate()
            .map_err(keyforge_model::error::ForgeError::InvalidData)
    }
}

/// DTO for `ParamType`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS))]
#[cfg_attr(feature = "ts_bindings", ts(export))]
pub enum ParamTypeDto {
    /// Floating point number.
    Float,
    /// Integer number.
    Integer,
    /// Boolean toggle.
    Boolean,
}

impl From<model::ParamType> for ParamTypeDto {
    fn from(val: model::ParamType) -> Self {
        match val {
            model::ParamType::Float => Self::Float,
            model::ParamType::Integer => Self::Integer,
            model::ParamType::Boolean => Self::Boolean,
        }
    }
}

/// DTO for `ParameterMetadata`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS))]
#[cfg_attr(feature = "ts_bindings", ts(export))]
pub struct ParameterMetadataDto {
    /// Internal key name.
    pub key: String,
    /// User-friendly label.
    pub label: String,
    /// Helpful description.
    pub description: String,
    /// Data type.
    pub param_type: ParamTypeDto,
    /// Minimum value.
    pub min: Option<f32>,
    /// Maximum value.
    pub max: Option<f32>,
    /// Default value.
    pub default: f32,
}

impl From<model::ParameterMetadata> for ParameterMetadataDto {
    fn from(val: model::ParameterMetadata) -> Self {
        Self {
            key: val.key,
            label: val.label,
            description: val.description,
            param_type: val.param_type.into(),
            min: val.min,
            max: val.max,
            default: val.default,
        }
    }
}
