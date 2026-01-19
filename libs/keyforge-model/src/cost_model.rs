// libs/keyforge-model/src/cost_model.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Data structures for the external Cost Matrix (Physics Model).
//!
//! This module mirrors the schema of `cost_matrix.json`, allowing the
//! physics engine to load static costs and dynamic rules from data
//! rather than hardcoded logic.

use crate::asset::{Asset, AssetCategory};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The root structure of the cost matrix file.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CostModel {
    /// Metadata about the model version.
    pub meta: Meta,
    /// Definitions for different physical layouts (e.g., Row Staggered vs Columnar).
    pub models: HashMap<String, ModelDefinition>,
    /// Global dynamic rules and penalties.
    pub dynamic_rules: DynamicRules,
}

impl Asset for CostModel {
    fn category() -> AssetCategory {
        AssetCategory::CostModel
    }
}

/// Metadata for the cost model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meta {
    /// Schema version.
    pub version: String,
    /// Human-readable description.
    pub description: String,
    /// Unit of measurement (e.g., "Effort Points").
    pub unit: String,
}

impl Default for Meta {
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

/// Costs for a specific hand (e.g., "`left_hand`", "`universal_hand`").
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HandDefinition {
    /// Map of finger names to their cost definitions.
    #[serde(flatten)]
    pub fingers: HashMap<String, FingerDefinition>,
}

/// Polymorphic definition for finger costs.
///
/// Thumbs typically have a flat list of positions, while other fingers
/// have zones (base, inner, outer) and rows.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FingerDefinition {
    /// Standard finger with zones and rows (e.g., Index -> Base -> r0).
    Standard(HashMap<String, HashMap<String, f32>>),
    /// Thumb with named positions (e.g., Thumb -> `pos_1`).
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
