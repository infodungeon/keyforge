use crate::config::HiveConfig;
use bytes::Bytes;
use keyforge_infra::{listing, FsProvider, ServerManifest};
use keyforge_model::loader::{AssetLoader, LoaderResult, RawCostData};
use keyforge_model::Corpus;
use keyforge_protocol::config::{Config as AppConfig, CorpusSource};
use keyforge_protocol::geometry::KeyboardDefinition;
use keyforge_protocol::keycodes::KeycodeRegistry;
use moka::sync::Cache;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{error, info};

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

struct CacheState {
    provider: FsProvider,
    keyboards: Cache<String, Arc<KeyboardDefinition>>,
    corpora: Cache<String, Arc<Corpus>>,
    costs: Cache<String, Arc<RawCostData>>,
    keycodes: Cache<String, Arc<KeycodeRegistry>>,
    file_cache: Cache<String, Bytes>,
    manifest: Cache<String, Arc<ServerManifest>>,
    _watcher: Option<RecommendedWatcher>,
}

#[derive(Clone)]
pub struct GlobalAssetCache {
    state: Arc<CacheState>,
}

impl GlobalAssetCache {
    pub fn new(data_path: PathBuf) -> Self {
        let provider = FsProvider::new(data_path.clone());
        let keyboards = Cache::new(100);
        let corpora = Cache::new(50);
        let costs = Cache::new(100);
        let keycodes = Cache::new(10);
        let file_cache = Cache::new(1000);
        let manifest = Cache::new(1);

        let kb_c = keyboards.clone();
        let cp_c = corpora.clone();
        let cs_c = costs.clone();
        let kc_c = keycodes.clone();
        let fl_c = file_cache.clone();
        let mf_c = manifest.clone();
        let dp_c = data_path.clone();

        let watcher_fn = move |res: notify::Result<Event>| match res {
            Ok(event) => {
                if event.kind.is_access() {
                    return;
                }
                for path in event.paths {
                    if let Ok(rel) = path.strip_prefix(&dp_c) {
                        let path_str = rel.to_string_lossy();
                        if !path_str.contains("system") {
                            continue;
                        }
                        info!("♻️ System asset changed: {}", path_str);
                        fl_c.invalidate_all();
                        mf_c.invalidate_all();
                        if path_str.contains("keyboards") {
                            kb_c.invalidate_all();
                        } else if path_str.contains("corpora") {
                            cp_c.invalidate_all();
                        } else if path_str.contains("weights") {
                            cs_c.invalidate_all();
                        } else if path_str.contains("config") {
                            kc_c.invalidate_all();
                        }
                    }
                }
            }
            Err(e) => error!("Watcher error: {:?}", e),
        };

        let mut watcher = RecommendedWatcher::new(watcher_fn, Config::default()).ok();
        if let Some(w) = &mut watcher {
            let _ = w.watch(&data_path, RecursiveMode::Recursive);
            info!("👀 Hot-Reload Watcher Active on {:?}", data_path);
        }

        Self {
            state: Arc::new(CacheState {
                provider,
                keyboards,
                corpora,
                costs,
                keycodes,
                file_cache,
                manifest,
                _watcher: watcher,
            }),
        }
    }

    pub fn warm_all(&self) -> Result<(), String> {
        info!("🔥 Warming Asset Cache (Full Binary Verification)...");
        let system_root = self.state.provider.root.join("system");

        let manifest = keyforge_infra::generate_manifest(&system_root)
            .map_err(|e| format!("Manifest error: {}", e))?;

        self.state
            .manifest
            .insert("default".into(), Arc::new(manifest.clone()));

        for (rel_path, _) in manifest.files {
            let full_path = system_root.join(&rel_path);
            let bytes =
                std::fs::read(&full_path).map_err(|e| format!("Read error {}: {}", rel_path, e))?;
            self.state
                .file_cache
                .insert(rel_path.clone(), Bytes::from(bytes));

            if rel_path.starts_with("keyboards/") {
                let stem = Path::new(&rel_path)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .strip_suffix(".mpk")
                    .unwrap_or("")
                    .to_string();
                if !stem.is_empty() {
                    if let Err(e) = self.load_keyboard(&stem) {
                        tracing::warn!("Eager load failed for keyboard {}: {}", stem, e);
                    }
                }
            } else if rel_path.starts_with("corpora/") && rel_path.ends_with("1grams.mpk.zst") {
                let path = Path::new(&rel_path);
                if let Some(parent) = path.parent() {
                    if let Ok(id_path) = parent.strip_prefix("corpora") {
                        let id = id_path.to_string_lossy().replace('\\', "/");
                        if !id.is_empty() {
                            if let Err(e) = self.load_corpus(&[CorpusSource {
                                id: id.clone(),
                                weight: 1.0,
                                hash: None,
                            }]) {
                                tracing::warn!("Eager load failed for corpus {}: {}", id, e);
                            }
                        }
                    }
                }
            } else if rel_path == "config/keycodes.mpk.zst" {
                if let Err(e) = self.load_keycodes("keycodes") {
                    tracing::warn!("Eager load failed for keycodes: {}", e);
                }
            }
        }

        let count = self.state.file_cache.entry_count();
        if count == 0 {
            error!("❌ Asset cache warming failed: 0 assets found in system library.");
            return Err("System library is empty".into());
        }

        info!(
            "✅ Cache Warmed: {} assets (including {} keyboards, {} corpora).",
            count,
            self.state.keyboards.entry_count(),
            self.state.corpora.entry_count()
        );
        Ok(())
    }

    pub fn get_file_content(&self, path: &str) -> Option<Bytes> {
        self.state.file_cache.get(path)
    }
    pub fn get_manifest(&self) -> Option<Arc<ServerManifest>> {
        self.state.manifest.get("default")
    }
    pub fn invalidate_all(&self) {
        self.state.keyboards.invalidate_all();
        self.state.corpora.invalidate_all();
        self.state.costs.invalidate_all();
        self.state.file_cache.invalidate_all();
    }
    pub fn list_keyboards(&self) -> Vec<String> {
        listing::list_keyboards(&self.state.provider.root).unwrap_or_default()
    }
    pub fn list_corpora(&self) -> Vec<String> {
        listing::list_corpora(&self.state.provider.root).unwrap_or_default()
    }
    pub fn list_cost_matrices(&self) -> Vec<String> {
        listing::list_cost_matrices(&self.state.provider.root).unwrap_or_default()
    }
    pub fn load_app_config(&self) -> Arc<AppConfig> {
        Arc::new(AppConfig::default())
    }
    pub fn load_hive_config(&self) -> Arc<HiveConfig> {
        Arc::new(HiveConfig::default())
    }
    pub fn get_corpus_hash(&self, id: &str) -> LoaderResult<String> {
        self.state.provider.get_corpus_hash(id)
    }
}

impl AssetLoader for GlobalAssetCache {
    fn load_keyboard(&self, name: &str) -> LoaderResult<KeyboardDefinition> {
        if let Some(c) = self.state.keyboards.get(name) {
            return Ok(c.as_ref().clone());
        }
        let kb = self.state.provider.load_keyboard(name)?;
        self.state
            .keyboards
            .insert(name.to_string(), Arc::new(kb.clone()));
        Ok(kb)
    }
    fn load_corpus(&self, sources: &[CorpusSource]) -> LoaderResult<Corpus> {
        let key = serde_json::to_string(sources).unwrap_or_default();
        if let Some(c) = self.state.corpora.get(&key) {
            return Ok(c.as_ref().clone());
        }
        let cp = self.state.provider.load_corpus(sources)?;
        self.state.corpora.insert(key, Arc::new(cp.clone()));
        Ok(cp)
    }
    fn load_cost_matrix(&self, filename: &str) -> LoaderResult<RawCostData> {
        if let Some(c) = self.state.costs.get(filename) {
            return Ok(c.as_ref().clone());
        }
        let mt = self.state.provider.load_cost_matrix(filename)?;
        self.state
            .costs
            .insert(filename.to_string(), Arc::new(mt.clone()));
        Ok(mt)
    }
    fn load_keycodes(&self, filename: &str) -> LoaderResult<KeycodeRegistry> {
        if let Some(c) = self.state.keycodes.get(filename) {
            return Ok(c.as_ref().clone());
        }
        let rg = self.state.provider.load_keycodes(filename)?;
        self.state
            .keycodes
            .insert(filename.to_string(), Arc::new(rg.clone()));
        Ok(rg)
    }
}
