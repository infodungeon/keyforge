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
use crate::net::sync::ServerManifest;
use bytes::Bytes;
use keyforge_core::loader::{AssetLoader, LoaderResult, RawCostData};
use keyforge_model::Corpus;
use keyforge_model::config::CorpusSource;
use keyforge_model::constants::ASSET_KEYCODES;
use keyforge_model::geometry::KeyboardDefinition;
use keyforge_model::keycodes::KeycodeRegistry;
use moka::sync::Cache;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{error, info};

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

/// A thread-safe, caching asset loader with hot-reloading capabilities.
/// Wraps FsProvider with memory caching and file-system watching.
#[derive(Clone)]
pub struct CachingProvider {
    state: Arc<CacheState>,
}

impl CachingProvider {
    /// Creates a new `CachingProvider` that caches assets from the specified data path.
    ///
    /// It also starts a filesystem watcher to invalidate the cache when system assets change.
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

    /// Eagerly loads all system assets into the memory cache.
    ///
    /// This is typically called during application startup to ensure that
    /// core assets are immediately available.
    pub async fn warm_all(&self) -> Result<(), String> {
        info!("🔥 Warming Asset Cache (Full Binary Verification)...");
        let system_root = self.state.provider.root.join("system");

        let manifest = crate::net::sync::generate_manifest(&system_root)
            .map_err(|e| format!("Manifest error: {}", e))?;

        self.state
            .manifest
            .insert("default".into(), Arc::new(manifest.clone()));

        let mut count_files = 0;
        let mut count_keyboards = 0;
        let mut count_corpora = 0;
        let mut count_weights = 0;

        for (rel_path, _) in manifest.files {
            let full_path = system_root.join(&rel_path);
            let bytes =
                tokio::fs::read(&full_path).await.map_err(|e| format!("Read error {}: {}", rel_path, e))?;
            self.state
                .file_cache
                .insert(rel_path.clone(), Bytes::from(bytes));

            count_files += 1;

            if self.try_ensure_keyboard(&rel_path).await? {
                count_keyboards += 1;
                continue;
            }
            if self.try_ensure_corpus(&rel_path).await? {
                count_corpora += 1;
                continue;
            }
            if self.try_ensure_weights(&rel_path).await? {
                count_weights += 1;
                continue;
            }
            if self.try_ensure_config(&rel_path).await? {
                continue;
            }
        }

        if count_files == 0 {
            error!("❌ Asset cache warming failed: 0 assets found in system library.");
            return Err("System library is empty".into());
        }

        info!(
            "✅ Cache Warmed: {} assets ({} kb, {} corp, {} wgt).",
            count_files, count_keyboards, count_corpora, count_weights
        );
        Ok(())
    }

    /// Retrieves the raw byte content of a cached file by its relative path.
    pub fn get_file_content(&self, path: &str) -> Option<Bytes> {
        self.state.file_cache.get(path)
    }

    /// Returns the current system asset manifest.
    pub fn get_manifest(&self) -> Option<Arc<ServerManifest>> {
        self.state.manifest.get("default")
    }

    /// Purges all cached items from memory.
    pub fn invalidate_all(&self) {
        self.state.keyboards.invalidate_all();
        self.state.corpora.invalidate_all();
        self.state.costs.invalidate_all();
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
                if let Err(e) = self.load_keyboard(clean_stem).await {
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
                 if let Err(e) = self.load_cost_matrix(clean_stem).await {
                     tracing::warn!("Eager load failed for weights {}: {}", clean_stem, e);
                 }
                 return Ok(true);
             }
        }
        Ok(false)
    }

    async fn try_ensure_config(&self, rel_path: &str) -> Result<bool, String> {
        if rel_path == format!("config/{}.mpk.zst", ASSET_KEYCODES) {
            if let Err(e) = self.load_keycodes(ASSET_KEYCODES).await {
                 tracing::warn!("Eager load failed for keycodes: {}", e);
            }
            return Ok(true);
        }
        Ok(false)
    }
}

#[async_trait::async_trait]
impl AssetLoader for CachingProvider {
    async fn load_keyboard(&self, name: &str) -> LoaderResult<Arc<KeyboardDefinition>> {
        if let Some(c) = self.state.keyboards.get(name) {
            return Ok(c);
        }
        let kb = self.state.provider.load_keyboard(name).await?;
        self.state
            .keyboards
            .insert(name.to_string(), kb.clone());
        Ok(kb)
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
    async fn load_cost_matrix(&self, filename: &str) -> LoaderResult<Arc<RawCostData>> {
        if let Some(c) = self.state.costs.get(filename) {
            return Ok(c);
        }
        let mt = self.state.provider.load_cost_matrix(filename).await?;
        self.state
            .costs
            .insert(filename.to_string(), mt.clone());
        Ok(mt)
    }
    async fn load_keycodes(&self, filename: &str) -> LoaderResult<Arc<KeycodeRegistry>> {
        if let Some(c) = self.state.keycodes.get(filename) {
            return Ok(c);
        }
        let rg = self.state.provider.load_keycodes(filename).await?;
        self.state
            .keycodes
            .insert(filename.to_string(), rg.clone());
        Ok(rg)
    }
}
