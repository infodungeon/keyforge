// apps/keyforge-hive/src/infra/repositories/jobs/dto.rs

use keyforge_model::error::ForgeError;
use keyforge_model::geometry::{KeyNode, KeyboardDefinition, KeyboardGeometry, KeyboardMeta};
use keyforge_model::mapping::Projection;
use keyforge_model::types::{ColIndex, FingerIndex, HandIndex, KeyIndex, RowIndex};
use keyforge_model::Asset;
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
            let kidx = KeyIndex(
                u16::try_from(idx)
                    .map_err(|_| ForgeError::Projection("Key index overflow".into()))?,
            );

            keys.push(KeyNode {
                index: usize::try_from(idx)
                    .map_err(|_| ForgeError::Projection("Key index negative".into()))?,
                label: format!("k{idx}"),
                x: row.x,
                y: row.y,
                w: row.w.unwrap_or(1.0),
                h: row.h.unwrap_or(1.0),
                hand: HandIndex(u8::try_from(row.hand).unwrap_or(0)),
                finger: FingerIndex::new_unchecked(u8::try_from(row.finger).unwrap_or(0)),
                row: RowIndex(i8::try_from(row.row_idx).unwrap_or(0)),
                col: ColIndex(i8::try_from(row.col_idx).unwrap_or(0)),
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
                home_row: i8::try_from(source.meta.home_row.unwrap_or(0)).unwrap_or(0),
            },
            layouts: HashMap::new(),
        };

        def.post_load()?;
        Ok(def)
    }
}
