// libs/keyforge-infra/src/asset/valkey_provider.rs

use crate::error::InfraResult;
use crate::net::distributed::DistributedCoordinator;
use crate::net::sync::ServerManifest;
use crate::util::corpus::inject_synthetic_data;
use async_trait::async_trait;
use keyforge_adapter::loader::{AssetLoader, LoaderResult};
use keyforge_boundary::SafePath;
use keyforge_model::config::CorpusSource;
use keyforge_model::constants::VALKEY_ASSET_PREFIX;
use keyforge_model::error::ForgeError;
use keyforge_model::{Asset, AssetCategory, Corpus};
use serde::de::DeserializeOwned;
use sha2::Digest;
use std::sync::Arc;

const ASSET_PREFIX: &str = VALKEY_ASSET_PREFIX;

/// An asset provider that loads data from a distributed data store (Valkey/Redis).
#[derive(Clone, Debug)]
pub struct ValkeyProvider {
    coordinator: Arc<dyn DistributedCoordinator>,
    root: SafePath,
}

impl ValkeyProvider {
    /// Creates a new `ValkeyProvider` using the provided distributed coordinator.
    #[must_use]
    #[allow(clippy::expect_used, clippy::missing_panics_doc)]
    pub fn new(coordinator: Arc<dyn DistributedCoordinator>) -> Self {
        Self {
            coordinator,
            root: SafePath::from_trusted_root_path(std::path::PathBuf::from(".")),
        }
    }

    /// Returns the underlying distributed coordinator.
    #[must_use]
    pub fn get_coordinator(&self) -> Arc<dyn DistributedCoordinator> {
        self.coordinator.clone()
    }

    /// Fetches the hash of a corpus from the distributed store.
    ///
    /// # Errors
    ///
    /// Returns `ForgeError::NotFound` if the hash is missing or `ForgeError::Internal` on store failure.
    pub async fn get_corpus_hash(&self, id: &str) -> LoaderResult<String> {
        let key = format!("corpora/{id}/1grams.mpk.zst");
        match self.coordinator.get_manifest_hash(&key).await {
            Ok(Some(h)) => Ok(h),
            _ => Err(ForgeError::NotFound(id.to_string())),
        }
    }

    async fn fetch_blob(&self, subpath: &str) -> LoaderResult<bytes::Bytes> {
        let key = format!("{ASSET_PREFIX}:{subpath}");
        self.coordinator
            .get_bin(&key)
            .await
            .map_err(|e| ForgeError::Internal(format!("Valkey fetch error: {e}")))?
            .ok_or_else(|| ForgeError::NotFound(subpath.to_string()))
    }

    /// Hydrates a MsgPack-encoded asset from the distributed store.
    ///
    /// # Errors
    ///
    /// Returns `ForgeError` if fetch or deserialization fails.
    async fn hydrate_mpk<T: serde::de::DeserializeOwned + Send + 'static>(
        &self,
        subpath: &str,
    ) -> LoaderResult<(T, [u8; 32])> {
        let compressed = self.fetch_blob(subpath).await?;
<<<<<<< HEAD
        tokio::task::spawn_blocking(move || {
            let decoder = zstd::Decoder::new(&compressed[..]).map_err(ForgeError::from)?;
            rmp_serde::from_read(decoder).map_err(|e| ForgeError::InvalidData(e.to_string()))
=======

        let (asset, hash) = tokio::task::spawn_blocking(move || -> LoaderResult<(T, [u8; 32])> {
            let mut decoder =
                zstd::Decoder::new(&compressed[..]).map_err(|e| ForgeError::Io(e.to_string()))?;
            let mut decompressed = Vec::new();
            std::io::copy(&mut decoder, &mut decompressed)
                .map_err(|e| ForgeError::Io(e.to_string()))?;

            let mut hasher = sha2::Sha256::new();
            sha2::Digest::update(&mut hasher, &decompressed);
            let hash = hasher.finalize().into();

            let asset = rmp_serde::from_read(&decompressed[..])
                .map_err(|e| ForgeError::InvalidData(e.to_string()))?;
            Ok((asset, hash))
>>>>>>> master
        })
        .await
        .map_err(|e| ForgeError::Internal(e.to_string()))??;

        Ok((asset, hash))
    }

    /// Invalidates all local caches (no-op for stateless distributed provider).
    pub fn invalidate_all(&self) {}

    /// Lists all available keyboard definitions in the distributed store.
    pub async fn list_keyboards(&self) -> Vec<String> {
        let pattern = format!("{ASSET_PREFIX}:keyboards/models/*.mpk.zst");
        self.coordinator
            .scan_keys(&pattern)
            .await
            .unwrap_or_default()
            .into_iter()
            .filter_map(|k| {
                k.split('/')
                    .next_back()?
                    .strip_suffix(".mpk.zst")
                    .map(String::from)
            })
            .collect()
    }

    /// Lists all available corpora IDs in the distributed store.
    pub async fn list_corpora(&self) -> Vec<String> {
        let pattern = format!("{ASSET_PREFIX}:corpora/*");
        self.coordinator
            .scan_keys(&pattern)
            .await
            .unwrap_or_default()
            .into_iter()
            .filter(|k| k.contains("1grams.mpk.zst"))
            .filter_map(|k| k.split('/').nth_back(1).map(String::from))
            .collect()
    }

    /// Lists all available cost matrices in the distributed store.
    pub async fn list_cost_matrices(&self) -> Vec<String> {
        let pattern = format!("{ASSET_PREFIX}:weights/*.mpk.zst");
        self.coordinator
            .scan_keys(&pattern)
            .await
            .unwrap_or_default()
            .into_iter()
            .filter_map(|k| {
                k.split('/')
                    .next_back()?
                    .strip_suffix(".mpk.zst")
                    .map(String::from)
            })
            .collect()
    }

    /// Loads a configuration asset from the distributed store.
    pub async fn load_config_asset<T: serde::de::DeserializeOwned + Send + 'static + Default>(
        &self,
        name: &str,
    ) -> Arc<T> {
        let mpk_path = format!("config/{name}.mpk.zst");
        if let Ok((cfg, _hash)) = self.hydrate_mpk::<T>(&mpk_path).await {
            return Arc::new(cfg);
        }
        Arc::new(T::default())
    }

    /// Maps an asset category and ID to its internal distributed storage path.
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

#[async_trait]
impl AssetLoader for ValkeyProvider {
    async fn load<T: Asset + DeserializeOwned>(&self, id: &str) -> LoaderResult<Arc<T>> {
        let category = T::category();
        let subpath = Self::id_to_subpath(category, id);
        let (mut asset, _hash): (T, [u8; 32]) = self.hydrate_mpk(&subpath).await?;
        asset.post_load()?;
        Ok(Arc::new(asset))
    }

    async fn load_corpus(&self, sources: &[CorpusSource]) -> LoaderResult<Arc<Corpus>> {
        let mut corpus = Corpus::default();
        for src in sources {
            let base = format!("corpora/{}", src.id);
            for part in ["1grams", "2grams", "3grams", "words"] {
                let path = format!("{base}/{part}.mpk.zst");
                if let Ok(compressed) = self.fetch_blob(&path).await {
                    let part_res: Vec<serde_json::Value> = tokio::task::spawn_blocking(move || {
<<<<<<< HEAD
                        let decoder = zstd::Decoder::new(&bytes[..]).map_err(ForgeError::from)?;
                        rmp_serde::from_read(decoder)
=======
                        let mut decoder = zstd::Decoder::new(&compressed[..])
                            .map_err(|e| ForgeError::Io(e.to_string()))?;
                        let mut decompressed = Vec::new();
                        std::io::copy(&mut decoder, &mut decompressed)
                            .map_err(|e| ForgeError::Io(e.to_string()))?;

                        // Compute hash for verification (DATA-005)
                        let mut hasher = sha2::Sha256::new();
                        sha2::Digest::update(&mut hasher, &decompressed);
                        let _hash: [u8; 32] = hasher.finalize().into();

                        rmp_serde::from_read(&decompressed[..])
>>>>>>> master
                            .map_err(|e| ForgeError::InvalidData(e.to_string()))
                    })
                    .await
                    .map_err(|e| ForgeError::Internal(e.to_string()))??;
                    crate::util::corpus::populate_corpus_from_segments(
                        &mut corpus,
                        src.weight,
                        vec![(part, part_res)],
                    )?;
                }
            }
        }
        inject_synthetic_data(&mut corpus, sources.iter().any(|s| s.id.contains("_std")));
        corpus.post_load()?;
        Ok(Arc::new(corpus))
    }

    async fn get_hash(
        &self,
        category: keyforge_model::AssetCategory,
        id: &str,
    ) -> LoaderResult<String> {
        let subpath = Self::id_to_subpath(category, id);
        let key = format!("{ASSET_PREFIX}:{subpath}");
        match self.coordinator.get_manifest_hash(&key).await {
            Ok(Some(h)) => Ok(h),
            _ => Err(ForgeError::NotFound(id.to_string())),
        }
    }

    fn root(&self) -> &SafePath {
        &self.root
    }
}

#[async_trait]
impl crate::asset::AssetServerProvider for ValkeyProvider {
    async fn get_manifest(&self) -> InfraResult<ServerManifest> {
        let files = self.coordinator.get_all_manifest_entries().await?;
        Ok(ServerManifest { files })
    }

    async fn get_file_content(&self, path: &str) -> InfraResult<Option<bytes::Bytes>> {
        let key = format!("{ASSET_PREFIX}:{path}");
        self.coordinator.get_bin(&key).await
    }
}
