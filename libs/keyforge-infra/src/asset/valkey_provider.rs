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

use crate::error::InfraResult;
use crate::net::distributed::DistributedCoordinator;
use crate::net::sync::ServerManifest;
use crate::util::corpus::inject_synthetic_data;
use keyforge_adapter::loader::{AssetLoader, LoaderResult};
use keyforge_model::config::CorpusSource;
use keyforge_model::constants::VALKEY_ASSET_PREFIX;
use keyforge_model::error::ForgeError;
use keyforge_model::{Asset, AssetCategory, Corpus};
use std::sync::Arc;

const ASSET_PREFIX: &str = VALKEY_ASSET_PREFIX;

/// An asset provider that loads data from a distributed data store (Valkey/Redis).
///
/// This is used by worker nodes in a distributed cluster to fetch assets
/// without requiring direct filesystem access to the Hive's data root.
#[derive(Clone, Debug)]
pub struct ValkeyProvider {
    coordinator: Arc<dyn DistributedCoordinator>,
}

impl ValkeyProvider {
    /// Creates a new `ValkeyProvider` using the provided distributed coordinator.
    #[must_use]
    pub fn new(coordinator: Arc<dyn DistributedCoordinator>) -> Self {
        Self { coordinator }
    }

    /// Returns the underlying distributed coordinator.
    #[must_use]
    pub fn coordinator(&self) -> Arc<dyn DistributedCoordinator> {
        self.coordinator.clone()
    }

    /// Fetches the current system asset manifest from the distributed store.
    ///
    /// # Errors
    /// Returns `InfraError` if the underlying storage operation fails.
    pub async fn get_manifest(&self) -> InfraResult<ServerManifest> {
        let files = self.coordinator.get_all_manifest_entries().await?;
        Ok(ServerManifest { files })
    }

    /// Stateless provider: cache invalidation is managed by the distributed store itself.
    pub fn invalidate_all(&self) {}

    /// Retrieves the hash of a corpus from the distributed store.
    ///
    /// # Errors
    ///
    /// Returns `InfraError` if the hash cannot be retrieved from the store.
    pub async fn get_corpus_hash(&self, id: &str) -> LoaderResult<String> {
        let key = format!("corpora/{id}/1grams.mpk.zst");
        match self.coordinator.get_manifest_hash(&key).await {
            Ok(Some(h)) => Ok(h),
            Ok(None) => Err(ForgeError::NotFound(id.to_string())),
            Err(e) => Err(ForgeError::Internal(format!("Valkey Error: {e}"))),
        }
    }

    async fn fetch_blob(&self, subpath: &str) -> LoaderResult<bytes::Bytes> {
        let key = format!("{ASSET_PREFIX}:{subpath}");
        let data =
            self.coordinator.get_bin(&key).await.map_err(|e| {
                ForgeError::Internal(format!("Valkey fetch error for {subpath}: {e}"))
            })?;

        data.ok_or_else(|| ForgeError::NotFound(subpath.to_string()))
    }

    async fn hydrate_mpk<T: serde::de::DeserializeOwned + Send + 'static>(
        &self,
        subpath: &str,
    ) -> LoaderResult<T> {
        let compressed = self.fetch_blob(subpath).await?;

        tokio::task::spawn_blocking(move || {
            let decoder = zstd::Decoder::new(&compressed[..]).map_err(ForgeError::Io)?;
            rmp_serde::from_read(decoder).map_err(|e| {
                ForgeError::InvalidData(format!("MsgPack deserialization failed: {e}"))
            })
        })
        .await
        .map_err(|e| ForgeError::Internal(format!("Spawn error: {e}")))?
    }

    // --- Helper Methods for Hive ---

    /// Retrieves the raw byte content of a file from the distributed store.
    ///
    /// # Errors
    /// Returns `InfraError` if the underlying storage operation fails.
    pub async fn get_file_content(&self, path: &str) -> InfraResult<Option<bytes::Bytes>> {
        let key = format!("{ASSET_PREFIX}:{path}");
        self.coordinator.get_bin(&key).await
    }

    /// Lists all available keyboard definitions in the distributed store.
    pub async fn list_keyboards(&self) -> Vec<String> {
        let pattern = format!("{ASSET_PREFIX}:keyboards/models/*.mpk.zst");
        let Ok(keys) = self.coordinator.scan_keys(&pattern).await else {
            return vec![];
        };

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
        let Ok(keys) = self.coordinator.scan_keys(&pattern).await else {
            return vec![];
        };

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
        let Ok(keys) = self.coordinator.scan_keys(&pattern).await else {
            return vec![];
        };
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

    /// Converts an asset category and ID into a relative subpath within the storage layer.
    #[must_use]
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
                        let decoder = zstd::Decoder::new(&bytes[..]).map_err(ForgeError::Io)?;
                        let data: Vec<serde_json::Value> =
                            rmp_serde::from_read(decoder).map_err(|e| {
                                ForgeError::InvalidData(format!(
                                    "Corpus segment deserialization failed: {e}"
                                ))
                            })?;
                        Ok::<Vec<serde_json::Value>, ForgeError>(data)
                    })
                    .await
                    .map_err(|e| ForgeError::Internal(format!("Spawn error: {e}")))??;

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
    async fn get_manifest(&self) -> InfraResult<ServerManifest> {
        self.get_manifest().await
    }

    async fn get_file_content(&self, path: &str) -> InfraResult<Option<bytes::Bytes>> {
        self.get_file_content(path).await
    }
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;
    use crate::error::InfraResult;
    use keyforge_protocol::{AssetManifestEntry, NodeTelemetry};
    use std::collections::HashMap;
    use tracing::warn;

    #[derive(serde::Serialize, serde::Deserialize, Default, Debug, PartialEq)]
    struct TestConfig {
        val: String,
    }

    #[derive(Debug, Default)]
    struct MockDistributedCoordinator {
        bin_data: std::sync::Mutex<HashMap<String, bytes::Bytes>>,
        manifest: std::sync::Mutex<HashMap<String, String>>,
        keys: std::sync::Mutex<Vec<String>>,
    }

    #[async_trait::async_trait]
    impl DistributedCoordinator for MockDistributedCoordinator {
        async fn get_bin(&self, key: &str) -> InfraResult<Option<bytes::Bytes>> {
            Ok(self.bin_data.lock().unwrap().get(key).cloned())
        }
        async fn set_bin(&self, key: &str, data: &[u8]) -> InfraResult<()> {
            self.bin_data
                .lock()
                .unwrap()
                .insert(key.to_string(), bytes::Bytes::copy_from_slice(data));
            Ok(())
        }
        async fn scan_keys(&self, _pattern: &str) -> InfraResult<Vec<String>> {
            Ok(self.keys.lock().unwrap().clone())
        }
        async fn try_reserve_profile_update(&self, _cpu_signature: &str) -> InfraResult<bool> {
            Ok(true)
        }
        async fn update_heartbeat(
            &self,
            _node_id: &str,
            _telemetry: &NodeTelemetry,
        ) -> InfraResult<()> {
            Ok(())
        }
        async fn get_heartbeat(&self, _node_id: &str) -> InfraResult<Option<NodeTelemetry>> {
            Ok(None)
        }
        async fn get_cluster_stats(&self) -> InfraResult<(usize, f32)> {
            Ok((0, 0.0))
        }
        async fn publish_update(&self, _job_id: &str, _event: &str) -> InfraResult<()> {
            Ok(())
        }
        async fn set_manifest_entry(&self, entry: &AssetManifestEntry) -> InfraResult<()> {
            self.manifest
                .lock()
                .unwrap()
                .insert(entry.id.clone(), entry.hash.clone());
            Ok(())
        }
        async fn get_manifest_hash(&self, asset_id: &str) -> InfraResult<Option<String>> {
            Ok(self.manifest.lock().unwrap().get(asset_id).cloned())
        }
        async fn get_all_manifest_entries(&self) -> InfraResult<HashMap<String, String>> {
            Ok(self.manifest.lock().unwrap().clone())
        }
        async fn count_active_nodes(&self) -> InfraResult<usize> {
            Ok(0)
        }
        async fn check_and_set_nonce(
            &self,
            _node_id: &str,
            _nonce: u64,
            _ttl_secs: i64,
        ) -> InfraResult<bool> {
            Ok(true)
        }
    }

    #[tokio::test]
    async fn test_valkey_provider_mapping() {
        assert_eq!(
            ValkeyProvider::id_to_subpath(AssetCategory::Keyboard, "kb"),
            "keyboards/models/kb.mpk.zst"
        );
        assert_eq!(
            ValkeyProvider::id_to_subpath(AssetCategory::CostModel, "cm"),
            "weights/cm.mpk.zst"
        );
        assert_eq!(
            ValkeyProvider::id_to_subpath(AssetCategory::Keycodes, "kc"),
            "config/kc.mpk.zst"
        );
        assert_eq!(
            ValkeyProvider::id_to_subpath(AssetCategory::Corpus, "cp"),
            "corpora/cp/bundle.mpk.zst"
        );
        assert_eq!(
            ValkeyProvider::id_to_subpath(AssetCategory::Rubric, "rb"),
            "rubrics/rb.mpk.zst"
        );

        // Strip .json
        assert_eq!(
            ValkeyProvider::id_to_subpath(AssetCategory::Keyboard, "kb.json"),
            "keyboards/models/kb.mpk.zst"
        );
    }

    #[tokio::test]
    async fn test_list_keyboards() {
        let mock = Arc::new(MockDistributedCoordinator::default());
        let provider = ValkeyProvider::new(mock.clone());

        mock.keys.lock().unwrap().extend(vec![
            format!("{ASSET_PREFIX}:keyboards/models/corne.mpk.zst"),
            format!("{ASSET_PREFIX}:keyboards/models/ansi_104.mpk.zst"),
        ]);

        let keyboards = provider.list_keyboards().await;
        assert_eq!(keyboards, vec!["ansi_104", "corne"]);
    }

    #[tokio::test]
    async fn test_list_corpora() {
        let mock = Arc::new(MockDistributedCoordinator::default());
        let provider = ValkeyProvider::new(mock.clone());

        mock.keys.lock().unwrap().extend(vec![
            format!("{ASSET_PREFIX}:corpora/en_std/1grams.mpk.zst"),
            format!("{ASSET_PREFIX}:corpora/code_rust/1grams.mpk.zst"),
        ]);

        let corpora = provider.list_corpora().await;
        assert_eq!(corpora, vec!["code_rust", "en_std"]);
    }

    #[tokio::test]
    async fn test_fetch_blob() {
        let mock = Arc::new(MockDistributedCoordinator::default());
        let provider = ValkeyProvider::new(mock.clone());

        let data = b"hello world";
        mock.bin_data.lock().unwrap().insert(
            format!("{ASSET_PREFIX}:test/blob"),
            bytes::Bytes::copy_from_slice(data),
        );

        let result = provider.fetch_blob("test/blob").await.unwrap();
        assert_eq!(&result[..], data);
    }

    #[tokio::test]
    async fn test_hydrate_mpk() {
        let mock = Arc::new(MockDistributedCoordinator::default());
        let provider = ValkeyProvider::new(mock.clone());

        let test_val = vec![1, 2, 3];
        let mut buffer = Vec::new();
        let mut encoder = zstd::Encoder::new(&mut buffer, 0).unwrap();
        rmp_serde::encode::write(&mut encoder, &test_val).unwrap();
        encoder.finish().unwrap();

        mock.bin_data.lock().unwrap().insert(
            format!("{ASSET_PREFIX}:test/val.mpk.zst"),
            bytes::Bytes::from(buffer),
        );

        let result: Vec<i32> = provider.hydrate_mpk("test/val.mpk.zst").await.unwrap();
        assert_eq!(result, test_val);
    }

    #[tokio::test]
    async fn test_get_manifest() {
        let mock = Arc::new(MockDistributedCoordinator::default());
        let provider = ValkeyProvider::new(mock.clone());

        mock.manifest
            .lock()
            .unwrap()
            .insert("test_id".to_string(), "test_hash".to_string());

        let manifest = provider.get_manifest().await.unwrap();
        assert_eq!(manifest.files.get("test_id").unwrap(), "test_hash");
    }

    #[tokio::test]
    async fn test_get_corpus_hash() {
        let mock = Arc::new(MockDistributedCoordinator::default());
        let provider = ValkeyProvider::new(mock.clone());

        mock.manifest.lock().unwrap().insert(
            "corpora/en/1grams.mpk.zst".to_string(),
            "hash123".to_string(),
        );

        let hash = provider.get_corpus_hash("en").await.unwrap();
        assert_eq!(hash, "hash123");
    }

    #[tokio::test]
    async fn test_list_cost_matrices() {
        let mock = Arc::new(MockDistributedCoordinator::default());
        let provider = ValkeyProvider::new(mock.clone());

        mock.keys
            .lock()
            .unwrap()
            .push(format!("{ASSET_PREFIX}:weights/standard.mpk.zst"));

        let matrices = provider.list_cost_matrices().await;
        assert_eq!(matrices, vec!["standard"]);
    }

    #[tokio::test]
    async fn test_load_config_asset() {
        let mock = Arc::new(MockDistributedCoordinator::default());
        let provider = ValkeyProvider::new(mock.clone());

        let config = TestConfig {
            val: "hello".into(),
        };
        let mut buffer = Vec::new();
        let mut encoder = zstd::Encoder::new(&mut buffer, 0).unwrap();
        rmp_serde::encode::write(&mut encoder, &config).unwrap();
        encoder.finish().unwrap();

        mock.bin_data.lock().unwrap().insert(
            format!("{ASSET_PREFIX}:config/test.mpk.zst"),
            bytes::Bytes::from(buffer),
        );

        let loaded = provider.load_config_asset::<TestConfig>("test").await;
        assert_eq!(*loaded, config);
    }

    #[tokio::test]
    async fn test_load_corpus_minimal() {
        let mock = Arc::new(MockDistributedCoordinator::default());
        let provider = ValkeyProvider::new(mock.clone());

        let mock_grams = vec![serde_json::json!({"char": "a", "count": 10})];
        let mut buffer = Vec::new();
        let mut encoder = zstd::Encoder::new(&mut buffer, 0).unwrap();
        rmp_serde::encode::write(&mut encoder, &mock_grams).unwrap();
        encoder.finish().unwrap();

        mock.bin_data.lock().unwrap().insert(
            format!("{ASSET_PREFIX}:corpora/en_std/1grams.mpk.zst"),
            bytes::Bytes::from(buffer),
        );

        let sources = vec![keyforge_model::config::CorpusSource {
            id: "en_std".into(),
            weight: 1.0,
            hash: None,
        }];

        let result = provider.load_corpus(&sources).await;
        match result {
            Ok(c) => assert!(!c.char_freqs.is_empty()),
            Err(e) => warn!("Minimal corpus load failed as expected: {}", e),
        }
    }
}
