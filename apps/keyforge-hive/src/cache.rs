use crate::config::HiveConfig;
use bytes::Bytes;
use keyforge_infra::{listing, CachingProvider, ServerManifest};
use keyforge_core::loader::{AssetLoader, LoaderResult, RawCostData};
use keyforge_model::Corpus;
use keyforge_model::Config as AppConfig;
use keyforge_model::CorpusSource;
use keyforge_model::KeyboardDefinition;
use keyforge_model::KeycodeRegistry;
use moka::sync::Cache;
use std::path::PathBuf;
use std::sync::Arc;


pub struct CompiledEngineCache {
    cache: Cache<String, Arc<keyforge_core::ScoringEngine>>,
}

impl Default for CompiledEngineCache {
    fn default() -> Self {
        Self::new()
    }
}

impl CompiledEngineCache {
    pub fn new() -> Self {
        Self {
            cache: Cache::builder().max_capacity(500).build(),
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

    pub async fn warm_all(&self) -> Result<(), String> {
        self.inner.warm_all().await
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
        Arc::new(AppConfig::default())
    }
    pub fn load_hive_config(&self) -> Arc<HiveConfig> {
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
