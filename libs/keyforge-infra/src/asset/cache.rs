// libs/keyforge-infra/src/asset/cache.rs

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

use std::sync::Arc;
use bytes::Bytes;
use moka::sync::Cache;
use keyforge_model::Corpus;
use keyforge_model::geometry::KeyboardDefinition;
use keyforge_model::cost_model::CostModel;
use keyforge_model::keycodes::KeycodeRegistry;
use crate::net::sync::ServerManifest;
use keyforge_model::constants::{
    DEFAULT_CORPUS_CACHE_CAPACITY, DEFAULT_COST_CACHE_CAPACITY,
    DEFAULT_KB_CACHE_CAPACITY, DEFAULT_KEYCODE_CACHE_CAPACITY,
};

/// specialized cache container for `KeyForge` assets.
#[derive(Debug, Clone)]
pub struct AssetCache {
    keyboards: Cache<String, Arc<KeyboardDefinition>>,
    corpora: Cache<String, Arc<Corpus>>,
    cost_models: Cache<String, Arc<CostModel>>,
    keycodes: Cache<String, Arc<KeycodeRegistry>>,
    file_cache: Cache<String, Bytes>,
    manifest: Cache<String, Arc<ServerManifest>>,
}

impl Default for AssetCache {
    fn default() -> Self {
        Self::new()
    }
}

impl AssetCache {
    /// Creates a new, empty asset cache with default capacities.
    #[must_use] 
    pub fn new() -> Self {
        Self {
            keyboards: Cache::new(DEFAULT_KB_CACHE_CAPACITY as u64),
            corpora: Cache::new(DEFAULT_CORPUS_CACHE_CAPACITY as u64),
            cost_models: Cache::new(DEFAULT_COST_CACHE_CAPACITY as u64),
            keycodes: Cache::new(DEFAULT_KEYCODE_CACHE_CAPACITY as u64),
            file_cache: Cache::new(1000), // Raw binary cache
            manifest: Cache::new(1),
        }
    }

    /// Invalidates all cached entries.
    pub fn invalidate_all(&self) {
        self.keyboards.invalidate_all();
        self.corpora.invalidate_all();
        self.cost_models.invalidate_all();
        self.keycodes.invalidate_all();
        self.file_cache.invalidate_all();
        self.manifest.invalidate_all();
    }

    // -- Typed Accessors --

    #[must_use] 
    pub fn get_keyboard(&self, id: &str) -> Option<Arc<KeyboardDefinition>> {
        self.keyboards.get(id)
    }

    pub fn insert_keyboard(&self, id: String, item: Arc<KeyboardDefinition>) {
        self.keyboards.insert(id, item);
    }

    pub fn invalidate_keyboard(&self, id: &str) {
        self.keyboards.invalidate(id);
    }
    
    pub fn invalidate_all_keyboards(&self) {
        self.keyboards.invalidate_all();
    }

    #[must_use] 
    pub fn get_corpus(&self, key: &str) -> Option<Arc<Corpus>> {
        self.corpora.get(key)
    }

    pub fn insert_corpus(&self, key: String, item: Arc<Corpus>) {
        self.corpora.insert(key, item);
    }
    
    pub fn invalidate_all_corpora(&self) {
        self.corpora.invalidate_all();
    }

    #[must_use] 
    pub fn get_cost_model(&self, id: &str) -> Option<Arc<CostModel>> {
        self.cost_models.get(id)
    }

    pub fn insert_cost_model(&self, id: String, item: Arc<CostModel>) {
        self.cost_models.insert(id, item);
    }

    pub fn invalidate_cost_model(&self, id: &str) {
        self.cost_models.invalidate(id);
    }
    
    pub fn invalidate_all_cost_models(&self) {
        self.cost_models.invalidate_all();
    }

    #[must_use] 
    pub fn get_keycodes(&self, id: &str) -> Option<Arc<KeycodeRegistry>> {
        self.keycodes.get(id)
    }

    pub fn insert_keycodes(&self, id: String, item: Arc<KeycodeRegistry>) {
        self.keycodes.insert(id, item);
    }
    
    pub fn invalidate_all_keycodes(&self) {
        self.keycodes.invalidate_all();
    }

    #[must_use] 
    pub fn get_file(&self, path: &str) -> Option<Bytes> {
        self.file_cache.get(path)
    }

    pub fn insert_file(&self, path: String, content: Bytes) {
        self.file_cache.insert(path, content);
    }

    pub fn invalidate_file(&self, path: &str) {
        self.file_cache.invalidate(path);
    }

    #[must_use] 
    pub fn get_manifest(&self) -> Option<Arc<ServerManifest>> {
        self.manifest.get("default")
    }

    pub fn insert_manifest(&self, item: Arc<ServerManifest>) {
        self.manifest.insert("default".into(), item);
    }
    
    pub fn invalidate_manifest(&self) {
        self.manifest.invalidate_all();
    }
}
