use keyforge_infra::FsProvider;
use keyforge_model::Corpus;
use keyforge_model::config::CorpusSource;
use keyforge_model::geometry::KeyboardDefinition;
use keyforge_model::keycodes::KeycodeRegistry;
// Removed unused import: WorkspaceError
use keyforge_compute::Runtime;
use keyforge_core::loader::{AssetLoader, LoaderResult, RawCostData};
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

/// Shared flag used to signal stop to asynchronous search operations.
pub struct SearchState {
    /// Thread-safe flag indicating if the search should terminate.
    pub stop_flag: Arc<Mutex<bool>>,
}

/// In-memory cache for frequently accessed assets, backed by the filesystem.
///
/// This provides a thread-safe implementation of `AssetLoader` with LRU eviction.
#[derive(Clone)]
pub struct AssetCache {
    /// The underlying filesystem provider for loading assets.
    provider: FsProvider,
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
        let kb_size = std::env::var("CACHE_KB_SIZE").ok().and_then(|s| s.parse().ok()).unwrap_or(100);
        let cp_size = std::env::var("CACHE_CORPUS_SIZE").ok().and_then(|s| s.parse().ok()).unwrap_or(50);

        Self {
            provider: FsProvider::new(root),
            keyboards: Cache::builder().max_capacity(kb_size).build(),
            corpora: Cache::builder().max_capacity(cp_size).build(),
            costs: Cache::builder().max_capacity(50).build(),
            keycodes: Cache::builder().max_capacity(10).build(),
        }
    }
}

#[async_trait::async_trait]
impl AssetLoader for AssetCache {
    async fn load_keyboard(&self, name: &str) -> LoaderResult<Arc<KeyboardDefinition>> {
        if let Some(cached) = self.keyboards.get(name) {
            tracing::info!("Loaded keyboard '{}' from cache. Keys: {}", name, cached.geometry.keys.len());
            return Ok(cached);
        }
        let item = self.provider.load_keyboard(name).await?;
        tracing::info!("Loaded keyboard '{}' from disk. Keys: {}", name, item.geometry.keys.len());
        self.keyboards.insert(name.to_string(), item.clone());
        Ok(item)
    }

    async fn load_corpus(&self, sources: &[CorpusSource]) -> LoaderResult<Arc<Corpus>> {
        // We need a deterministic key for caching
        let key =
            serde_json::to_string(sources).map_err(keyforge_model::error::ForgeError::Serde)?;

        if let Some(cached) = self.corpora.get(&key) {
            return Ok(cached);
        }
        let item = self.provider.load_corpus(sources).await?;
        self.corpora.insert(key, item.clone());
        Ok(item)
    }

    async fn load_cost_matrix(&self, filename: &str) -> LoaderResult<Arc<RawCostData>> {
        if let Some(cached) = self.costs.get(filename) {
            return Ok(cached);
        }
        let item = self.provider.load_cost_matrix(filename).await?;
        self.costs.insert(filename.to_string(), item.clone());
        Ok(item)
    }

    async fn load_keycodes(&self, filename: &str) -> LoaderResult<Arc<KeycodeRegistry>> {
        if let Some(cached) = self.keycodes.get(filename) {
            return Ok(cached);
        }
        let item = self.provider.load_keycodes(filename).await?;
        self.keycodes.insert(filename.to_string(), item.clone());
        Ok(item)
    }
}

/// Central application state for the current user session.
pub struct SessionState {
    /// The active search runtime, if one is currently engaged.
    pub active: Arc<RwLock<Option<Runtime>>>,
    /// Global asset cache for the session.
    pub assets: Arc<AssetCache>,
}
