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

use keyforge_core::loader::{AssetLoader, LoaderResult};
use keyforge_model::config::CorpusSource;
use keyforge_model::cost_model::CostModel;
use keyforge_model::error::ForgeError;
use keyforge_model::geometry::KeyboardDefinition;
use keyforge_model::keycodes::KeycodeRegistry;
use keyforge_model::{Asset, Corpus};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// An in-memory asset loader for WASM environments.
///
/// Since browsers cannot access the filesystem, assets must be injected
/// into this loader from JavaScript before the engine is initialized.
#[derive(Debug, Default)]
pub struct InMemoryLoader {
    keyboards: RwLock<HashMap<String, Arc<KeyboardDefinition>>>,
    corpora: RwLock<HashMap<String, Arc<Corpus>>>,
    cost_models: RwLock<HashMap<String, Arc<CostModel>>>,
    keycodes: RwLock<HashMap<String, Arc<KeycodeRegistry>>>,
}

impl InMemoryLoader {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(clippy::unwrap_used)]
    pub fn inject_keyboard(&self, name: String, kb: KeyboardDefinition) {
        self.keyboards.write().unwrap().insert(name, Arc::new(kb));
    }

    #[allow(clippy::unwrap_used)]
    pub fn inject_corpus(&self, name: String, corpus: Corpus) {
        self.corpora.write().unwrap().insert(name, Arc::new(corpus));
    }

    #[allow(clippy::unwrap_used)]
    pub fn inject_cost_model(&self, name: String, model: CostModel) {
        self.cost_models
            .write()
            .unwrap()
            .insert(name, Arc::new(model));
    }

    #[allow(clippy::unwrap_used)]
    pub fn inject_keycodes(&self, name: String, registry: KeycodeRegistry) {
        self.keycodes
            .write()
            .unwrap()
            .insert(name, Arc::new(registry));
    }
}

#[async_trait::async_trait]
impl AssetLoader for InMemoryLoader {
    async fn load<T: Asset>(&self, id: &str) -> LoaderResult<Arc<T>> {
        let tid = TypeId::of::<T>();

        if tid == TypeId::of::<KeyboardDefinition>() {
            let kb = self
                .keyboards
                .read()
                .map_err(|e| ForgeError::Internal(format!("RwLock poisoned: {e}")))?
                .get(id)
                .cloned()
                .ok_or_else(|| ForgeError::NotFound(id.to_string()))?;
            let any_kb: Arc<dyn Any + Send + Sync> = kb;
            return any_kb
                .downcast::<T>()
                .map_err(|_| ForgeError::Internal("Downcast failed".into()));
        }

        if tid == TypeId::of::<CostModel>() {
            let cm = self
                .cost_models
                .read()
                .map_err(|e| ForgeError::Internal(format!("RwLock poisoned: {e}")))?
                .get(id)
                .cloned()
                .ok_or_else(|| ForgeError::NotFound(id.to_string()))?;
            let any_cm: Arc<dyn Any + Send + Sync> = cm;
            return any_cm
                .downcast::<T>()
                .map_err(|_| ForgeError::Internal("Downcast failed".into()));
        }

        if tid == TypeId::of::<KeycodeRegistry>() {
            let rg = self
                .keycodes
                .read()
                .map_err(|e| ForgeError::Internal(format!("RwLock poisoned: {e}")))?
                .get(id)
                .cloned()
                .ok_or_else(|| ForgeError::NotFound(id.to_string()))?;
            let any_rg: Arc<dyn Any + Send + Sync> = rg;
            return any_rg
                .downcast::<T>()
                .map_err(|_| ForgeError::Internal("Downcast failed".into()));
        }

        Err(ForgeError::NotFound(format!(
            "Asset type not supported in WASM loader: {id}"
        )))
    }

    async fn load_corpus(&self, sources: &[CorpusSource]) -> LoaderResult<Arc<Corpus>> {
        // For WASM, we assume the corpus is pre-merged or we just load the first one by ID.
        // Real merging logic is heavy and usually done server-side.
        if let Some(src) = sources.first() {
            self.corpora
                .read()
                .map_err(|e| ForgeError::Internal(format!("RwLock poisoned: {e}")))?
                .get(&src.id)
                .cloned()
                .ok_or_else(|| ForgeError::NotFound(src.id.clone()))
        } else {
            Err(ForgeError::Config("No corpus sources provided".into()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_loader_lifecycle() {
        let loader = InMemoryLoader::new();

        // 1. Keyboard
        loader.inject_keyboard("k1".into(), KeyboardDefinition::default());
        let kb = loader.load::<KeyboardDefinition>("k1").await.unwrap();
        assert_eq!(kb.meta.name, "");

        // 2. Cost Model
        loader.inject_cost_model("c1".into(), CostModel::default());
        let cm = loader.load::<CostModel>("c1").await.unwrap();
        assert_eq!(cm.meta.version, "2.0");

        // 3. Keycodes
        loader.inject_keycodes("r1".into(), KeycodeRegistry::default());
        let rg = loader.load::<KeycodeRegistry>("r1").await.unwrap();
        assert!(rg.definitions.is_empty());

        // 4. Corpus
        loader.inject_corpus("en".into(), Corpus::default());
        let sources = vec![CorpusSource {
            id: "en".into(),
            weight: 1.0,
            hash: None,
        }];
        let corp = loader.load_corpus(&sources).await.unwrap();
        assert_eq!(corp.char_freqs[0], 0);
    }

    #[tokio::test]
    async fn test_in_memory_loader_errors() {
        let loader = InMemoryLoader::new();

        // Missing ID
        assert!(loader.load::<KeyboardDefinition>("missing").await.is_err());

        // No corpus sources
        assert!(loader.load_corpus(&[]).await.is_err());

        // Poisoned lock - Hard to trigger in unit test without unsafe or intentional panic in a thread,
        // but we've covered the code paths by reading the source.
    }
}
