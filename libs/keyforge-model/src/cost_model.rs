// libs/keyforge-model/src/cost_model.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/ BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Data structures for the external Cost Matrix (Physics Model).
//!
//! This module mirrors the schema of `cost_matrix.json`, allowing the
//! physics engine to load static costs and dynamic rules from data
//! rather than hardcoded logic.

use crate::asset::{Asset, AssetCategory};
use crate::types::RowIndex;
use crate::validator::Validator;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The root structure of the cost matrix file.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CostModel {
    /// Metadata about the model version.
    pub meta: CostModelMeta,
    /// Definitions for different physical layouts (e.g., Row Staggered vs Columnar).
    pub models: HashMap<String, ModelDefinition>,
    /// Global dynamic rules and penalties.
    pub dynamic_rules: DynamicRules,
}

impl Asset for CostModel {
    fn category() -> AssetCategory {
        AssetCategory::CostModel
    }

    fn post_load(&mut self) -> Result<(), crate::error::ForgeError> {
        self.validate()
            .map_err(crate::error::ForgeError::InvalidData)
    }
}

impl Validator for CostModel {
    fn validate(&self) -> Result<(), String> {
        if self.models.is_empty() {
            return Err("CostModel must have at least one model definition".into());
        }
        for (name, model) in &self.models {
            model
                .validate()
                .map_err(|e| format!("Model '{name}': {e}"))?;
        }
        self.dynamic_rules
            .validate()
            .map_err(|e| format!("Dynamic rules: {e}"))?;
        Ok(())
    }
}

/// Metadata for the cost model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostModelMeta {
    /// Schema version.
    pub version: String,
    /// Human-readable description.
    pub description: String,
    /// Unit of measurement (e.g., "Effort Points").
    pub unit: String,
}

impl Default for CostModelMeta {
    fn default() -> Self {
        Self {
            version: "2.0".to_string(),
            description: "Default Cost Model".to_string(),
            unit: "pts".to_string(),
        }
    }
}

/// Definition of a specific physical model (e.g., "`model_a_row_staggered`").
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelDefinition {
    /// Description of the hardware geometry this model applies to.
    pub description: String,
    /// Static costs for key positions, organized by hand and finger.
    pub static_costs: HashMap<String, HandDefinition>,
}

impl Validator for ModelDefinition {
    fn validate(&self) -> Result<(), String> {
        if self.static_costs.is_empty() {
            return Err("Static costs cannot be empty".into());
        }
        Ok(())
    }
}

/// Costs for a specific hand (e.g., "`left_hand`", "`universal_hand`").
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HandDefinition {
    /// Map of finger names to their cost definitions.
    #[serde(flatten)]
    pub fingers: HashMap<String, FingerDefinition>,
}

/// A map of `RowIndex` to cost.
pub type RowCosts = HashMap<RowIndex, f32>;

/// Definition of costs within a finger's reach.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FingerReach {
    /// Costs for keys in the base column.
    #[serde(default)]
    pub base: RowCosts,
    /// Costs for keys in the inner column (closer to center).
    #[serde(default)]
    pub inner: RowCosts,
    /// Costs for keys in the outer column (closer to edge).
    #[serde(default)]
    pub outer: RowCosts,
}

/// Polymorphic definition for finger costs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FingerDefinition {
    /// Standard finger with Reach zones.
    Standard(FingerReach),
    /// Thumb with named positions (backward compatible).
    Thumb(HashMap<String, f32>),
}

/// Dynamic scoring rules and global constraints.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DynamicRules {
    /// Modifiers for key sequences (e.g., rolls).
    pub sequence_modifiers: HashMap<String, f32>,
    /// Penalties for biomechanical violations (e.g., scissors).
    pub penalties: HashMap<String, f32>,
    /// Global constraints (e.g., hand balance).
    pub constraints: HashMap<String, f32>,
}

impl Validator for DynamicRules {
    fn validate(&self) -> Result<(), String> {
        // Basic check to ensure no infinite/NaN values
        for (k, v) in &self.sequence_modifiers {
            if !v.is_finite() {
                return Err(format!("Sequence modifier '{k}' is not finite"));
            }
        }
        Ok(())
    }
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;

    #[test]
    fn test_cost_model_defaults() {
        let meta = CostModelMeta::default();
        assert_eq!(meta.version, "2.0");

        let cm = CostModel::default();
        assert!(cm.models.is_empty());
    }

    #[test]
    fn test_asset_trait() {
        assert_eq!(CostModel::category(), AssetCategory::CostModel);
    }
}
