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

pub type LoaderResult<T> = Result<T, ForgeError>;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CostEntry {
    pub from: String,
    pub to: String,
    pub cost: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawCostData {
    pub entries: Vec<CostEntry>,
}

impl RawCostData {
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

#[async_trait::async_trait]
pub trait AssetLoader: Send + Sync {
    async fn load_keyboard(&self, name: &str) -> LoaderResult<KeyboardDefinition>;
    async fn load_corpus(&self, sources: &[CorpusSource]) -> LoaderResult<Corpus>;
    async fn load_cost_matrix(&self, filename: &str) -> LoaderResult<RawCostData>;
    async fn load_keycodes(&self, filename: &str) -> LoaderResult<KeycodeRegistry>;
}
