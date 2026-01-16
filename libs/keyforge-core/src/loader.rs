// libs/keyforge-core/src/loader.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use keyforge_model::config::CorpusSource;
use keyforge_model::error::ForgeError;
use keyforge_model::geometry::KeyboardDefinition;
use keyforge_model::keycodes::KeycodeRegistry;
use keyforge_model::Corpus;

/// A specialized result type for asset loading operations.
pub type LoaderResult<T> = Result<T, ForgeError>;

use serde::{Deserialize, Serialize};

/// A single entry in a cost matrix, defining the travel cost between two keys.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CostEntry {
    /// The label of the starting key.
    pub from: String,
    /// The label of the destination key.
    pub to: String,
    /// The physical cost (e.g., distance or effort) of moving between these keys.
    pub cost: f32,
}

/// A collection of travel cost entries, typically loaded from a JSON or CSV file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawCostData {
    /// The list of individual cost mappings.
    pub entries: Vec<CostEntry>,
}

impl RawCostData {
    /// Resolves the raw cost entries against a specific keyboard geometry.
    ///
    /// This uses generic key selectors (e.g., "hand:left finger:index") to apply costs
    /// to all matching keys, ensuring transposability across different layouts.
    pub fn resolve(&self, geo: &keyforge_model::geometry::KeyboardGeometry) -> Vec<(usize, usize, f32)> {
        let mut overrides = Vec::new();
        let mut applied_count = 0;

        for entry in &self.entries {
            // 1. Parse selectors
            let from_criteria = SelectorCriteria::parse(&entry.from);
            let to_criteria = SelectorCriteria::parse(&entry.to);

            // 2. Find matching keys
            let from_indices: Vec<usize> = geo.keys.iter()
                .enumerate()
                .filter(|(_, k)| from_criteria.matches(k, geo))
                .map(|(i, _)| i)
                .collect();

            let to_indices: Vec<usize> = geo.keys.iter()
                .enumerate()
                .filter(|(_, k)| to_criteria.matches(k, geo))
                .map(|(i, _)| i)
                .collect();

            // 3. Apply costs to all combinations
            for &src in &from_indices {
                for &dst in &to_indices {
                    overrides.push((src, dst, entry.cost));
                    applied_count += 1;
                }
            }
        }

        tracing::info!(
            "Applied {} cost matrix rules resulting in {} resolved overrides", 
            self.entries.len(), 
            applied_count
        );
        
        overrides
    }
}

/// Criteria for selecting keys based on their physical attributes.
struct SelectorCriteria {
    hand: Option<keyforge_model::types::HandIndex>,
    finger: Option<keyforge_model::types::FingerIndex>,
    row: Option<keyforge_model::types::RowIndex>,
    col: Option<keyforge_model::types::ColIndex>,
    is_home: Option<bool>,
    is_stretch: Option<bool>,
}

impl SelectorCriteria {
    fn parse(input: &str) -> Self {
        let mut criteria = Self {
            hand: None,
            finger: None,
            row: None,
            col: None,
            is_home: None,
            is_stretch: None,
        };

        for part in input.split_whitespace() {
            if let Some((key, val)) = part.split_once(':') {
                match key.to_lowercase().as_str() {
                    "hand" | "h" => {
                        criteria.hand = match val.to_lowercase().as_str() {
                            "left" | "l" | "0" => Some(keyforge_model::types::HandIndex::LEFT),
                            "right" | "r" | "1" => Some(keyforge_model::types::HandIndex::RIGHT),
                            _ => None,
                        };
                    },
                    "finger" | "f" => {
                        criteria.finger = match val.to_lowercase().as_str() {
                            "thumb" | "t" | "0" => Some(keyforge_model::types::FingerIndex::THUMB),
                            "index" | "i" | "1" => Some(keyforge_model::types::FingerIndex::INDEX),
                            "middle" | "m" | "2" => Some(keyforge_model::types::FingerIndex::MIDDLE),
                            "ring" | "r" | "3" => Some(keyforge_model::types::FingerIndex::RING),
                            "pinky" | "p" | "4" => Some(keyforge_model::types::FingerIndex::PINKY),
                            v => v.parse::<u8>().ok().and_then(|n| n.try_into().ok()),
                        };
                    },
                    "row" | "r" => {
                        if let Ok(n) = val.parse::<i8>() {
                            criteria.row = Some(keyforge_model::types::RowIndex(n));
                        }
                    },
                    "col" | "c" => {
                        if let Ok(n) = val.parse::<i8>() {
                            criteria.col = Some(keyforge_model::types::ColIndex(n));
                        }
                    },
                    "home" => criteria.is_home = val.parse().ok(),
                    "stretch" => criteria.is_stretch = val.parse().ok(),
                    _ => {}
                }
            }
        }
        criteria
    }

    fn matches(&self, key: &keyforge_model::geometry::KeyNode, _geo: &keyforge_model::geometry::KeyboardGeometry) -> bool {
        if let Some(h) = self.hand { if key.hand != h { return false; } }
        if let Some(f) = self.finger { if key.finger != f { return false; } }
        if let Some(r) = self.row { if key.row != r { return false; } }
        if let Some(c) = self.col { if key.col != c { return false; } }
        if let Some(home) = self.is_home { if key.is_home != home { return false; } }
        if let Some(stretch) = self.is_stretch { if key.is_stretch != stretch { return false; } }
        true
    }
}

impl keyforge_model::validator::Validator for RawCostData {
    fn validate(&self) -> Result<(), String> {
        if self.entries.len() > 100_000 {
            return Err("Too many cost entries (limit 100,000)".to_string());
        }
        for entry in &self.entries {
            if entry.from.is_empty() || entry.to.is_empty() {
                return Err("Cost entry labels cannot be empty".to_string());
            }
            if entry.from.len() > 32 || entry.to.len() > 32 {
                return Err("Cost entry label too long".to_string());
            }
            if !entry.cost.is_finite() {
                return Err(format!("Invalid cost (NaN or Inf) found for {} -> {}", entry.from, entry.to));
            }
            if entry.cost < 0.0 {
                return Err(format!("Negative cost found for {} -> {}", entry.from, entry.to));
            }
            if entry.cost > 1000.0 {
                return Err(format!("Cost too high for {} -> {} (limit 1000.0)", entry.from, entry.to));
            }
        }
        Ok(())
    }
}

use std::sync::Arc;

/// A trait for types that can load KeyForge assets from an external source.
///
/// This is the primary abstraction for IO, allowing core logic to remain 
/// agnostic to the filesystem, network, or embedded storage.
#[async_trait::async_trait]
pub trait AssetLoader: Send + Sync {
    /// Loads a keyboard definition by name.
    async fn load_keyboard(&self, name: &str) -> LoaderResult<Arc<KeyboardDefinition>>;
    /// Loads one or more corpora and merges them into a single bundle.
    async fn load_corpus(&self, sources: &[CorpusSource]) -> LoaderResult<Arc<Corpus>>;
    /// Loads a cost matrix from the specified file.
    async fn load_cost_matrix(&self, filename: &str) -> LoaderResult<Arc<RawCostData>>;
    /// Loads a keycode registry for mapping between labels and codes.
    async fn load_keycodes(&self, filename: &str) -> LoaderResult<Arc<KeycodeRegistry>>;
}
