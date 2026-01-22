// libs/keyforge-compute/src/loader.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You    may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use keyforge_model::config::CorpusSource;
use keyforge_model::cost_model::CostModel;
use keyforge_model::error::ForgeError;
use keyforge_model::geometry::KeyboardDefinition;
use keyforge_model::keycodes::KeycodeRegistry;
use keyforge_model::{Asset, Corpus};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};

/// A specialized result type for asset loading operations.
pub type LoaderResult<T> = Result<T, ForgeError>;

/// A trait for types that can load `KeyForge` assets from an external source.
///
/// This is the primary abstraction for IO, allowing core logic to remain
/// agnostic to the filesystem, network, or embedded storage.
#[async_trait::async_trait]
pub trait AssetLoader: Send + Sync + Debug {
    /// Generic asset loader.
    async fn load<T: Asset>(&self, id: &str) -> LoaderResult<Arc<T>>;

    /// Loads one or more corpora and merges them into a single bundle.
    ///
    /// Corpus is currently special as it often requires merging multiple sources.
    async fn load_corpus(&self, sources: &[CorpusSource]) -> LoaderResult<Arc<Corpus>>;
}

/// An in-memory implementation of `AssetLoader`.
///
/// Useful for testing, WASM environments, or scenarios where assets are
/// bundled into the binary.
#[derive(Debug, Default)]
pub struct InMemoryLoader {
    keyboards: RwLock<HashMap<String, Arc<KeyboardDefinition>>>,
    corpora: RwLock<HashMap<String, Arc<Corpus>>>,
    cost_models: RwLock<HashMap<String, Arc<CostModel>>>,
    keycodes: RwLock<HashMap<String, Arc<KeycodeRegistry>>>,
}

impl InMemoryLoader {
    /// Creates a new, empty `InMemoryLoader`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Injects a keyboard definition into the in-memory loader.
    pub fn inject_keyboard(&self, id: &str, kb: KeyboardDefinition) {
        if let Ok(mut map) = self.keyboards.write() {
            map.insert(id.to_string(), Arc::new(kb));
        }
    }

    /// Injects a corpus into the in-memory loader.
    pub fn inject_corpus(&self, id: &str, corpus: Corpus) {
        if let Ok(mut map) = self.corpora.write() {
            map.insert(id.to_string(), Arc::new(corpus));
        }
    }

    /// Injects a cost model into the in-memory loader.
    pub fn inject_cost_model(&self, id: &str, model: CostModel) {
        if let Ok(mut map) = self.cost_models.write() {
            map.insert(id.to_string(), Arc::new(model));
        }
    }

    /// Injects a keycode registry into the in-memory loader.
    pub fn inject_keycodes(&self, id: &str, registry: KeycodeRegistry) {
        if let Ok(mut map) = self.keycodes.write() {
            map.insert(id.to_string(), Arc::new(registry));
        }
    }
}

#[async_trait::async_trait]
impl AssetLoader for InMemoryLoader {
    async fn load<T: Asset>(&self, id: &str) -> LoaderResult<Arc<T>> {
        let tid = TypeId::of::<T>();

        if tid == TypeId::of::<KeyboardDefinition>() {
            let res = self
                .keyboards
                .read()
                .map_err(|e| ForgeError::Internal(format!("Lock poisoned: {e}")))?
                .get(id)
                .cloned()
                .ok_or_else(|| ForgeError::NotFound(id.to_string()))?;
            let any_res: Arc<dyn Any + Send + Sync> = res;
            return any_res
                .downcast::<T>()
                .map_err(|_| ForgeError::Internal("Downcast failed".into()));
        }

        if tid == TypeId::of::<CostModel>() {
            let res = self
                .cost_models
                .read()
                .map_err(|e| ForgeError::Internal(format!("Lock poisoned: {e}")))?
                .get(id)
                .cloned()
                .ok_or_else(|| ForgeError::NotFound(id.to_string()))?;
            let any_res: Arc<dyn Any + Send + Sync> = res;
            return any_res
                .downcast::<T>()
                .map_err(|_| ForgeError::Internal("Downcast failed".into()));
        }

        if tid == TypeId::of::<KeycodeRegistry>() {
            let res = self
                .keycodes
                .read()
                .map_err(|e| ForgeError::Internal(format!("Lock poisoned: {e}")))?
                .get(id)
                .cloned()
                .ok_or_else(|| ForgeError::NotFound(id.to_string()))?;
            let any_res: Arc<dyn Any + Send + Sync> = res;
            return any_res
                .downcast::<T>()
                .map_err(|_| ForgeError::Internal("Downcast failed".into()));
        }

        Err(ForgeError::NotFound(format!(
            "Asset type not supported in InMemoryLoader: {id}"
        )))
    }

    async fn load_corpus(&self, sources: &[CorpusSource]) -> LoaderResult<Arc<Corpus>> {
        let mut blended = Corpus::default();
        let mut found_any = false;

        for src in sources {
            let loaded = self
                .corpora
                .read()
                .map_err(|e| ForgeError::Internal(format!("Lock poisoned: {e}")))?
                .get(&src.id)
                .cloned();

            if let Some(corpus) = loaded {
                blended.merge(&corpus, src.weight);
                found_any = true;
            } else {
                return Err(ForgeError::NotFound(src.id.clone()));
            }
        }

        if !found_any {
            return Err(ForgeError::NotFound("Empty corpus source list".into()));
        }

        Ok(Arc::new(blended))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use keyforge_model::config::CorpusSource;

    #[tokio::test]
    async fn test_load_corpus_blending() {
        let loader = InMemoryLoader::new();

        // 1. Inject Corpus A
        let mut c1 = Corpus::default();
        c1.char_freqs['a' as usize] = 100;
        loader.inject_corpus("corpus_a", c1);

        // 2. Inject Corpus B
        let mut c2 = Corpus::default();
        c2.char_freqs['a' as usize] = 200;
        loader.inject_corpus("corpus_b", c2);

        // 3. Load with merging (Additive)
        let sources = vec![
            CorpusSource {
                id: "corpus_a".into(),
                weight: 1.0,
                hash: None,
            },
            CorpusSource {
                id: "corpus_b".into(),
                weight: 1.0,
                hash: None,
            },
        ];

        let loaded = loader
            .load_corpus(&sources)
            .await
            .expect("Failed to load corpus");

        // 4. Assert: Should be 100 + 200 = 300
        assert_eq!(
            loaded.char_freqs['a' as usize],
            300,
            "Corpus blending failed: Expected 300, got {}",
            loaded.char_freqs['a' as usize]
        );
    }
}
