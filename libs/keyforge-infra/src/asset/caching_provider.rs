// libs/keyforge-infra/src/asset/caching_provider.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not userefix except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::asset::fs_provider::FsProvider;
use crate::asset::AssetServerProvider;
use crate::net::sync::ServerManifest;
use bytes::Bytes;
use keyforge_core::loader::{AssetLoader, LoaderResult};
use keyforge_model::{Asset, Corpus, ForgeError};
use keyforge_model::config::CorpusSource;
use keyforge_model::constants::{
    ASSET_KEYCODES, DEFAULT_CORPUS_CACHE_CAPACITY, DEFAULT_COST_CACHE_CAPACITY,
    DEFAULT_KB_CACHE_CAPACITY, DEFAULT_KEYCODE_CACHE_CAPACITY,
};
use keyforge_model::geometry::KeyboardDefinition;
use keyforge_model::keycodes::KeycodeRegistry;
use keyforge_model::cost_model::CostModel;
use moka::sync::Cache;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{error, info};
use std::any::{Any, TypeId};

struct CacheState {
    provider: FsProvider,
    keyboards: Cache<String, Arc<KeyboardDefinition>>,
    corpora: Cache<String, Arc<Corpus>>,
    cost_models: Cache<String, Arc<CostModel>>,
    keycodes: Cache<String, Arc<KeycodeRegistry>>,
    file_cache: Cache<String, Bytes>,
    manifest: Cache<String, Arc<ServerManifest>>,
    _watcher: Option<RecommendedWatcher>,
}

impl std::fmt::Debug for CacheState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CacheState")
            .field("provider", &self.provider)
            .field("keyboards", &"Cache")
            .field("corpora", &"Cache")
            .field("cost_models", &"Cache")
            .field("keycodes", &"Cache")
            .field("file_cache", &"Cache")
            .field("manifest", &"Cache")
            .field("_watcher", &self._watcher.as_ref().map(|_| "RecommendedWatcher"))
            .finish()
    }
}

/// A thread-safe, caching asset loader with hot-reloading capabilities.
/// Wraps FsProvider with memory caching and file-system watching.
#[derive(Clone, Debug)]
pub struct CachingProvider {
    state: Arc<CacheState>,
}

impl CachingProvider {
    /// Creates a new `CachingProvider` that caches assets from the specified data path.
    ///
    /// It also starts a filesystem watcher to invalidate the cache when system assets change.
    pub fn new(data_path: PathBuf) -> Self {
        let provider = FsProvider::new(data_path.clone());
        let keyboards = Cache::new(DEFAULT_KB_CACHE_CAPACITY as u64);
        let corpora = Cache::new(DEFAULT_CORPUS_CACHE_CAPACITY as u64);
        let cost_models = Cache::new(DEFAULT_COST_CACHE_CAPACITY as u64);
        let keycodes = Cache::new(DEFAULT_KEYCODE_CACHE_CAPACITY as u64);
        let file_cache = Cache::new(1000); // RAW binary cache
        let manifest = Cache::new(1);

        let kb_c = keyboards.clone();
        let cp_c = corpora.clone();
        let cm_c = cost_models.clone();
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
                        
                        // Granular Invalidation
                        fl_c.invalidate(path_str.as_ref());
                        mf_c.invalidate_all();

                        if path_str.contains("keyboards") {
                            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                                let clean = stem.strip_suffix(".mpk").unwrap_or(stem);
                                kb_c.invalidate(clean);
                            } else {
                                kb_c.invalidate_all();
                            }
                        } else if path_str.contains("corpora") {
                            // Corpora keys are complex JSON strings of sources.
                            // Hard to map file -> key. Invalidate all for safety.
                            cp_c.invalidate_all();
                        } else if path_str.contains("weights") {
                            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                                let clean = stem.strip_suffix(".mpk").unwrap_or(stem);
                                cm_c.invalidate(clean);
                            } else {
                                cm_c.invalidate_all();
                            }
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
                cost_models,
                keycodes,
                file_cache,
                manifest,
                _watcher: watcher,
            }),
        }
    }

    /// Eagerly loads system assets into the memory cache.
    ///
    /// This is typically called during application startup. A safety limit is enforced
    /// to prevent excessive memory consumption if the system library is unexpectedly large.
    pub async fn warm_all(&self) -> Result<(), String> {
        info!("🔥 Warming Asset Cache (Parsed Objects Only)...");
        let system_root = self.state.provider.root().join("system");

        let manifest = crate::net::sync::generate_manifest(&system_root)
            .map_err(|e| format!("Manifest error: {}", e))?;

        self.state
            .manifest
            .insert("default".into(), Arc::new(manifest.clone()));

        let mut count_files = 0;
        let mut count_keyboards = 0;
        let mut count_corpora = 0;
        let mut count_weights = 0;

        // SAFETY LIMIT: Don't warm more than 1000 files to prevent OOM
        const MAX_WARM_FILES: usize = 1000;

        for (rel_path, _) in manifest.files.iter().take(MAX_WARM_FILES) {
            // OPTIMIZATION: Do NOT load file content into file_cache eagerly.
            // Only warm high-cost parsed objects.
            count_files += 1;

            if self.try_ensure_keyboard(rel_path).await? {
                count_keyboards += 1;
                continue;
            }
            if self.try_ensure_corpus(rel_path).await? {
                count_corpora += 1;
                continue;
            }
            if self.try_ensure_weights(rel_path).await? {
                count_weights += 1;
                continue;
            }
            if self.try_ensure_config(rel_path).await? {
                continue;
            }
        }

        if manifest.files.len() > MAX_WARM_FILES {
            tracing::warn!(
                "System library contains {} files, but only {} were warmed. Remaining assets will be loaded lazily.",
                manifest.files.len(),
                MAX_WARM_FILES
            );
        }

        if count_files == 0 {
            error!("❌ Asset cache warming failed: 0 assets found in system library.");
            return Err("System library is empty".into());
        }

        info!(
            "✅ Cache Warmed: {} assets scanned ({} kb, {} corp, {} wgt).",
            count_files, count_keyboards, count_corpora, count_weights
        );
        Ok(())
    }

    /// Retrieves the raw byte content of a cached file by its relative path.
    /// Lazily loads from disk if not in cache.
    pub async fn get_file_content(&self, path: &str) -> Option<Bytes> {
        if let Some(bytes) = self.state.file_cache.get(path) {
            return Some(bytes);
        }

        let system_root = self.state.provider.root().join("system");
        let full_path = system_root.join(path);

        match tokio::fs::read(&full_path).await {
            Ok(data) => {
                let bytes = Bytes::from(data);
                self.state.file_cache.insert(path.to_string(), bytes.clone());
                Some(bytes)
            }
            Err(e) => {
                tracing::debug!("Cache miss & Disk read failed for {}: {}", path, e);
                None
            }
        }
    }

    /// Returns the current system asset manifest.
    pub fn get_manifest(&self) -> Option<Arc<ServerManifest>> {
        self.state.manifest.get("default")
    }

    /// Purges all cached items from memory.
    pub fn invalidate_all(&self) {
        self.state.keyboards.invalidate_all();
        self.state.corpora.invalidate_all();
        self.state.cost_models.invalidate_all();
        self.state.file_cache.invalidate_all();
        self.state.manifest.invalidate_all();
        self.state.keycodes.invalidate_all();
    }

    /// Calculates a stable hash for a corpus, using the underlying `FsProvider`.
    pub async fn get_corpus_hash(&self, id: &str) -> LoaderResult<String> {
        self.state.provider.get_corpus_hash(id).await
    }

    async fn try_ensure_keyboard(&self, rel_path: &str) -> Result<bool, String> {
        if rel_path.starts_with(crate::asset::ASSET_PATH_KEYBOARDS) {
            let path = Path::new(rel_path);
            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
             let clean_stem = if let Some(s) = stem.strip_suffix(".mpk") { s } else { stem };
            if !clean_stem.is_empty() {
                if let Err(e) = self.load::<KeyboardDefinition>(clean_stem).await {
                    tracing::warn!("Eager load failed for keyboard {}: {}", clean_stem, e);
                }
                return Ok(true);
            }
        }
        Ok(false)
    }

    async fn try_ensure_corpus(&self, rel_path: &str) -> Result<bool, String> {
        if rel_path.starts_with(crate::asset::ASSET_PATH_CORPORA) && rel_path.ends_with("1grams.mpk.zst") {
             let path = Path::new(rel_path);
             if let Some(parent) = path.parent() {
                 if let Ok(id_path) = parent.strip_prefix("corpora") {
                     let id = id_path.to_string_lossy().replace('\\', "/");
                     if !id.is_empty() {
                         if let Err(e) = self.load_corpus(&[CorpusSource {
                             id: id.clone(),
                             weight: 1.0,
                             hash: None,
                         }]).await {
                             tracing::warn!("Eager load failed for corpus {}: {}", id, e);
                         }
                         return Ok(true);
                     }
                 }
             }
        }
        Ok(false)
    }

    async fn try_ensure_weights(&self, rel_path: &str) -> Result<bool, String> {
        if rel_path.starts_with(crate::asset::ASSET_PATH_WEIGHTS) {
             let path = Path::new(rel_path);
             let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
             let clean_stem = if let Some(s) = stem.strip_suffix(".mpk") { s } else { stem };

             if !clean_stem.is_empty() {
                 let _ = self.load::<CostModel>(clean_stem).await;
                 return Ok(true);
             }
        }
        Ok(false)
    }

    async fn try_ensure_config(&self, rel_path: &str) -> Result<bool, String> {
        if rel_path == format!("config/{}.mpk.zst", ASSET_KEYCODES) {
            if let Err(e) = self.load::<KeycodeRegistry>(ASSET_KEYCODES).await {
                 tracing::warn!("Eager load failed for keycodes: {}", e);
            }
            return Ok(true);
        }
        Ok(false)
    }
}

#[async_trait::async_trait]
impl AssetServerProvider for CachingProvider {
    async fn get_manifest(&self) -> ServerManifest {
        // If manifest is missing, regenerate it.
        // We assume warm_all has been called, but for robustness:
        if let Some(m) = self.get_manifest() {
            return (*m).clone();
        }
        // Fallback: Generate on fly (expensive)
        let system_root = self.state.provider.root().join("system");
        crate::net::sync::generate_manifest(&system_root).unwrap_or_default()
    }

    async fn get_file_content(&self, path: &str) -> Option<bytes::Bytes> {
        self.get_file_content(path).await
    }
}

#[async_trait::async_trait]
impl AssetLoader for CachingProvider {
    async fn load<T: Asset>(&self, id: &str) -> LoaderResult<Arc<T>> {
        let tid = TypeId::of::<T>();

        if tid == TypeId::of::<KeyboardDefinition>() {
            let kb = if let Some(c) = self.state.keyboards.get(id) {
                c
            } else {
                let kb = self.state.provider.load::<KeyboardDefinition>(id).await?;
                self.state.keyboards.insert(id.to_string(), kb.clone());
                kb
            };
            let any_kb: Arc<dyn Any + Send + Sync> = kb;
            return any_kb.downcast::<T>().map_err(|_| ForgeError::Internal("Downcast failed".into()));
        }

        if tid == TypeId::of::<CostModel>() {
            let cm = if let Some(c) = self.state.cost_models.get(id) {
                c
            } else {
                let cm = self.state.provider.load::<CostModel>(id).await?;
                self.state.cost_models.insert(id.to_string(), cm.clone());
                cm
            };
            let any_cm: Arc<dyn Any + Send + Sync> = cm;
            return any_cm.downcast::<T>().map_err(|_| ForgeError::Internal("Downcast failed".into()));
        }

        if tid == TypeId::of::<KeycodeRegistry>() {
            let rg = if let Some(c) = self.state.keycodes.get(id) {
                c
            } else {
                let rg = self.state.provider.load::<KeycodeRegistry>(id).await?;
                self.state.keycodes.insert(id.to_string(), rg.clone());
                rg
            };
            let any_rg: Arc<dyn Any + Send + Sync> = rg;
            return any_rg.downcast::<T>().map_err(|_| ForgeError::Internal("Downcast failed".into()));
        }

        // Fallback for types without dedicated caches
        self.state.provider.load::<T>(id).await
    }

    async fn load_corpus(&self, sources: &[CorpusSource]) -> LoaderResult<Arc<Corpus>> {
        let key = serde_json::to_string(sources).unwrap_or_default();
        if let Some(c) = self.state.corpora.get(&key) {
            return Ok(c);
        }
        let cp = self.state.provider.load_corpus(sources).await?;
        self.state.corpora.insert(key, cp.clone());
        Ok(cp)
    }
}
