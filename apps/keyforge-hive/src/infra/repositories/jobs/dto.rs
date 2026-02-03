// apps/keyforge-hive/src/infra/repositories/jobs/dto.rs

use keyforge_model::error::ForgeError;
use keyforge_model::geometry::{KeyNode, KeyboardDefinition, KeyboardGeometry, KeyboardMeta};
use keyforge_model::mapping::Projection;
use keyforge_model::types::{ColIndex, FingerIndex, HandIndex, KeyIndex, RowIndex};
use keyforge_model::Asset;
use keyforge_protocol::CorpusSourceDto;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Database-aligned DTO for a Keyboard metadata row.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct HiveKeyboardMetaRow {
    pub name: String,
    pub author: Option<String>,
    pub version: Option<String>,
    pub notes: Option<String>,
    pub kb_type: Option<String>,
    pub home_row: Option<i32>,
}

/// Database-aligned DTO for a Keyboard Key row.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct HiveKeyRow {
    pub idx: i32,
    pub x: f32,
    pub y: f32,
    pub w: Option<f32>,
    pub h: Option<f32>,
    pub hand: i32,
    pub finger: i32,
    pub row_idx: i32,
    pub col_idx: i32,
    pub is_stretch: Option<bool>,
    pub is_prime: Option<bool>,
    pub is_med: Option<bool>,
    pub is_low: Option<bool>,
    pub r: Option<f32>,
}

/// Database-aligned DTO for a Job row.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct HiveJobRow {
    pub id: String,
    pub keyboard_id: i32,
    pub weights_json: serde_json::Value,
    pub params_json: Option<serde_json::Value>,
    pub pinned_keys: String,
    pub corpus_name: String,
    pub cost_matrix: String,
    pub parent_job_id: Option<String>,
}

/// Database-aligned DTO for Job configuration retrieval.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct HiveJobConfigRow {
    pub keyboard_id: i32,
    pub weights_json: serde_json::Value,
    pub corpus_name: String,
    pub cost_matrix: String,
}

/// A projection bundle that combines raw DB rows with fetched components.
#[derive(Debug)]
pub struct HiveJobProjection {
    pub row: HiveJobRow,
    pub definition: KeyboardDefinition,
}

/// Projection bundle for simple config retrieval.
#[derive(Debug)]
pub struct HiveJobConfigProjection {
    pub row: HiveJobConfigRow,
    pub definition: KeyboardDefinition,
}

/// Anti-Corruption Layer: Project domain `KeyboardDefinition` from Hive DB rows.
#[derive(Debug)]
pub struct HiveKeyboardProjection {
    pub meta: HiveKeyboardMetaRow,
    pub keys: Vec<HiveKeyRow>,
}

impl Projection<HiveKeyboardProjection> for KeyboardDefinition {
    fn project(source: HiveKeyboardProjection) -> Result<Self, ForgeError> {
        let meta = KeyboardMeta {
            name: source.meta.name,
            author: source.meta.author.unwrap_or_default(),
            version: source.meta.version.unwrap_or_default(),
            notes: source.meta.notes.unwrap_or_default(),
            kb_type: source.meta.kb_type.unwrap_or_default(),
        };

        let mut keys = Vec::with_capacity(source.keys.len());
        let mut prime_slots = Vec::new();
        let mut med_slots = Vec::new();
        let mut low_slots = Vec::new();

        for row in source.keys {
            let idx = row.idx;
            #[allow(clippy::cast_possible_truncation)]
            let kidx = KeyIndex::new(u16::try_from(idx)
                .map_err(|_| ForgeError::Projection("Key index overflow".into()))?);

            keys.push(KeyNode {
                index: usize::try_from(idx)
                    .map_err(|_| ForgeError::Projection("Key index negative".into()))?,
                label: format!("k{idx}"),
                x: keyforge_model::types::SpatialUnit::from_f32(row.x),
                y: keyforge_model::types::SpatialUnit::from_f32(row.y),
                w: row.w.unwrap_or(1.0),
                h: row.h.unwrap_or(1.0),
                hand: HandIndex::new(u8::try_from(row.hand).unwrap_or(0)),
                finger: FingerIndex::new_unchecked(u8::try_from(row.finger).unwrap_or(0)),
                row: RowIndex::new(i8::try_from(row.row_idx).unwrap_or(0)),
                col: ColIndex::new(i8::try_from(row.col_idx).unwrap_or(0)),
                is_stretch: row.is_stretch.unwrap_or(false),
                r: row.r.unwrap_or(0.0),
                ..Default::default()
            });

            if row.is_prime.unwrap_or(false) {
                prime_slots.push(kidx);
            }
            if row.is_med.unwrap_or(false) {
                med_slots.push(kidx);
            }
            if row.is_low.unwrap_or(false) {
                low_slots.push(kidx);
            }
        }

        let mut def = KeyboardDefinition {
            meta,
            geometry: KeyboardGeometry {
                keys,
                prime_slots,
                med_slots,
                low_slots,
                home_row: RowIndex::new(i8::try_from(source.meta.home_row.unwrap_or(0)).unwrap_or(0)),
            },
            layouts: HashMap::new(),
        };

        def.post_load()?;
        Ok(def)
    }
}

impl Projection<HiveJobProjection> for keyforge_protocol::JobConfig {
    fn project(source: HiveJobProjection) -> Result<Self, ForgeError> {
        let weights = keyforge_model::config::ScoringWeights::project(source.row.weights_json)?;
        let params = keyforge_model::config::SearchParams::project(
            source.row.params_json.unwrap_or_default(),
        )?;

        let pinned_keys: Vec<keyforge_model::config::KeyConstraint> =
            serde_json::from_str(&source.row.pinned_keys).map_err(ForgeError::from)?;
        let cost_matrix: keyforge_model::config::CostMatrixSource =
            serde_json::from_str(&source.row.cost_matrix).map_err(ForgeError::from)?;

        Ok(Self {
            definition: source.definition.into(),
            weights: weights.into(),
            params: params.into(),
            pinned_keys: pinned_keys
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>()
                .into(),
            corpora: vec![CorpusSourceDto {
                id: source.row.corpus_name,
                weight: keyforge_model::constants::DEFAULT_CORPUS_WEIGHT,
                hash: None,
            }]
            .into(),
            cost_matrix: cost_matrix.into(),
            biometrics: vec![].into(),
            parent_job_id: source.row.parent_job_id,
            baseline_score: None,
            parents: vec![].into(),
        })
    }
}

/// Helper for `get_config` projection.
pub type HiveConfigTuple = (
    keyforge_model::geometry::KeyboardGeometry,
    keyforge_model::config::ScoringWeights,
    String,
    keyforge_model::CostMatrixSource,
);

impl Projection<HiveJobConfigProjection> for HiveConfigTuple {
    fn project(source: HiveJobConfigProjection) -> Result<Self, ForgeError> {
        let weights = keyforge_model::config::ScoringWeights::project(source.row.weights_json)?;
        let cost_matrix =
            serde_json::from_str(&source.row.cost_matrix).map_err(ForgeError::from)?;

        Ok((
            source.definition.geometry,
            weights,
            source.row.corpus_name,
            cost_matrix,
        ))
    }
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_job_projection_logic() {
        let row = HiveJobRow {
            id: "test-job".to_string(),
            keyboard_id: 1,
            weights_json: json!({
                "weights": {
                    "monogram_cost": 1.0
                },
                "finger_penalty_scale": [1.0, 1.0, 1.0, 1.0, 1.0],
                "comfortable_scissors": ""
            }),
            params_json: Some(json!({
                "params": {
                    "search_epochs": 10.0,
                    "search_steps": 100.0
                }
            })),
            pinned_keys: "[]".to_string(),
            corpus_name: "en".to_string(),
            cost_matrix: "{\"type\": \"predefined\", \"data\": \"default.json\"}".to_string(),
            parent_job_id: None,
        };
        let definition = KeyboardDefinition::default();
        let projection = HiveJobProjection { row, definition };

        let config = keyforge_protocol::JobConfig::project(projection).unwrap();
        assert_eq!(config.corpora[0].id, "en");
    }
}
