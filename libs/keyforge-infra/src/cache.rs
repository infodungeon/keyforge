use crate::config::HiveConfig;
use bytes::Bytes;
use keyforge_infra::{listing, AssetLoader, FsProvider, RawCostData, ServerManifest};
use keyforge_model::loader::LoaderResult;
use keyforge_model::Corpus;
use keyforge_protocol::config::{Config as AppConfig, CorpusSource};
use keyforge_protocol::constants::MAX_INPUT_FILE_SIZE;
use keyforge_protocol::geometry::KeyboardDefinition;
use keyforge_protocol::keycodes::KeycodeRegistry;
use moka::sync::Cache;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};

struct AssetCacheState {
    provider: FsProvider,

    // Parsed Objects
    costs: Cache<String, Arc<RawCostData>>,
    corpora: Cache<String, Arc<Corpus>>,
    keyboards: Cache<String, Arc<KeyboardDefinition>>,
    keycodes: Cache<String, Arc<KeycodeRegistry>>,
    hive_config: Cache<String, Arc<HiveConfig>>,
    app_config: Cache<String, Arc<AppConfig>>,

    // Raw File Cache (for Sync)
    file_cache: Cache<String, Bytes>,
    manifest: Cache<String, Arc<ServerManifest>>,

    _watcher: Option<RecommendedWatcher>,
}

#[derive(Clone)]
pub struct GlobalAssetCache {
    state: Arc<AssetCacheState>,
}

impl GlobalAssetCache {
    pub fn new(data_path: PathBuf) -> Self {
        let provider = FsProvider::new(data_path.clone());

        let costs = Cache::builder().max_capacity(100).build();
        let corpora = Cache::builder().max_capacity(50).build();
        let keyboards = Cache::builder().max_capacity(100).build();
        let keycodes = Cache::builder().max_capacity(10).build();
        let hive_config = Cache::builder().max_capacity(1).build();
        let app_config = Cache::builder().max_capacity(1).build();

        let file_cache = Cache::builder().max_capacity(1000).build();
        let manifest = Cache::builder().max_capacity(1).build();

        let costs_clone = costs.clone();
        let corpora_clone = corpora.clone();
        let keyboards_clone = keyboards.clone();
        let keycodes_clone = keycodes.clone();
        let hive_config_clone = hive_config.clone();
        let app_config_clone = app_config.clone();
        let data_path_clone = data_path.clone();

        let watcher_fn = move |res: notify::Result<Event>| match res {
            Ok(event) => {
                if event.kind.is_access() {
                    return;
                }

                for path in event.paths {
                    if let Ok(rel) = path.strip_prefix(&data_path_clone) {
                        let path_str = rel.to_string_lossy();
                        info!("♻️  File changed: {}", path_str);

                        if path_str.contains("keyboards") {
                            keyboards_clone.invalidate_all();
                        } else if path_str.contains("corpora") {
                            corpora_clone.invalidate_all();
                        } else if path_str.contains("weights") {
                            costs_clone.invalidate_all();
                        } else if path_str.contains("config") {
                            if path_str.contains("keycodes") {
                                keycodes_clone.invalidate_all();
                            } else if path_str.contains("hive.json") {
                                hive_config_clone.invalidate_all();
                            } else if path_str.contains("config.json") {
                                app_config_clone.invalidate_all();
                            }
                        }
                    }
                }
            }
            Err(e) => error!("Watcher error: {:?}", e),
        };

        let mut watcher = RecommendedWatcher::new(watcher_fn, Config::default()).ok();

        if let Some(w) = &mut watcher {
            if let Err(e) = w.watch(&data_path, RecursiveMode::Recursive) {
                error!("Failed to start file watcher: {}", e);
            } else {
                info!("👀 Hot-Reload Watcher Active on {:?}", data_path);
            }
        }

        Self {
            state: Arc::new(AssetCacheState {
                provider,
                costs,
                corpora,
                keyboards,
                keycodes,
                hive_config,
                app_config,
                file_cache,
                manifest,
                _watcher: watcher,
            }),
        }
    }

    pub fn warm_all(&self) -> Result<(), String> {
        info!("🔥 Warming Asset Cache (Full Binary Verification)...");
        let system_root = self.state.provider.root.join("system");
        if !system_root.exists() {
            return Err("System directory missing".into());
        }

        let manifest = keyforge_infra::generate_manifest(&system_root)
            .map_err(|e| format!("Failed to generate manifest: {}", e))?;

        self.state
            .manifest
            .insert("default".to_string(), Arc::new(manifest.clone()));

        // Explicit counters for accurate reporting
        let mut count_files = 0;
        let mut count_keyboards = 0;
        let mut count_corpora = 0;

        for (rel_path, _) in manifest.files {
            let full_path = system_root.join(&rel_path);
            let bytes = match std::fs::read(&full_path) {
                Ok(b) => b,
                Err(e) => return Err(format!("Read error {}: {}", rel_path, e)),
            };

            self.state
                .file_cache
                .insert(rel_path.clone(), Bytes::from(bytes));
            count_files += 1;

            if rel_path.starts_with("keyboards/") {
                let stem = std::path::Path::new(&rel_path)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .strip_suffix(".mpk")
                    .unwrap_or("")
                    .to_string();
                if !stem.is_empty() {
                    if let Err(e) = self.load_keyboard(&stem) {
                        tracing::warn!("Eager load failed for keyboard {}: {}", stem, e);
                    } else {
                        count_keyboards += 1;
                    }
                }
            } else if rel_path.starts_with("corpora/") && rel_path.ends_with("1grams.mpk.zst") {
                let path = std::path::Path::new(&rel_path);
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
                            } else {
                                count_corpora += 1;
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

        if count_files == 0 {
            error!("❌ Asset cache warming failed: 0 assets found in system library.");
            return Err("System library is empty".into());
        }

        info!(
            "✅ Cache Warmed: {} assets (including {} keyboards, {} corpora).",
            count_files, count_keyboards, count_corpora
        );
        Ok(())
    }

    pub fn invalidate_all(&self) {
        self.state.costs.invalidate_all();
        self.state.corpora.invalidate_all();
        self.state.keyboards.invalidate_all();
        self.state.keycodes.invalidate_all();
        self.state.hive_config.invalidate_all();
        self.state.app_config.invalidate_all();
    }

    pub fn get_manifest(&self) -> Option<Arc<ServerManifest>> {
        self.state.manifest.get("default")
    }

    pub fn get_file_content(&self, path: &str) -> Option<Bytes> {
        self.state.file_cache.get(path)
    }

    pub fn list_keyboards(&self) -> Vec<String> {
        let mut keys: Vec<String> = self
            .state
            .keyboards
            .iter()
            .map(|(k, _)| k.as_ref().clone())
            .collect();
        keys.sort();
        keys
    }

    pub fn list_corpora(&self) -> Vec<String> {
        let mut ids = Vec::new();
        for (key, _) in self.state.corpora.iter() {
            if let Ok(sources) = serde_json::from_str::<Vec<CorpusSource>>(key.as_ref()) {
                if sources.len() == 1 {
                    ids.push(sources[0].id.clone());
                }
            }
        }
        ids.sort();
        ids.dedup();
        ids
    }

    pub fn list_cost_matrices(&self) -> Vec<String> {
        let mut keys: Vec<String> = self
            .state
            .costs
            .iter()
            .map(|(k, _)| k.as_ref().clone())
            .collect();
        keys.sort();
        keys
    }

    pub fn load_hive_config(&self) -> Arc<HiveConfig> {
        if let Some(cached) = self.state.hive_config.get("default") {
            return cached;
        }

        let path = self.state.provider.root.join("system/config/hive.json");
        let config = if path.exists() {
            match keyforge_infra::read_to_string_limited(&path, MAX_INPUT_FILE_SIZE) {
                Ok(content) => serde_json::from_str(&content).unwrap_or_else(|e| {
                    warn!("Failed to parse hive.json: {}, using defaults", e);
                    HiveConfig::default()
                }),
                Err(e) => {
                    warn!("Failed to read hive.json: {}, using defaults", e);
                    HiveConfig::default()
                }
            }
        } else {
            HiveConfig::default()
        };

        let arc = Arc::new(config);
        self.state
            .hive_config
            .insert("default".to_string(), arc.clone());
        arc
    }

    pub fn load_app_config(&self) -> Arc<AppConfig> {
        if let Some(cached) = self.state.app_config.get("default") {
            return cached;
        }

        let path = self.state.provider.root.join("system/config/config.json");
        let config = if path.exists() {
            match keyforge_infra::read_to_string_limited(&path, MAX_INPUT_FILE_SIZE) {
                Ok(content) => serde_json::from_str(&content).unwrap_or_else(|e| {
                    warn!("Failed to parse config.json: {}, using defaults", e);
                    AppConfig::default()
                }),
                Err(e) => {
                    warn!("Failed to read config.json: {}, using defaults", e);
                    AppConfig::default()
                }
            }
        } else {
            AppConfig::default()
        };

        let arc = Arc::new(config);
        self.state
            .app_config
            .insert("default".to_string(), arc.clone());
        arc
    }
}

impl AssetLoader for GlobalAssetCache {
    fn load_keyboard(&self, name: &str) -> LoaderResult<KeyboardDefinition> {
        if let Some(cached) = self.state.keyboards.get(name) {
            return Ok(cached.as_ref().clone());
        }
        let def = self.state.provider.load_keyboard(name)?;
        self.state
            .keyboards
            .insert(name.to_string(), Arc::new(def.clone()));
        Ok(def)
    }

    fn load_corpus(&self, sources: &[CorpusSource]) -> LoaderResult<Corpus> {
        let mut sorted_sources = sources.to_vec();
        sorted_sources.sort_by(|a, b| a.id.cmp(&b.id));
        let key = serde_json::to_string(&sorted_sources)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        if let Some(cached) = self.state.corpora.get(&key) {
            return Ok(cached.as_ref().clone());
        }
        let corpus = self.state.provider.load_corpus(sources)?;
        self.state.corpora.insert(key, Arc::new(corpus.clone()));
        Ok(corpus)
    }

    fn load_cost_matrix(&self, filename: &str) -> LoaderResult<RawCostData> {
        if let Some(cached) = self.state.costs.get(filename) {
            return Ok(cached.as_ref().clone());
        }
        let data = self.state.provider.load_cost_matrix(filename)?;
        self.state
            .costs
            .insert(filename.to_string(), Arc::new(data.clone()));
        Ok(data)
    }

    fn load_keycodes(&self, filename: &str) -> LoaderResult<KeycodeRegistry> {
        if let Some(cached) = self.state.keycodes.get(filename) {
            return Ok(cached.as_ref().clone());
        }
        let reg = self.state.provider.load_keycodes(filename)?;
        self.state
            .keycodes
            .insert(filename.to_string(), Arc::new(reg.clone()));
        Ok(reg)
    }
}

// Re-export CompiledEngineCache
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
            cache: Cache::builder()
                .max_capacity(500)
                .time_to_live(Duration::from_secs(1800))
                .build(),
        }
    }

    pub fn get(&self, job_id: &str) -> Option<Arc<keyforge_core::ScoringEngine>> {
        self.cache.get(job_id)
    }

    pub fn insert(&self, job_id: &str, engine: Arc<keyforge_core::ScoringEngine>) {
        self.cache.insert(job_id.to_string(), engine);
    }

    pub fn invalidate_all(&self) {
        self.cache.invalidate_all();
    }
}
