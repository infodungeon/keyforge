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
    /// This translates label-based lookups into high-performance index-based lookups.
    pub fn resolve(&self, geo: &keyforge_model::geometry::KeyboardGeometry) -> Vec<(usize, usize, f32)> {
        let mut overrides = Vec::new();
        let mut id_map = std::collections::HashMap::new();
        for (i, k) in geo.keys.iter().enumerate() {
            id_map.insert(k.label.clone(), i);
        }
        for entry in &self.entries {
            if let (Some(&idx1), Some(&idx2)) = (id_map.get(&entry.from), id_map.get(&entry.to)) {
                overrides.push((idx1, idx2, entry.cost));
            }
        }
        overrides
    }
}

impl keyforge_model::validator::Validator for RawCostData {
    fn validate(&self) -> Result<(), String> {
        for entry in &self.entries {
            if entry.from.is_empty() || entry.to.is_empty() {
                return Err("Cost entry labels cannot be empty".to_string());
            }
            if entry.cost < 0.0 {
                return Err(format!("Negative cost found for {} -> {}", entry.from, entry.to));
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
