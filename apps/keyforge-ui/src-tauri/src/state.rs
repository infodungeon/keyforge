use keyforge_infra::FsProvider;
use keyforge_model::Corpus;
use keyforge_protocol::config::CorpusSource;
use keyforge_protocol::geometry::KeyboardDefinition;
use keyforge_protocol::keycodes::KeycodeRegistry;
// Removed unused import: WorkspaceError
use keyforge_compute::Runtime;
use keyforge_model::loader::{AssetLoader, LoaderResult, RawCostData};
use moka::sync::Cache;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri_plugin_shell::process::CommandChild;
use tokio::sync::RwLock;

pub struct LocalWorkerState {
    pub child: Arc<Mutex<Option<CommandChild>>>,
}

pub struct SearchState {
    pub stop_flag: Arc<Mutex<bool>>,
}

#[derive(Clone)]
pub struct AssetCache {
    provider: FsProvider,
    keyboards: Cache<String, Arc<KeyboardDefinition>>,
    corpora: Cache<String, Arc<Corpus>>,
    costs: Cache<String, Arc<RawCostData>>,
    keycodes: Cache<String, Arc<KeycodeRegistry>>,
}

impl AssetCache {
    pub fn new(root: PathBuf) -> Self {
        Self {
            provider: FsProvider::new(root),
            keyboards: Cache::builder().max_capacity(100).build(),
            corpora: Cache::builder().max_capacity(50).build(),
            costs: Cache::builder().max_capacity(50).build(),
            keycodes: Cache::builder().max_capacity(10).build(),
        }
    }
}

impl AssetLoader for AssetCache {
    fn load_keyboard(&self, name: &str) -> LoaderResult<KeyboardDefinition> {
        if let Some(cached) = self.keyboards.get(name) {
            return Ok(cached.as_ref().clone());
        }
        let item = self.provider.load_keyboard(name)?;
        self.keyboards
            .insert(name.to_string(), Arc::new(item.clone()));
        Ok(item)
    }

    fn load_corpus(&self, sources: &[CorpusSource]) -> LoaderResult<Corpus> {
        // We need a deterministic key for caching
        let key =
            serde_json::to_string(sources).map_err(keyforge_model::error::ForgeError::Serde)?;

        if let Some(cached) = self.corpora.get(&key) {
            return Ok(cached.as_ref().clone());
        }
        let item = self.provider.load_corpus(sources)?;
        self.corpora.insert(key, Arc::new(item.clone()));
        Ok(item)
    }

    fn load_cost_matrix(&self, filename: &str) -> LoaderResult<RawCostData> {
        if let Some(cached) = self.costs.get(filename) {
            return Ok(cached.as_ref().clone());
        }
        let item = self.provider.load_cost_matrix(filename)?;
        self.costs
            .insert(filename.to_string(), Arc::new(item.clone()));
        Ok(item)
    }

    fn load_keycodes(&self, filename: &str) -> LoaderResult<KeycodeRegistry> {
        if let Some(cached) = self.keycodes.get(filename) {
            return Ok(cached.as_ref().clone());
        }
        let item = self.provider.load_keycodes(filename)?;
        self.keycodes
            .insert(filename.to_string(), Arc::new(item.clone()));
        Ok(item)
    }
}

pub struct SessionState {
    pub active: Arc<RwLock<Option<Runtime>>>,
    pub assets: Arc<AssetCache>,
}
