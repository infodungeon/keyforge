use keyforge_infra::FsProvider;
use keyforge_model::Corpus;
use keyforge_model::config::CorpusSource;
use keyforge_model::geometry::KeyboardDefinition;
use keyforge_model::keycodes::KeycodeRegistry;
use keyforge_infra::RawCostData;
use keyforge_infra::AssetManager;
use keyforge_infra::AssetLoader; // FsProvider still implements it, need trait to call load_* methods on provider?
// Actually FsProvider implements AssetLoader, so we need the trait in scope to call its methods.
// But we won't implement it for AssetCache.

use keyforge_protocol::JobConfig;
use keyforge_model::constants::{
    DEFAULT_CORPUS_CACHE_CAPACITY, DEFAULT_COST_CACHE_CAPACITY, DEFAULT_KB_CACHE_CAPACITY,
    DEFAULT_KEYCODE_CACHE_CAPACITY,
};
use moka::sync::Cache;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri_plugin_shell::process::CommandChild;
use tokio::sync::RwLock;

/// Manages the state and lifecycle of a local child process worker.
pub struct LocalWorkerState {
    /// Handle to the active child process, if any.
    pub child: Arc<Mutex<Option<CommandChild>>>,
}

impl std::fmt::Debug for LocalWorkerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let guard = self.child.lock().unwrap();
        f.debug_struct("LocalWorkerState")
            .field("active", &guard.is_some())
            .finish()
    }
}

/// Shared flag used to signal stop to asynchronous search operations.
#[derive(Debug)]
pub struct SearchState {
    /// Thread-safe flag indicating if the search should terminate.
    pub stop_flag: Arc<Mutex<bool>>,
}

/// In-memory cache for frequently accessed assets, backed by the filesystem.
///
/// This provides a thread-safe implementation of `AssetLoader` with LRU eviction,
/// and can optionally download missing assets via an `AssetManager`.
#[derive(Clone, Debug)]
pub struct AssetCache {
    /// The underlying filesystem provider for loading assets.
    provider: FsProvider,
    /// Optional manager for downloading missing assets from a remote server.
    pub manager: Arc<RwLock<Option<AssetManager>>>,
    /// Cache for keyboard geometry definitions.
    keyboards: Cache<String, Arc<KeyboardDefinition>>,
    /// Cache for processed corpora.
    corpora: Cache<String, Arc<Corpus>>,
    /// Cache for raw cost matrix data.
    costs: Cache<String, Arc<RawCostData>>,
    /// Cache for keycode registries.
    keycodes: Cache<String, Arc<KeycodeRegistry>>,
}

impl AssetCache {
    /// Creates a new `AssetCache` rooted at the specified data directory.
    pub fn new(root: PathBuf) -> Self {
        // Configurable cache sizes via env or defaults
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
        let kc_size = std::env::var("CACHE_KEYCODE_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_KEYCODE_CACHE_CAPACITY);

        Self {
            provider: FsProvider::new(root),
            manager: Arc::new(RwLock::new(None)),
            keyboards: Cache::builder().max_capacity(kb_size as u64).build(),
            corpora: Cache::builder().max_capacity(cp_size as u64).build(),
            costs: Cache::builder().max_capacity(cost_size as u64).build(),
            keycodes: Cache::builder().max_capacity(kc_size as u64).build(),
        }
    }

    pub async fn load_keyboard(&self, name: &str) -> anyhow::Result<Arc<KeyboardDefinition>> {
        if let Some(cached) = self.keyboards.get(name) {
            return Ok(cached);
        }

        // Try local load
        match self.provider.load_keyboard(name).await {
            Ok(item) => {
                self.keyboards.insert(name.to_string(), item.clone());
                Ok(item)
            }
            Err(e) => {
                // Try remote download if manager is available
                let manager_guard = self.manager.read().await;
                if let Some(mgr) = &*manager_guard {
                    tracing::info!("Asset '{}' not found locally, attempting remote download...", name);
                    if mgr.ensure_keyboard(name).await.is_ok() {
                        // Retry local load after download
                        if let Ok(item) = self.provider.load_keyboard(name).await {
                            self.keyboards.insert(name.to_string(), item.clone());
                            return Ok(item);
                        }
                    }
                }
                Err(e.into())
            }
        }
    }

    pub async fn load_corpus(&self, sources: &[CorpusSource]) -> anyhow::Result<Arc<Corpus>> {
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

    pub async fn load_cost_matrix(&self, filename: &str) -> anyhow::Result<Arc<RawCostData>> {
        if let Some(cached) = self.costs.get(filename) {
            return Ok(cached);
        }

        match self.provider.load_cost_matrix(filename).await {
            Ok(item) => {
                self.costs.insert(filename.to_string(), item.clone());
                Ok(item)
            }
            Err(e) => {
                let manager_guard = self.manager.read().await;
                if let Some(mgr) = &*manager_guard {
                    if mgr.ensure_cost_matrix(filename).await.is_ok() {
                        if let Ok(item) = self.provider.load_cost_matrix(filename).await {
                            self.costs.insert(filename.to_string(), item.clone());
                            return Ok(item);
                        }
                    }
                }
                Err(e.into())
            }
        }
    }

    pub async fn load_keycodes(&self, filename: &str) -> anyhow::Result<Arc<KeycodeRegistry>> {
        if let Some(cached) = self.keycodes.get(filename) {
            return Ok(cached);
        }
        let item = self.provider.load_keycodes(filename).await?;
        self.keycodes.insert(filename.to_string(), item.clone());
        Ok(item)
    }
}

/// Central application state for the current user session.
#[derive(Debug)]
pub struct SessionState {
    /// The active job configuration.
    pub active_job: Arc<RwLock<Option<JobConfig>>>,
    /// Global asset cache for the session.
    pub assets: Arc<AssetCache>,
    /// Optional client for interacting with the remote Hive.
    pub client: Arc<RwLock<Option<keyforge_infra::HiveClient>>>,
}