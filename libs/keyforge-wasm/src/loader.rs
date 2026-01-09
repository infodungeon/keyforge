// libs/keyforge-wasm/src/loader.rs

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

use keyforge_model::error::ForgeError;
use keyforge_core::loader::{AssetLoader, LoaderResult, RawCostData};
use keyforge_model::Corpus;

// Actually we need model types.
use keyforge_model::config::CorpusSource;
use keyforge_model::geometry::KeyboardDefinition;
use keyforge_model::keycodes::KeycodeRegistry;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Clone, Default)]
pub struct InMemoryLoader {
    keyboards: Arc<RwLock<HashMap<String, Arc<KeyboardDefinition>>>>,
    corpora: Arc<RwLock<HashMap<String, Arc<Corpus>>>>,
    costs: Arc<RwLock<HashMap<String, Arc<RawCostData>>>>,
    keycodes: Arc<RwLock<Arc<KeycodeRegistry>>>,
}

impl InMemoryLoader {
    pub fn add_keyboard(&self, name: String, def: KeyboardDefinition) -> LoaderResult<()> {
        self.keyboards.write().map_err(|e| ForgeError::Internal(format!("RwLock poisoned: {}", e)))?.insert(name, Arc::new(def));
        Ok(())
    }

    pub fn add_corpus(&self, name: String, corpus: Corpus) -> LoaderResult<()> {
        self.corpora.write().map_err(|e| ForgeError::Internal(format!("RwLock poisoned: {}", e)))?.insert(name, Arc::new(corpus));
        Ok(())
    }

    pub fn add_cost(&self, name: String, cost: RawCostData) -> LoaderResult<()> {
        self.costs.write().map_err(|e| ForgeError::Internal(format!("RwLock poisoned: {}", e)))?.insert(name, Arc::new(cost));
        Ok(())
    }

    pub fn set_keycodes(&self, registry: KeycodeRegistry) -> LoaderResult<()> {
        *self.keycodes.write().map_err(|e| ForgeError::Internal(format!("RwLock poisoned: {}", e)))? = Arc::new(registry);
        Ok(())
    }
}

#[async_trait::async_trait]
impl AssetLoader for InMemoryLoader {
    async fn load_keyboard(&self, name: &str) -> LoaderResult<Arc<KeyboardDefinition>> {
        self.keyboards
            .read()
            .map_err(|e| ForgeError::Internal(format!("RwLock poisoned: {}", e)))?
            .get(name)
            .cloned()
            .ok_or_else(|| {
                ForgeError::NotFound(format!("Keyboard '{}' not found in memory", name))
            })
    }

    async fn load_corpus(&self, sources: &[CorpusSource]) -> LoaderResult<Arc<Corpus>> {
        let mut merged = Corpus::default();
        let corpora = self.corpora.read().map_err(|e| ForgeError::Internal(format!("RwLock poisoned: {}", e)))?;

        for source in sources {
            if let Some(corpus) = corpora.get(&source.id) {
                merged.merge(corpus, source.weight);
            } else {
                return Err(ForgeError::NotFound(format!("Corpus '{}' not found in memory", source.id)));
            }
        }

        Ok(Arc::new(merged))
    }

    async fn load_cost_matrix(&self, filename: &str) -> LoaderResult<Arc<RawCostData>> {
        self.costs
            .read()
            .map_err(|e| ForgeError::Internal(format!("RwLock poisoned: {}", e)))?
            .get(filename)
            .cloned()
            .ok_or_else(|| {
                ForgeError::NotFound(format!("Cost matrix '{}' not found in memory", filename))
            })
    }

    async fn load_keycodes(&self, _filename: &str) -> LoaderResult<Arc<KeycodeRegistry>> {
        Ok(self.keycodes.read().map_err(|e| ForgeError::Internal(format!("RwLock poisoned: {}", e)))?.clone())
    }
}
