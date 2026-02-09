// apps/keyforge-ui/src-tauri/src/state.rs

use crate::utils::get_data_dir;
use async_trait::async_trait;
use keyforge_adapter::loader::{AssetLoader, LoaderResult};
use keyforge_compute::ScoringSession;
use keyforge_infra::AssetManager;
use keyforge_model::config::CorpusSource;
use keyforge_model::error::ForgeError;
use keyforge_model::{Asset, Corpus};
use sha2::Digest;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Shared application state managed by Tauri.
#[derive(Debug)]
pub struct SessionState {
    pub assets: Arc<AssetCache>,
    pub active_job: Arc<RwLock<Option<keyforge_protocol::JobConfig>>>,
    pub scoring_session: Arc<RwLock<Option<ScoringSession>>>,
}

pub struct LocalWorkerState {
    pub child: std::sync::Mutex<Option<tauri_plugin_shell::process::CommandChild>>,
}

impl fmt::Debug for LocalWorkerState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LocalWorkerState")
            .field(
                "child",
                &self.child.lock().map(|c| c.is_some()).unwrap_or(false),
            )
            .finish()
    }
}

#[derive(Debug)]
pub struct SearchState {
    pub stop_flag: Arc<std::sync::atomic::AtomicBool>,
}

/// A tiered asset cache that prioritizes high-performance reads for UI commands.
#[derive(Debug)]
pub struct AssetCache {
    pub root: PathBuf,
    pub manager: Arc<AssetManager>,
}

impl AssetCache {
    /// Creates a new `AssetCache` using the application handle to resolve the data directory.
    pub fn new(app: &tauri::AppHandle) -> Result<Self, crate::error::CommandError> {
        let root = get_data_dir(app)?;
        let client_config = keyforge_infra::net::client::ClientConfig::default();
        let client = keyforge_infra::HiveClient::new(client_config)?;
        let manager = Arc::new(AssetManager::new(client, root.clone()));
        Ok(Self { root, manager })
    }
}

#[async_trait]
impl AssetLoader for AssetCache {
    async fn load<T: Asset>(&self, id: &str) -> LoaderResult<Arc<T>> {
        self.manager
            .ensure_keyboard(id)
            .await
            .map_err(|e| ForgeError::NotFound(e.to_string()))?;
        let path = self
            .root
            .join("system/keyboards")
            .join(format!("{id}.json"));
        let data = std::fs::read(path).map_err(|e| ForgeError::Io(e.to_string()))?;
        serde_json::from_slice(&data)
            .map(Arc::new)
            .map_err(|e| ForgeError::Serde(e.to_string()))
    }

    async fn load_corpus(&self, sources: &[CorpusSource]) -> LoaderResult<Arc<Corpus>> {
        let mut blended = Corpus::default();
        for src in sources {
            self.manager
                .ensure_corpus(&src.id, None)
                .await
                .map_err(|e| ForgeError::NotFound(e.to_string()))?;
            let path = self.root.join("system/corpora").join(&src.id);
            let data = std::fs::read(path).map_err(|e| ForgeError::Io(e.to_string()))?;
            let corpus: Corpus =
                serde_json::from_slice(&data).map_err(|e| ForgeError::Serde(e.to_string()))?;
            blended.merge(&corpus, src.weight);
        }
        Ok(Arc::new(blended))
    }

    async fn get_hash(&self, _category: keyforge_model::AssetCategory, id: &str) -> LoaderResult<String> {
        let path = self.root.join("system/keyboards").join(format!("{id}.json"));
        if path.exists() {
            let data = std::fs::read(path).map_err(|e| ForgeError::Io(e.to_string()))?;
            let mut hasher = sha2::Sha256::new();
            sha2::Digest::update(&mut hasher, &data);
            Ok(hex::encode(hasher.finalize()))
        } else {
            Ok("ui-placeholder-hash".to_string())
        }
    }

    fn root(&self) -> &Path {
        &self.root
    }
}

#[derive(Debug)]
pub struct AppState {
    pub hive: Arc<keyforge_infra::AssetManager>,
}
