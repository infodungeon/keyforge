// libs/keyforge-infra/src/asset/valkey_provider.rs

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


use crate::net::distributed::DistributedCoordinator;
use crate::util::corpus::inject_synthetic_data;
use crate::net::sync::ServerManifest;
use keyforge_core::loader::{AssetLoader, LoaderResult};
use keyforge_model::Corpus;
use keyforge_model::config::CorpusSource;
use keyforge_model::constants::VALKEY_ASSET_PREFIX;
use keyforge_model::geometry::KeyboardDefinition;
use keyforge_model::keycodes::KeycodeRegistry;
use keyforge_model::cost_model::CostModel;
use keyforge_model::Validator;
use keyforge_model::error::ForgeError;
use std::sync::Arc;
use tracing::warn;

const ASSET_PREFIX: &str = VALKEY_ASSET_PREFIX;

/// An asset provider that loads data from a distributed data store (Valkey/Redis).
///
/// This is used by worker nodes in a distributed cluster to fetch assets
/// without requiring direct filesystem access to the Hive's data root.
#[derive(Clone, Debug)]
pub struct ValkeyProvider {
    coordinator: Arc<DistributedCoordinator>,
}

impl ValkeyProvider {
    /// Creates a new `ValkeyProvider` using the provided distributed coordinator.
    pub fn new(coordinator: Arc<DistributedCoordinator>) -> Self {
        Self { coordinator }
    }

    /// Fetches the current system asset manifest from the distributed store.
    pub async fn get_manifest(&self) -> ServerManifest {
        match self.coordinator.get_all_manifest_entries().await {
            Ok(files) => ServerManifest { files },
            Err(e) => {
                warn!("Failed to fetch manifest from Valkey: {}", e);
                ServerManifest { files: std::collections::HashMap::new() }
            }
        }
    }

    /// Stateless provider: cache invalidation is managed by the distributed store itself.
    pub fn invalidate_all(&self) {}

    /// Retrieves the hash of a corpus from the distributed store.
    pub async fn get_corpus_hash(&self, id: &str) -> LoaderResult<String> {
        let key = format!("corpora/{}/1grams.mpk.zst", id);
        match self.coordinator.get_manifest_hash(&key).await {
            Ok(Some(h)) => Ok(h),
            _ => Err(ForgeError::NotFound(id.to_string()))
        }
    }

    async fn fetch_blob(&self, subpath: &str) -> LoaderResult<bytes::Bytes> {
        let key = format!("{}:{}", ASSET_PREFIX, subpath);
        let data = self.coordinator.get_bin(&key).await.map_err(|e| {
            ForgeError::Internal(format!("Valkey Fetch Error: {}", e))
        })?;

        data.ok_or_else(|| {
            ForgeError::NotFound(subpath.to_string())
        })
    }

    async fn hydrate_mpk<T: serde::de::DeserializeOwned + Send + 'static>(&self, subpath: &str) -> LoaderResult<T> {
        let compressed = self.fetch_blob(subpath).await?;
        
        tokio::task::spawn_blocking(move || {
            let decoder = zstd::Decoder::new(&compressed[..])
                .map_err(|e| ForgeError::Internal(format!("Zstd Init Error: {}", e)))?;
            rmp_serde::from_read(decoder)
                .map_err(|e| ForgeError::Internal(format!("Deserialization Error: {}", e)))
        }).await.map_err(|e| ForgeError::Internal(e.to_string()))?
    }

    // --- Helper Methods for Hive ---

    /// Retrieves the raw byte content of a file from the distributed store.
    pub async fn get_file_content(&self, path: &str) -> Option<bytes::Bytes> {
        let key = format!("{}:{}", ASSET_PREFIX, path);
        self.coordinator.get_bin(&key).await.unwrap_or(None)
    }

    /// Lists all available keyboard definitions in the distributed store.
    pub async fn list_keyboards(&self) -> Vec<String> {
        let pattern = format!("{}:keyboards/models/*.mpk.zst", ASSET_PREFIX);
        let keys = self.coordinator.scan_keys(&pattern).await.unwrap_or_default();
        
        let mut names = Vec::new();
        for k in keys {
            if let Some(stem) = k.split('/').last() {
                if let Some(name) = stem.strip_suffix(".mpk.zst") {
                    names.push(name.to_string());
                }
            }
        }
        names.sort();
        names
    }

    /// Lists all available corpora IDs in the distributed store.
    pub async fn list_corpora(&self) -> Vec<String> {
        let pattern = format!("{}:corpora/*", ASSET_PREFIX);
        let keys = self.coordinator.scan_keys(&pattern).await.unwrap_or_default();
        
        let mut ids = Vec::new();
        for k in keys {
            if k.contains("1grams.mpk.zst") {
                if let Some(start) = k.find("corpora/") {
                    let sub = &k[start + 8..]; 
                    if let Some(end) = sub.find("/1grams") {
                        ids.push(sub[..end].to_string());
                    }
                }
            }
        }
        ids.sort();
        ids.dedup();
        ids
    }

    /// Lists all available cost matrices in the distributed store.
    pub async fn list_cost_matrices(&self) -> Vec<String> {
        let pattern = format!("{}:weights/*.mpk.zst", ASSET_PREFIX);
        let keys = self.coordinator.scan_keys(&pattern).await.unwrap_or_default();
        let mut names = Vec::new();
        for k in keys {
            if let Some(stem) = k.split('/').last() {
                if let Some(name) = stem.strip_suffix(".mpk.zst") {
                    names.push(name.to_string());
                }
            }
        }
        names.sort();
        names
    }

    /// Loads a configuration asset from the distributed store, with an optional fallback to JSON if compressed MsgPack is missing.
    pub async fn load_config_asset<T: serde::de::DeserializeOwned + Send + 'static + Default>(&self, name: &str) -> Arc<T> {
        let mpk_path = format!("config/{}.mpk.zst", name);
        if let Ok(cfg) = self.hydrate_mpk::<T>(&mpk_path).await {
            return Arc::new(cfg);
        }
        let json_key = format!("{}:config/{}.json", ASSET_PREFIX, name);
        if let Ok(Some(bytes)) = self.coordinator.get_bin(&json_key).await {
             if let Ok(cfg) = serde_json::from_slice(&bytes) {
                 return Arc::new(cfg);
             }
        }
        Arc::new(T::default())
    }
}

#[async_trait::async_trait]
impl AssetLoader for ValkeyProvider {
    async fn load_keyboard(&self, name: &str) -> LoaderResult<Arc<KeyboardDefinition>> {
        let path = format!("keyboards/models/{}.mpk.zst", name);
        let kb: KeyboardDefinition = self.hydrate_mpk(&path).await?;
        kb.validate().map_err(|e| ForgeError::InvalidData(e))?;
        Ok(Arc::new(kb))
    }

    async fn load_cost_model(&self, filename: &str) -> LoaderResult<Arc<CostModel>> {
        let stem = filename.strip_suffix(".json").unwrap_or(filename);
        let path = format!("weights/{}.mpk.zst", stem);
        let model: CostModel = self.hydrate_mpk(&path).await?;
        Ok(Arc::new(model))
    }

    async fn load_keycodes(&self, filename: &str) -> LoaderResult<Arc<KeycodeRegistry>> {
        let stem = filename.strip_suffix(".json").unwrap_or(filename);
        let path = format!("config/{}.mpk.zst", stem);
        let reg: KeycodeRegistry = self.hydrate_mpk(&path).await?;
        reg.validate().map_err(|e| ForgeError::InvalidData(e))?;
        Ok(Arc::new(reg))
    }

    async fn load_corpus(&self, sources: &[CorpusSource]) -> LoaderResult<Arc<Corpus>> {
        let mut corpus = Corpus::default();
        
        for src in sources {
            let base = format!("corpora/{}", src.id);
            let parts = ["1grams", "2grams", "3grams", "words"];
            
            let mut segments = Vec::new();

            for part_name in parts {
                let path = format!("{}/{}.mpk.zst", base, part_name);
                if let Ok(bytes) = self.fetch_blob(&path).await {
                    let part_res = tokio::task::spawn_blocking(move || {
                        let decoder = zstd::Decoder::new(&bytes[..]).map_err(|e| ForgeError::Internal(e.to_string()))?;
                        let data: Vec<serde_json::Value> = rmp_serde::from_read(decoder).map_err(|e| ForgeError::Internal(e.to_string()))?;
                        Ok::<Vec<serde_json::Value>, ForgeError>(data)
                    }).await.map_err(|e| ForgeError::Internal(e.to_string()))??;
                    
                    segments.push((part_name, part_res));
                }
            }

            crate::util::corpus::populate_corpus_from_segments(&mut corpus, src.weight, segments)?;
        }

        let is_std = sources.iter().any(|s| s.id.contains("_std"));
        inject_synthetic_data(&mut corpus, is_std);

        corpus.validate().map_err(|e| ForgeError::InvalidData(format!("Invalid corpus: {}", e)))?;
        Ok(Arc::new(corpus))
    }
}

#[async_trait::async_trait]
impl crate::asset::AssetServerProvider for ValkeyProvider {
    async fn get_manifest(&self) -> ServerManifest {
        self.get_manifest().await
    }

    async fn get_file_content(&self, path: &str) -> Option<bytes::Bytes> {
        self.get_file_content(path).await
    }
}
