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
use crate::net::sync::ServerManifest;
use crate::util::corpus::inject_synthetic_data;
use keyforge_core::loader::{AssetLoader, LoaderResult};
use keyforge_model::config::CorpusSource;
use keyforge_model::constants::VALKEY_ASSET_PREFIX;
use keyforge_model::error::ForgeError;
use keyforge_model::{Asset, AssetCategory, Corpus};
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
    #[must_use]
    pub fn new(coordinator: Arc<DistributedCoordinator>) -> Self {
        Self { coordinator }
    }

    /// Fetches the current system asset manifest from the distributed store.
    pub async fn get_manifest(&self) -> ServerManifest {
        match self.coordinator.get_all_manifest_entries().await {
            Ok(files) => ServerManifest { files },
            Err(e) => {
                warn!("Failed to fetch manifest from Valkey: {}", e);
                ServerManifest {
                    files: std::collections::HashMap::new(),
                }
            }
        }
    }

    /// Stateless provider: cache invalidation is managed by the distributed store itself.
    pub fn invalidate_all(&self) {}

    /// Retrieves the hash of a corpus from the distributed store.
    /// Retrieves the hash of a corpus from Valkey.
    ///
    /// # Errors
    ///
    /// Returns `InfraError` if the hash cannot be retrieved from the store.
    pub async fn get_corpus_hash(&self, id: &str) -> LoaderResult<String> {
        let key = format!("corpora/{id}/1grams.mpk.zst");
        match self.coordinator.get_manifest_hash(&key).await {
            Ok(Some(h)) => Ok(h),
            _ => Err(ForgeError::NotFound(id.to_string())),
        }
    }

    async fn fetch_blob(&self, subpath: &str) -> LoaderResult<bytes::Bytes> {
        let key = format!("{ASSET_PREFIX}:{subpath}");
        let data = self
            .coordinator
            .get_bin(&key)
            .await
            .map_err(|e| ForgeError::Internal(format!("Valkey Fetch Error: {e}")))?;

        data.ok_or_else(|| ForgeError::NotFound(subpath.to_string()))
    }

    async fn hydrate_mpk<T: serde::de::DeserializeOwned + Send + 'static>(
        &self,
        subpath: &str,
    ) -> LoaderResult<T> {
        let compressed = self.fetch_blob(subpath).await?;

        tokio::task::spawn_blocking(move || {
            let decoder = zstd::Decoder::new(&compressed[..])
                .map_err(|e| ForgeError::Internal(format!("Zstd Init Error: {e}")))?;
            rmp_serde::from_read(decoder)
                .map_err(|e| ForgeError::Internal(format!("Deserialization Error: {e}")))
        })
        .await
        .map_err(|e| ForgeError::Internal(e.to_string()))?
    }

    // --- Helper Methods for Hive ---

    /// Retrieves the raw byte content of a file from the distributed store.
    pub async fn get_file_content(&self, path: &str) -> Option<bytes::Bytes> {
        let key = format!("{ASSET_PREFIX}:{path}");
        self.coordinator.get_bin(&key).await.unwrap_or(None)
    }

    /// Lists all available keyboard definitions in the distributed store.
    pub async fn list_keyboards(&self) -> Vec<String> {
        let pattern = format!("{ASSET_PREFIX}:keyboards/models/*.mpk.zst");
        let keys = self
            .coordinator
            .scan_keys(&pattern)
            .await
            .unwrap_or_default();

        let mut names = Vec::new();
        for k in keys {
            if let Some(stem) = k.split('/').next_back() {
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
        let pattern = format!("{ASSET_PREFIX}:corpora/*");
        let keys = self
            .coordinator
            .scan_keys(&pattern)
            .await
            .unwrap_or_default();

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
        let pattern = format!("{ASSET_PREFIX}:weights/*.mpk.zst");
        let keys = self
            .coordinator
            .scan_keys(&pattern)
            .await
            .unwrap_or_default();
        let mut names = Vec::new();
        for k in keys {
            if let Some(stem) = k.split('/').next_back() {
                if let Some(name) = stem.strip_suffix(".mpk.zst") {
                    names.push(name.to_string());
                }
            }
        }
        names.sort();
        names
    }

    /// Loads a configuration asset from the distributed store, with an optional fallback to JSON if compressed `MsgPack` is missing.
    pub async fn load_config_asset<T: serde::de::DeserializeOwned + Send + 'static + Default>(
        &self,
        name: &str,
    ) -> Arc<T> {
        let mpk_path = format!("config/{name}.mpk.zst");
        if let Ok(cfg) = self.hydrate_mpk::<T>(&mpk_path).await {
            return Arc::new(cfg);
        }
        let json_key = format!("{ASSET_PREFIX}:config/{name}.json");
        if let Ok(Some(bytes)) = self.coordinator.get_bin(&json_key).await {
            if let Ok(cfg) = serde_json::from_slice(&bytes) {
                return Arc::new(cfg);
            }
        }
        Arc::new(T::default())
    }

    pub fn id_to_subpath(category: AssetCategory, id: &str) -> String {
        let stem = id.strip_suffix(".json").unwrap_or(id);
        match category {
            AssetCategory::Keyboard => format!("keyboards/models/{stem}.mpk.zst"),
            AssetCategory::CostModel => format!("weights/{stem}.mpk.zst"),
            AssetCategory::Keycodes => format!("config/{stem}.mpk.zst"),
            AssetCategory::Corpus => format!("corpora/{stem}/bundle.mpk.zst"),
            AssetCategory::Rubric => format!("rubrics/{stem}.mpk.zst"),
        }
    }
}

#[async_trait::async_trait]
impl AssetLoader for ValkeyProvider {
    async fn load<T: Asset>(&self, id: &str) -> LoaderResult<Arc<T>> {
        let category = T::category();
        let subpath = Self::id_to_subpath(category, id);
        let mut asset: T = self.hydrate_mpk(&subpath).await?;
        asset.post_load()?;
        Ok(Arc::new(asset))
    }

    async fn load_corpus(&self, sources: &[CorpusSource]) -> LoaderResult<Arc<Corpus>> {
        let mut corpus = Corpus::default();

        for src in sources {
            let base = format!("corpora/{}", src.id);
            let parts = ["1grams", "2grams", "3grams", "words"];

            let mut segments = Vec::new();

            for part_name in parts {
                let path = format!("{base}/{part_name}.mpk.zst");
                if let Ok(bytes) = self.fetch_blob(&path).await {
                    let part_res = tokio::task::spawn_blocking(move || {
                        let decoder = zstd::Decoder::new(&bytes[..])
                            .map_err(|e| ForgeError::Internal(e.to_string()))?;
                        let data: Vec<serde_json::Value> = rmp_serde::from_read(decoder)
                            .map_err(|e| ForgeError::Internal(e.to_string()))?;
                        Ok::<Vec<serde_json::Value>, ForgeError>(data)
                    })
                    .await
                    .map_err(|e| ForgeError::Internal(e.to_string()))??;

                    segments.push((part_name, part_res));
                }
            }

            crate::util::corpus::populate_corpus_from_segments(&mut corpus, src.weight, segments)?;
        }

        let is_std = sources.iter().any(|s| s.id.contains("_std"));
        inject_synthetic_data(&mut corpus, is_std);

        corpus.post_load()?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use keyforge_model::KeyboardDefinition;

    #[tokio::test]
    async fn test_valkey_provider_mapping() {
        assert_eq!(ValkeyProvider::id_to_subpath(AssetCategory::Keyboard, "kb"), "keyboards/models/kb.mpk.zst");
        assert_eq!(ValkeyProvider::id_to_subpath(AssetCategory::CostModel, "cm"), "weights/cm.mpk.zst");
        assert_eq!(ValkeyProvider::id_to_subpath(AssetCategory::Keycodes, "kc"), "config/kc.mpk.zst");
        assert_eq!(ValkeyProvider::id_to_subpath(AssetCategory::Corpus, "cp"), "corpora/cp/bundle.mpk.zst");
        assert_eq!(ValkeyProvider::id_to_subpath(AssetCategory::Rubric, "rb"), "rubrics/rb.mpk.zst");
        
        // Strip .json
        assert_eq!(ValkeyProvider::id_to_subpath(AssetCategory::Keyboard, "kb.json"), "keyboards/models/kb.mpk.zst");
    }
}
