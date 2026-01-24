// libs/keyforge-adapter/src/loader.rs

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

use keyforge_model::config::CorpusSource;
use keyforge_model::cost_model::CostModel;
use keyforge_model::error::ForgeError;
use keyforge_model::geometry::KeyboardDefinition;
use keyforge_model::keycodes::KeycodeRegistry;
pub use keyforge_model::loader::{AssetLoader, LoaderResult};
use keyforge_model::{Asset, Corpus};
use async_trait::async_trait;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};

/// An in-memory implementation of `AssetLoader`.
///
/// Useful for testing, WASM environments, or scenarios where assets are
/// bundled into the binary.
#[derive(Debug, Default)]
pub struct InMemoryLoader {
    #[allow(clippy::type_complexity)]
    assets: RwLock<HashMap<TypeId, HashMap<String, Arc<dyn Any + Send + Sync>>>>,
}

impl InMemoryLoader {
    /// Creates a new, empty `InMemoryLoader`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Generic injection of an asset into the in-memory loader.
    pub fn inject<T: Asset>(&self, id: &str, asset: T) {
        let tid = TypeId::of::<T>();
        if let Ok(mut maps) = self.assets.write() {
            maps.entry(tid)
                .or_default()
                .insert(id.to_string(), Arc::new(asset));
        }
    }

    /// Injects a keyboard definition into the in-memory loader.
    pub fn inject_keyboard(&self, id: &str, kb: KeyboardDefinition) {
        self.inject(id, kb);
    }

    /// Injects a corpus into the in-memory loader.
    pub fn inject_corpus(&self, id: &str, corpus: Corpus) {
        self.inject(id, corpus);
    }

    /// Injects a cost model into the in-memory loader.
    pub fn inject_cost_model(&self, id: &str, model: CostModel) {
        self.inject(id, model);
    }

    /// Injects a keycode registry into the in-memory loader.
    pub fn inject_keycodes(&self, id: &str, registry: KeycodeRegistry) {
        self.inject(id, registry);
    }
}

#[async_trait]
impl AssetLoader for InMemoryLoader {
    async fn load<T: Asset>(&self, id: &str) -> LoaderResult<Arc<T>> {
        let tid = TypeId::of::<T>();
        let type_name = std::any::type_name::<T>();

        let maps = self
            .assets
            .read()
            .map_err(|e| ForgeError::Internal(format!("Lock poisoned: {e}")))?;

        let map = maps.get(&tid).ok_or_else(|| {
            ForgeError::NotFound(format!("No assets of type '{type_name}' registered"))
        })?;

        let res = map
            .get(id)
            .cloned()
            .ok_or_else(|| ForgeError::NotFound(id.to_string()))?;

        res.downcast::<T>().map_err(|_| {
            ForgeError::Internal(format!(
                "Downcast failed: type mismatch in registry for type '{type_name}'"
            ))
        })
    }

    async fn load_corpus(&self, sources: &[CorpusSource]) -> LoaderResult<Arc<Corpus>> {
        let mut blended = Corpus::default();
        let mut found_any = false;

        for src in sources {
            // Corpora are just Assets, so we can use the generic load mechanism internally
            let loaded = self.load::<Corpus>(&src.id).await;

            if let Ok(corpus) = loaded {
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