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
use crate::validator::Validator;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Validated and performance-optimized Cost Model (Domain Model).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostModel {
    /// Metadata about the model version.
    pub version: String,
    /// Human-readable description.
    pub description: String,
    /// Unit of measurement (e.g., "Effort Points").
    pub unit: String,
    /// Definitions for different physical layouts (e.g., Row Staggered vs Columnar).
    pub models: HashMap<String, ModelDefinition>,
    /// Global dynamic rules and penalties.
    pub dynamic_rules: DynamicRules,
}

impl Default for CostModel {
    fn default() -> Self {
        Self {
            version: "2.0".to_string(),
            description: "Default Cost Model".to_string(),
            unit: "pts".to_string(),
            models: HashMap::default(),
            dynamic_rules: DynamicRules::default(),
        }
    }
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

impl CostModel {
    /// Returns a specific model definition by name.
    #[must_use]
    pub fn get_model(&self, name: &str) -> Option<&ModelDefinition> {
        self.models.get(name)
    }

    /// Returns the dynamic rules.
    #[must_use]
    pub fn dynamic_rules(&self) -> &DynamicRules {
        &self.dynamic_rules
    }

    /// Returns the map of models.
    #[must_use]
    pub fn models(&self) -> &HashMap<String, ModelDefinition> {
        &self.models
    }

    /// Helper to get a model key (picking the only one or a preferred one).
    #[must_use]
    pub fn preferred_model_key(&self) -> Option<&str> {
        if self.models.len() == 1 {
            self.models.keys().next().map(String::as_str)
        } else if self.models.contains_key("model_a_row_staggered") {
            Some("model_a_row_staggered")
        } else {
            self.models.keys().next().map(String::as_str)
        }
    }

    /// Baked cost lookup for performance-critical loops.
    /// Maps [Hand][Finger][Zone][Row] to f32.
    #[must_use]
    pub fn bake(&self, model_name: &str) -> Option<BakedModel> {
        let model = self.get_model(model_name)?;
        let mut baked = BakedModel::default();

        for (hand_name, hand_def) in &model.static_costs {
            let h_idx = match hand_name.as_str() {
                "right_hand" => 1,
                "left_hand" | "universal_hand" => 0, // Fallback
                _ => continue,
            };

            for (finger_name, finger_def) in &hand_def.fingers {
                let f_idx = match finger_name.as_str() {
                    "thumb" => 0,
                    "index" => 1,
                    "middle" => 2,
                    "ring" => 3,
                    "pinky" => 4,
                    _ => continue,
                };

                match finger_def {
                    FingerDefinition::Standard(reach) => {
                        Self::fill_reach(&mut baked.costs[h_idx][f_idx][0], &reach.base);
                        Self::fill_reach(&mut baked.costs[h_idx][f_idx][1], &reach.inner);
                        Self::fill_reach(&mut baked.costs[h_idx][f_idx][2], &reach.outer);
                    }
                    FingerDefinition::Thumb(map) => {
                        // Map named thumb positions to rows (heuristic for now)
                        for (pos, &cost) in map {
                            let r_idx = match pos.as_str() {
                                "pos_1" => 0,
                                "pos_2" => 1,
                                "pos_3" => 2,
                                _ => continue,
                            };
                            baked.costs[h_idx][f_idx][0][r_idx] = cost;
                        }
                    }
                }
            }

            // If universal, clone to right hand
            if hand_name == "universal_hand" {
                baked.costs[1] = baked.costs[0];
            }
        }
        Some(baked)
    }

    fn fill_reach(target: &mut [f32; 8], source: &RowCosts) {
        for (row, &cost) in source {
            if let Ok(r_idx) = usize::try_from(row.0) {
                if r_idx < 8 {
                    target[r_idx] = cost;
                }
            }
        }
    }
}

/// Performance-optimized baked model for O(1) lookups.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BakedModel {
    /// 4D array: [Hand index: 2][Finger index: 5][Zone index: 3][Row index: 8]
    pub costs: [[[[f32; 8]; 3]; 5]; 2],
}

impl Default for BakedModel {
    fn default() -> Self {
        Self {
            costs: [[[[100.0; 8]; 3]; 5]; 2], // High default cost
        }
    }
}

impl BakedModel {
    /// Returns the cost for a specific finger at a position.
    #[must_use]
    pub fn get_cost(&self, hand: usize, finger: usize, zone: usize, row: usize) -> f32 {
        if hand < 2 && finger < 5 && zone < 3 && row < 8 {
            self.costs[hand][finger][zone][row]
        } else {
            100.0 // Penalty for out of bounds
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
pub type RowCosts = HashMap<crate::types::RowIndex, f32>;

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

/// Wrapper for sequence-based scoring modifiers.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct SequenceModifiers {
    /// Mapping of sequence strings (e.g. "TH") to their cost modifiers.
    pub map: HashMap<String, f32>,
}

/// Dynamic scoring rules and global constraints.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DynamicRules {
    /// Modifiers for key sequences (e.g., rolls).
    pub sequence_modifiers: SequenceModifiers,
    /// Penalties for biomechanical violations (e.g., scissors).
    pub penalties: HashMap<String, f32>,
    /// Global constraints (e.g., hand balance).
    pub constraints: HashMap<String, f32>,
}

impl Validator for DynamicRules {
    fn validate(&self) -> Result<(), String> {
        // Basic check to ensure no infinite/NaN values
        for (k, v) in &self.sequence_modifiers.map {
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
        let cm = CostModel::default();
        assert_eq!(cm.version, "2.0");
        assert!(cm.models().is_empty());
    }

    #[test]
    fn test_asset_trait() {
        assert_eq!(CostModel::category(), AssetCategory::CostModel);
    }
}
