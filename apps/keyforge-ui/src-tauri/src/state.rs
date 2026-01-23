// apps/keyforge-ui/src-tauri/src/state.rs

#![allow(unsafe_code)]
use keyforge_compute::loader::AssetLoader;
use keyforge_infra::AssetManager;
use keyforge_infra::FsProvider;
use keyforge_model::config::CorpusSource;
use keyforge_model::cost_model::CostModel;
use keyforge_model::geometry::KeyboardDefinition;
use keyforge_model::keycodes::KeycodeRegistry;
use keyforge_model::Corpus;

use keyforge_model::constants::{
    DEFAULT_CORPUS_CACHE_CAPACITY, DEFAULT_COST_CACHE_CAPACITY, DEFAULT_KB_CACHE_CAPACITY,
    DEFAULT_KEYCODE_CACHE_CAPACITY,
};
use keyforge_protocol::JobConfig;
use moka::sync::Cache;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri_plugin_shell::process::CommandChild;
use tokio::sync::RwLock;

pub struct LocalWorkerState {
    pub child: Arc<Mutex<Option<CommandChild>>>,
}

impl std::fmt::Debug for LocalWorkerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[allow(clippy::unwrap_used)]
        let guard = self.child.lock().unwrap();
        f.debug_struct("LocalWorkerState")
            .field("active", &guard.is_some())
            .finish()
    }
}

use std::sync::atomic::AtomicBool;

#[derive(Debug)]
pub struct SearchState {
    pub stop_flag: Arc<AtomicBool>,
}

#[derive(Clone, Debug)]
pub struct AssetCache {
    provider: FsProvider,
    pub manager: Arc<RwLock<Option<AssetManager>>>,
    keyboards: Cache<String, Arc<KeyboardDefinition>>,
    corpora: Cache<String, Arc<Corpus>>,
    cost_models: Cache<String, Arc<CostModel>>,
    keycodes: Cache<String, Arc<KeycodeRegistry>>,
}

use std::any::TypeId;

#[async_trait::async_trait]
impl AssetLoader for AssetCache {
    async fn load<T: keyforge_model::Asset>(
        &self,
        id: &str,
    ) -> keyforge_compute::loader::LoaderResult<Arc<T>> {
        if TypeId::of::<T>() == TypeId::of::<KeyboardDefinition>() {
            let res = self
                .load_keyboard_internal(id)
                .await
                .map_err(|e| keyforge_model::error::ForgeError::Internal(e.to_string()))?;
            // SAFETY: We verified TypeId matches KeyboardDefinition.
            let ptr = Arc::into_raw(res).cast::<T>();
            return Ok(unsafe { Arc::from_raw(ptr) });
        }
        if TypeId::of::<T>() == TypeId::of::<CostModel>() {
            let res = self
                .load_cost_model_internal(id)
                .await
                .map_err(|e| keyforge_model::error::ForgeError::Internal(e.to_string()))?;
            let ptr = Arc::into_raw(res).cast::<T>();
            return Ok(unsafe { Arc::from_raw(ptr) });
        }
        if TypeId::of::<T>() == TypeId::of::<KeycodeRegistry>() {
            let res = self
                .load_keycodes_internal(id)
                .await
                .map_err(|e| keyforge_model::error::ForgeError::Internal(e.to_string()))?;
            let ptr = Arc::into_raw(res).cast::<T>();
            return Ok(unsafe { Arc::from_raw(ptr) });
        }

        // Fallback or error for unknown types
        // Ideally we should delegate to provider.load::<T>(id) if not cached, but caching is the point.
        // If T is not one of the cached types, we can try to load directly from provider.
        self.provider.load::<T>(id).await
    }

    async fn load_corpus(
        &self,
        sources: &[CorpusSource],
    ) -> keyforge_compute::loader::LoaderResult<Arc<Corpus>> {
        self.load_corpus_internal(sources)
            .await
            .map_err(|e| keyforge_model::error::ForgeError::Internal(e.to_string()))
    }
}

impl AssetCache {
    #[must_use]
    pub fn new(root: PathBuf) -> Self {
        let kb_size = std::env::var("CACHE_KB_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_KB_CACHE_CAPACITY);
        let cp_size = std::env::var("CACHE_CORPUS_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_CORPUS_CACHE_CAPACITY);
        let cost_size = std::env::var("CACHE_COST_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_COST_CACHE_CAPACITY);
        let keycode_cache_size = std::env::var("CACHE_KEYCODE_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_KEYCODE_CACHE_CAPACITY);

        Self {
            provider: FsProvider::new(root),
            manager: Arc::new(RwLock::new(None)),
            keyboards: Cache::builder().max_capacity(kb_size as u64).build(),
            corpora: Cache::builder().max_capacity(cp_size as u64).build(),
            cost_models: Cache::builder().max_capacity(cost_size as u64).build(),
            keycodes: Cache::builder()
                .max_capacity(keycode_cache_size as u64)
                .build(),
        }
    }

    async fn load_keyboard_internal(&self, name: &str) -> anyhow::Result<Arc<KeyboardDefinition>> {
        if let Some(cached) = self.keyboards.get(name) {
            return Ok(cached);
        }
        match self.provider.load::<KeyboardDefinition>(name).await {
            Ok(item) => {
                self.keyboards.insert(name.to_string(), item.clone());
                Ok(item)
            }
            Err(e) => {
                let manager_guard = self.manager.read().await;
                if let Some(mgr) = &*manager_guard {
                    if mgr.ensure_keyboard(name).await.is_ok() {
                        if let Ok(item) = self.provider.load::<KeyboardDefinition>(name).await {
                            self.keyboards.insert(name.to_string(), item.clone());
                            return Ok(item);
                        }
                    }
                }
                Err(e.into())
            }
        }
    }

    async fn load_corpus_internal(&self, sources: &[CorpusSource]) -> anyhow::Result<Arc<Corpus>> {
        let key = keyforge_infra::util::common::calculate_fingerprint(sources);
        if let Some(cached) = self.corpora.get(&key) {
            return Ok(cached);
        }
        match self.provider.load_corpus(sources).await {
            Ok(item) => {
                self.corpora.insert(key, item.clone());
                Ok(item)
            }
            Err(e) => {
                let manager_guard = self.manager.read().await;
                if let Some(mgr) = &*manager_guard {
                    for source in sources {
                        let _ = mgr.ensure_corpus(&source.id, source.hash.as_deref()).await;
                    }
                    if let Ok(item) = self.provider.load_corpus(sources).await {
                        self.corpora.insert(key, item.clone());
                        return Ok(item);
                    }
                }
                Err(e.into())
            }
        }
    }

    async fn load_cost_model_internal(&self, filename: &str) -> anyhow::Result<Arc<CostModel>> {
        if let Some(cached) = self.cost_models.get(filename) {
            return Ok(cached);
        }
        match self.provider.load::<CostModel>(filename).await {
            Ok(item) => {
                self.cost_models.insert(filename.to_string(), item.clone());
                Ok(item)
            }
            Err(e) => {
                let manager_guard = self.manager.read().await;
                if let Some(mgr) = &*manager_guard {
                    if mgr.ensure_cost_matrix(filename).await.is_ok() {
                        if let Ok(item) = self.provider.load::<CostModel>(filename).await {
                            self.cost_models.insert(filename.to_string(), item.clone());
                            return Ok(item);
                        }
                    }
                }
                Err(e.into())
            }
        }
    }

    async fn load_keycodes_internal(&self, filename: &str) -> anyhow::Result<Arc<KeycodeRegistry>> {
        if let Some(cached) = self.keycodes.get(filename) {
            return Ok(cached);
        }
        let item = self.provider.load::<KeycodeRegistry>(filename).await?;
        self.keycodes.insert(filename.to_string(), item.clone());
        Ok(item)
    }
}

#[derive(Debug)]
pub struct SessionState {
    pub active_job: Arc<RwLock<Option<JobConfig>>>,
    pub assets: Arc<AssetCache>,
    pub client: Arc<RwLock<Option<keyforge_infra::HiveClient>>>,
    pub scoring_session: Arc<RwLock<Option<keyforge_compute::ScoringSession>>>,
}
