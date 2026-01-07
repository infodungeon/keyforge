use crate::config::HiveConfig;
use bytes::Bytes;
use keyforge_infra::{listing, AssetLoader, CachingProvider, ServerManifest, DistributedCoordinator};
use keyforge_core::loader::{LoaderResult, RawCostData};
use keyforge_model::Corpus;
use keyforge_model::config::{Config as AppConfig, CorpusSource};
use keyforge_model::geometry::KeyboardDefinition;
use keyforge_model::keycodes::KeycodeRegistry;
use keyforge_protocol::AssetManifestEntry;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;

pub struct CompiledEngineCache {
    cache: moka::sync::Cache<String, Arc<keyforge_core::ScoringEngine>>,
}

impl Default for CompiledEngineCache {
    fn default() -> Self {
        Self::new()
    }
}

impl CompiledEngineCache {
    pub fn new() -> Self {
        Self {
            cache: moka::sync::Cache::builder()
                .max_capacity(500)
                .build(),
        }
    }
    pub fn get(&self, id: &str) -> Option<Arc<keyforge_core::ScoringEngine>> {
        self.cache.get(id)
    }
    pub fn insert(&self, id: &str, engine: Arc<keyforge_core::ScoringEngine>) {
        self.cache.insert(id.to_string(), engine);
    }
    pub fn invalidate_all(&self) {
        self.cache.invalidate_all();
    }
}

#[derive(Clone)]
pub struct GlobalAssetCache {
    inner: CachingProvider,
    data_path: PathBuf,
}

impl GlobalAssetCache {
    pub fn new(data_path: PathBuf) -> Self {
        Self {
            inner: CachingProvider::new(data_path.clone()),
            data_path,
        }
    }

    pub async fn warm_all(&self, coordinator: &DistributedCoordinator) -> Result<(), String> {
        // 1. Local Warmup (Disk -> RAM)
        self.inner.warm_all().await?;

        // 2. Distributed Sync (RAM -> Valkey)
        if let Some(manifest) = self.inner.get_manifest() {
            info!("🌍 Syncing {} assets to Distributed Manifest...", manifest.files.len());
            
            for (id, hash) in &manifest.files {
                let entry = AssetManifestEntry {
                    id: id.clone(),
                    hash: hash.clone(),
                    size_bytes: 0, 
                    last_updated: chrono::Utc::now().timestamp() as u64,
                };
                
                if let Err(e) = coordinator.set_manifest_entry(&entry).await {
                    tracing::warn!("Failed to sync asset {} to Valkey: {}", id, e);
                }
            }
        }
        Ok(())
    }

    pub fn get_file_content(&self, path: &str) -> Option<Bytes> {
        self.inner.get_file_content(path)
    }
    pub fn get_manifest(&self) -> Option<Arc<ServerManifest>> {
        self.inner.get_manifest()
    }
    pub fn invalidate_all(&self) {
        self.inner.invalidate_all();
    }
    pub fn list_keyboards(&self) -> Vec<String> {
        listing::list_keyboards(&self.data_path).unwrap_or_default()
    }
    pub fn list_corpora(&self) -> Vec<String> {
        listing::list_corpora(&self.data_path).unwrap_or_default()
    }
    pub fn list_cost_matrices(&self) -> Vec<String> {
        listing::list_cost_matrices(&self.data_path).unwrap_or_default()
    }
    pub fn load_app_config(&self) -> Arc<AppConfig> {
        // Default impl for now
        Arc::new(AppConfig::default())
    }
    pub fn load_hive_config(&self) -> Arc<HiveConfig> {
        // CachingProvider wraps FsProvider which handles raw loading,
        // but currently load_hive_config isn't exposed on CachingProvider directly
        // in the infra struct. We use default for now or expose it if needed.
        // For phase 3, we use default to satisfy the trait/struct usage.
        Arc::new(HiveConfig::default())
    }
    pub async fn get_corpus_hash(&self, id: &str) -> LoaderResult<String> {
        self.inner.get_corpus_hash(id).await
    }
}

#[async_trait::async_trait]
impl AssetLoader for GlobalAssetCache {
    async fn load_keyboard(&self, name: &str) -> LoaderResult<KeyboardDefinition> {
        self.inner.load_keyboard(name).await
    }
    async fn load_corpus(&self, sources: &[CorpusSource]) -> LoaderResult<Corpus> {
        self.inner.load_corpus(sources).await
    }
    async fn load_cost_matrix(&self, filename: &str) -> LoaderResult<RawCostData> {
        self.inner.load_cost_matrix(filename).await
    }
    async fn load_keycodes(&self, filename: &str) -> LoaderResult<KeycodeRegistry> {
        self.inner.load_keycodes(filename).await
    }
}