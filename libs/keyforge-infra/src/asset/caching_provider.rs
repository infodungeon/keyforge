// libs/keyforge-infra/src/asset/caching_provider.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::asset::cache::AssetCache;
use crate::asset::fs_provider::FsProvider;
use crate::asset::AssetServerProvider;
use crate::net::sync::ServerManifest;
use bytes::Bytes;
use keyforge_core::loader::{AssetLoader, LoaderResult};
use keyforge_model::config::CorpusSource;
use keyforge_model::constants::ASSET_KEYCODES;
use keyforge_model::cost_model::CostModel;
use keyforge_model::geometry::KeyboardDefinition;
use keyforge_model::keycodes::KeycodeRegistry;
use keyforge_model::{Asset, Corpus, ForgeError};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::any::{Any, TypeId};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{error, info};

/// A thread-safe, caching asset loader with hot-reloading capabilities.
/// Wraps `FsProvider` with memory caching and file-system watching.
#[derive(Clone, Debug)]
pub struct CachingProvider {
    provider: FsProvider,
    cache: Arc<AssetCache>,
    // Watcher is held in Arc to keep it alive but we don't need to access it directly often.
    // It's just for side-effects (invalidation).
    _watcher: Arc<Option<RecommendedWatcher>>,
}

impl CachingProvider {
    /// Creates a new `CachingProvider` that caches assets from the specified data path.
    ///
    /// It also starts a filesystem watcher to invalidate the cache when system assets change.
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(data_path: PathBuf) -> Self {
        let provider = FsProvider::new(data_path.clone());
        let cache = Arc::new(AssetCache::new());

        let cache_clone = cache.clone();
        let dp_c = data_path.clone();

        let watcher_fn = move |res: notify::Result<Event>| match res {
            Ok(event) => {
                if event.kind.is_access() {
                    return;
                }
                Self::handle_fs_event(event, &dp_c, &cache_clone);
            }
            Err(e) => error!("Watcher error: {:?}", e),
        };

        let mut watcher = RecommendedWatcher::new(watcher_fn, Config::default()).ok();
        if let Some(w) = &mut watcher {
            let _ = w.watch(&data_path, RecursiveMode::Recursive);
            info!("👀 Hot-Reload Watcher Active on {:?}", data_path);
        }

        Self {
            provider,
            cache,
            _watcher: Arc::new(watcher),
        }
    }

    fn handle_fs_event(event: Event, root: &Path, cache: &AssetCache) {
        for path in event.paths {
            if let Ok(rel) = path.strip_prefix(root) {
                let path_str = rel.to_string_lossy();
                if path_str.contains("system") {
                    info!("♻️ System asset changed: {}", path_str);

                    // Granular Invalidation
                    cache.invalidate_file(path_str.as_ref());
                    cache.invalidate_manifest();

                    if path_str.contains("keyboards") {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            let clean = stem.strip_suffix(".mpk").unwrap_or(stem);
                            cache.invalidate_keyboard(clean);
                        } else {
                            cache.invalidate_all_keyboards();
                        }
                    } else if path_str.contains("corpora") {
                        // Corpora keys are complex JSON strings of sources.
                        // Hard to map file -> key. Invalidate all for safety.
                        cache.invalidate_all_corpora();
                    } else if path_str.contains("weights") {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            let clean = stem.strip_suffix(".mpk").unwrap_or(stem);
                            cache.invalidate_cost_model(clean);
                        } else {
                            cache.invalidate_all_cost_models();
                        }
                    } else if path_str.contains("config") {
                        cache.invalidate_all_keycodes();
                    }
                }
            }
        }
    }

    /// Eagerly loads system assets into the memory cache.
    ///
    /// # Errors
    ///
    /// Returns an error string if the manifest cannot be generated or assets cannot be loaded.
    pub async fn warm_all(&self) -> Result<(), String> {
        // SAFETY LIMIT: Don't warm more than 1000 files to prevent OOM
        const MAX_WARM_FILES: usize = 1000;

        info!("🔥 Warming Asset Cache (Parsed Objects Only)...");
        let system_root = self.provider.root().join("system");

        let manifest = crate::net::sync::generate_manifest(&system_root)
            .map_err(|e| format!("Manifest error: {e}"))?;

        self.cache.insert_manifest(Arc::new(manifest.clone()));

        let mut count_files = 0;
        let mut count_keyboards = 0;
        let mut count_corpora = 0;
        let mut count_weights = 0;

        for (rel_path, _) in manifest.files.iter().take(MAX_WARM_FILES) {
            count_files += 1;

            if self.try_ensure_keyboard(rel_path).await? {
                count_keyboards += 1;
            } else if self.try_ensure_corpus(rel_path).await? {
                count_corpora += 1;
            } else if self.try_ensure_weights(rel_path).await? {
                count_weights += 1;
            } else if self.try_ensure_config(rel_path).await? {
                // Handled
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
    pub async fn get_file_content(&self, path: &str) -> Option<Bytes> {
        if let Some(bytes) = self.cache.get_file(path) {
            return Some(bytes);
        }

        let system_root = self.provider.root().join("system");
        let full_path = system_root.join(path);

        match tokio::fs::read(&full_path).await {
            Ok(data) => {
                let bytes = Bytes::from(data);
                self.cache.insert_file(path.to_string(), bytes.clone());
                Some(bytes)
            }
            Err(e) => {
                tracing::debug!("Cache miss & Disk read failed for {}: {}", path, e);
                None
            }
        }
    }

    /// Returns the current system asset manifest.
    #[must_use]
    pub fn get_manifest(&self) -> Option<Arc<ServerManifest>> {
        self.cache.get_manifest()
    }

    /// Purges all cached items from memory.
    pub fn invalidate_all(&self) {
        self.cache.invalidate_all();
    }

    /// Calculates a stable hash for a corpus, using the underlying `FsProvider`.
    /// Retrieves the hash of a corpus by its identifier.
    ///
    /// # Errors
    ///
    /// Returns `InfraError` if the hash cannot be retrieved.
    pub async fn get_corpus_hash(&self, id: &str) -> LoaderResult<String> {
        self.provider.get_corpus_hash(id).await
    }

    async fn try_ensure_keyboard(&self, rel_path: &str) -> Result<bool, String> {
        if rel_path.starts_with(crate::asset::ASSET_PATH_KEYBOARDS) {
            let path = Path::new(rel_path);
            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            let clean_stem = if let Some(s) = stem.strip_suffix(".mpk") {
                s
            } else {
                stem
            };
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
        if rel_path.starts_with(crate::asset::ASSET_PATH_CORPORA)
            && rel_path.ends_with("1grams.mpk.zst")
        {
            let path = Path::new(rel_path);
            if let Some(parent) = path.parent() {
                if let Ok(id_path) = parent.strip_prefix("corpora") {
                    let id = id_path.to_string_lossy().replace('\\', "/");
                    if !id.is_empty() {
                        if let Err(e) = self
                            .load_corpus(&[CorpusSource {
                                id: id.clone(),
                                weight: 1.0,
                                hash: None,
                            }])
                            .await
                        {
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
            let clean_stem = if let Some(s) = stem.strip_suffix(".mpk") {
                s
            } else {
                stem
            };

            if !clean_stem.is_empty() {
                let _ = self.load::<CostModel>(clean_stem).await;
                return Ok(true);
            }
        }
        Ok(false)
    }

    async fn try_ensure_config(&self, rel_path: &str) -> Result<bool, String> {
        if rel_path == format!("config/{ASSET_KEYCODES}.mpk.zst") {
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
        let system_root = self.provider.root().join("system");
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
            let kb = if let Some(c) = self.cache.get_keyboard(id) {
                c
            } else {
                let kb = self.provider.load::<KeyboardDefinition>(id).await?;
                self.cache.insert_keyboard(id.to_string(), kb.clone());
                kb
            };
            let any_kb: Arc<dyn Any + Send + Sync> = kb;
            return any_kb
                .downcast::<T>()
                .map_err(|_| ForgeError::Internal("Downcast failed".into()));
        }

        if tid == TypeId::of::<CostModel>() {
            let cm = if let Some(c) = self.cache.get_cost_model(id) {
                c
            } else {
                let cm = self.provider.load::<CostModel>(id).await?;
                self.cache.insert_cost_model(id.to_string(), cm.clone());
                cm
            };
            let any_cm: Arc<dyn Any + Send + Sync> = cm;
            return any_cm
                .downcast::<T>()
                .map_err(|_| ForgeError::Internal("Downcast failed".into()));
        }

        if tid == TypeId::of::<KeycodeRegistry>() {
            let rg = if let Some(c) = self.cache.get_keycodes(id) {
                c
            } else {
                let rg = self.provider.load::<KeycodeRegistry>(id).await?;
                self.cache.insert_keycodes(id.to_string(), rg.clone());
                rg
            };
            let any_rg: Arc<dyn Any + Send + Sync> = rg;
            return any_rg
                .downcast::<T>()
                .map_err(|_| ForgeError::Internal("Downcast failed".into()));
        }

        // Fallback for types without dedicated caches
        self.provider.load::<T>(id).await
    }

    async fn load_corpus(&self, sources: &[CorpusSource]) -> LoaderResult<Arc<Corpus>> {
        let key = serde_json::to_string(sources).unwrap_or_default();
        if let Some(c) = self.cache.get_corpus(&key) {
            return Ok(c);
        }
        let cp = self.provider.load_corpus(sources).await?;
        self.cache.insert_corpus(key, cp.clone());
        Ok(cp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use tempfile::TempDir;

    fn setup_env() -> (TempDir, CachingProvider) {
        let temp = tempfile::tempdir().unwrap();
        let provider = CachingProvider::new(temp.path().to_path_buf());
        (temp, provider)
    }

    #[tokio::test]
    async fn test_caching_provider_basic_caching() {
        let (temp, provider) = setup_env();
        let root = temp.path();
        
        let kb_dir = root.join("user/keyboards");
        fs::create_dir_all(&kb_dir).unwrap();
        
        let kb_json = r#"{
            "meta": { "name": "CacheTest" },
            "geometry": { "keys": [{"x":0,"y":0,"hand":0,"finger":1,"row":0}], "prime_slots":[0], "med_slots":[], "low_slots":[], "home_row": 0 }
        }"#;
        let kb_path = kb_dir.join("test.json");
        fs::write(&kb_path, kb_json).unwrap();

        // 1. Initial load (Miss)
        let res1: Arc<KeyboardDefinition> = provider.load("test").await.unwrap();
        assert_eq!(res1.meta.name, "CacheTest");

        // 2. Modify file on disk
        let kb_json_v2 = kb_json.replace("CacheTest", "Updated");
        fs::write(&kb_path, kb_json_v2).unwrap();

        // 3. Second load (Hit) - Should still have old name
        let res2: Arc<KeyboardDefinition> = provider.load("test").await.unwrap();
        assert_eq!(res2.meta.name, "CacheTest");

        // 4. Invalidate manually
        provider.invalidate_all();
        
        // 5. Third load (Miss) - Should have new name
        let res3: Arc<KeyboardDefinition> = provider.load("test").await.unwrap();
        assert_eq!(res3.meta.name, "Updated");
    }

    #[tokio::test]
    async fn test_caching_provider_warming() {
        let (temp, provider) = setup_env();
        let root = temp.path();
        
        let sys_kb_dir = root.join("system/keyboards/models");
        fs::create_dir_all(&sys_kb_dir).unwrap();
        
        // Create a valid-ish empty zstd-compressed MessagePack file
        let path = sys_kb_dir.join("sys1.mpk.zst");
        let mut kb = KeyboardDefinition::default();
        kb.geometry.keys.push(keyforge_model::geometry::KeyNode::default());
        kb.geometry.prime_slots.push(keyforge_model::types::KeyIndex(0));
        kb.geometry.home_row = 0; // Match KeyNode::default() row
        
        {
            let file = File::create(&path).unwrap();
            let mut encoder = zstd::Encoder::new(file, 3).unwrap();
            rmp_serde::encode::write(&mut encoder, &kb).unwrap();
            encoder.finish().unwrap();
        }

        provider.warm_all().await.unwrap();
        
        // Should be in cache now
        assert!(provider.cache.get_keyboard("sys1").is_some());
    }

    #[tokio::test]
    async fn test_caching_provider_file_caching() {
        let (temp, provider) = setup_env();
        let root = temp.path();
        
        let sys_dir = root.join("system");
        fs::create_dir_all(&sys_dir).unwrap();
        fs::write(sys_dir.join("raw.txt"), "raw content").unwrap();

        // 1. Load (Miss)
        let content1 = provider.get_file_content("raw.txt").await.unwrap();
        assert_eq!(content1, "raw content");

        // 2. Update disk
        fs::write(sys_dir.join("raw.txt"), "new content").unwrap();

        // 3. Load (Hit)
        let content2 = provider.get_file_content("raw.txt").await.unwrap();
        assert_eq!(content2, "raw content");
    }

    #[test]
    fn test_caching_provider_event_handling() {
        let (temp, provider) = setup_env();
        let root = temp.path();
        
        // Manually trigger invalidation event
        let event = Event::new(notify::EventKind::Modify(notify::event::ModifyKind::Data(notify::event::DataChange::Content)))
            .add_path(root.join("system/keyboards/test.mpk"));
            
        // Populate cache first
        provider.cache.insert_keyboard("test".into(), Arc::new(KeyboardDefinition::default()));
        assert!(provider.cache.get_keyboard("test").is_some());

        CachingProvider::handle_fs_event(event, root, &provider.cache);
        
        // Should be invalidated
        assert!(provider.cache.get_keyboard("test").is_none());
    }
}
