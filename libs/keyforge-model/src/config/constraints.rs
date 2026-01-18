// libs/keyforge-model/src/config/constraints.rs

use crate::types::KeyIndex;
use crate::validator::Validator;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use utoipa::ToSchema;
#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

/// Constraint forcing a key to a specific physical index.
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct KeyConstraint {
    /// The physical index of the key.
    pub index: KeyIndex,
    /// The logical key label/ID to pin.
    pub key: String,
}

impl Validator for KeyConstraint {
    fn validate(&self) -> Result<(), String> {
        if self.key.trim().is_empty() {
            return Err(format!("Constraint for index {} has empty key", self.index));
        }
        Ok(())
    }
}

impl FromStr for KeyConstraint {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.trim().is_empty() { return Err("Empty constraint".to_string()); }
        let (idx_str, key_str) = s.split_once(':').ok_or_else(|| format!("invalid format '{}': expected INDEX:KEY", s))?;
        let index_val = idx_str.trim().parse::<u16>().map_err(|_| format!("invalid index '{}': must be 0-65535", idx_str))?;
        let key_clean = key_str.trim();
        if key_clean.is_empty() { return Err(format!("Empty key in constraint '{}'", s)); }
        Ok(KeyConstraint { index: KeyIndex(index_val), key: key_clean.to_string() })
    }
}
