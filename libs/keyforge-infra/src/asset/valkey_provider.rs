// libs/keyforge-infra/src/asset/valkey_provider.rs

use crate::net::distributed::DistributedCoordinator;
use crate::util::corpus::{inject_synthetic_data, resolve_corpus_char};
use crate::net::sync::ServerManifest;
use keyforge_core::loader::{AssetLoader, LoaderResult, RawCostData};
use keyforge_model::Corpus;
use keyforge_model::config::CorpusSource;
use keyforge_model::geometry::KeyboardDefinition;
use keyforge_model::keycodes::KeycodeRegistry;
use keyforge_model::Validator;
use keyforge_model::error::ForgeError;
use std::sync::Arc;
use tracing::warn;

const ASSET_PREFIX: &str = "asset:blob";

#[derive(Clone)]
pub struct ValkeyProvider {
    coordinator: Arc<DistributedCoordinator>,
}

impl ValkeyProvider {
    pub fn new(coordinator: Arc<DistributedCoordinator>) -> Self {
        Self { coordinator }
    }

    pub async fn get_manifest(&self) -> ServerManifest {
        match self.coordinator.get_all_manifest_entries().await {
            Ok(files) => ServerManifest { files },
            Err(e) => {
                warn!("Failed to fetch manifest from Valkey: {}", e);
                ServerManifest { files: std::collections::HashMap::new() }
            }
        }
    }

    // No-op for stateless provider
    pub fn invalidate_all(&self) {}

    pub async fn get_corpus_hash(&self, id: &str) -> LoaderResult<String> {
        let key = format!("corpora/{}/1grams.mpk.zst", id);
        match self.coordinator.get_manifest_hash(&key).await {
            Ok(Some(h)) => Ok(h),
            _ => Err(ForgeError::NotFound(id.to_string()))
        }
    }

    async fn fetch_blob(&self, subpath: &str) -> LoaderResult<Vec<u8>> {
        let key = format!("{}:{}", ASSET_PREFIX, subpath);
        let data = self.coordinator.get_bin(&key).await.map_err(|e| {
            ForgeError::Internal(format!("Valkey Fetch Error: {}", e))
        })?;

        data.map(|b| b.to_vec()).ok_or_else(|| {
            ForgeError::NotFound(subpath.to_string())
        })
    }

    async fn hydrate_mpk<T: serde::de::DeserializeOwned + Send + 'static>(&self, subpath: &str) -> LoaderResult<T> {
        let compressed = self.fetch_blob(subpath).await?;
        
        tokio::task::spawn_blocking(move || {
            let cursor = std::io::Cursor::new(compressed);
            let decoder = zstd::Decoder::new(cursor)
                .map_err(|e| ForgeError::Internal(format!("Zstd Init Error: {}", e)))?;
            rmp_serde::from_read(decoder)
                .map_err(|e| ForgeError::Internal(format!("Deserialization Error: {}", e)))
        }).await.map_err(|e| ForgeError::Internal(e.to_string()))?
    }

    // --- Helper Methods for Hive ---

    pub async fn get_file_content(&self, path: &str) -> Option<bytes::Bytes> {
        let key = format!("{}:{}", ASSET_PREFIX, path);
        self.coordinator.get_bin(&key).await.unwrap_or(None)
    }

    pub async fn list_keyboards(&self) -> Vec<String> {
        let pattern = format!("{}:keyboards/models/*.mpk.zst", ASSET_PREFIX);
        let keys = self.coordinator.scan_keys(&pattern).await.unwrap_or_default();
        
        let mut names = Vec::new();
        for k in keys {
            if let Some(stem) = k.split('/').last() {
                if let Some(name) = stem.strip_suffix(".mpk.zst") {
                    names.push(name.to_string());
                }
            }
        }
        names.sort();
        names
    }

    pub async fn list_corpora(&self) -> Vec<String> {
        let pattern = format!("{}:corpora/*", ASSET_PREFIX);
        let keys = self.coordinator.scan_keys(&pattern).await.unwrap_or_default();
        
        let mut ids = Vec::new();
        for k in keys {
            if k.contains("1grams.mpk.zst") {
                if let Some(start) = k.find("corpora/") {
                    let sub = &k[start + 8..]; 
                    if let Some(end) = sub.find("/1grams") {
                        ids.push(sub[..end].to_string());
                    }
                }
            }
        }
        ids.sort();
        ids.dedup();
        ids
    }

    pub async fn list_cost_matrices(&self) -> Vec<String> {
        let pattern = format!("{}:weights/*.mpk.zst", ASSET_PREFIX);
        let keys = self.coordinator.scan_keys(&pattern).await.unwrap_or_default();
        let mut names = Vec::new();
        for k in keys {
            if let Some(stem) = k.split('/').last() {
                if let Some(name) = stem.strip_suffix(".mpk.zst") {
                    names.push(name.to_string());
                }
            }
        }
        names.sort();
        names
    }

    pub async fn load_config_asset<T: serde::de::DeserializeOwned + Send + 'static + Default>(&self, name: &str) -> Arc<T> {
        let mpk_path = format!("config/{}.mpk.zst", name);
        if let Ok(cfg) = self.hydrate_mpk::<T>(&mpk_path).await {
            return Arc::new(cfg);
        }
        let json_key = format!("{}:config/{}.json", ASSET_PREFIX, name);
        if let Ok(Some(bytes)) = self.coordinator.get_bin(&json_key).await {
             if let Ok(cfg) = serde_json::from_slice(&bytes) {
                 return Arc::new(cfg);
             }
        }
        Arc::new(T::default())
    }
}

#[async_trait::async_trait]
impl AssetLoader for ValkeyProvider {
    async fn load_keyboard(&self, name: &str) -> LoaderResult<KeyboardDefinition> {
        let path = format!("keyboards/models/{}.mpk.zst", name);
        let kb: KeyboardDefinition = self.hydrate_mpk(&path).await?;
        kb.validate().map_err(|e| ForgeError::InvalidData(e))?;
        Ok(kb)
    }

    async fn load_cost_matrix(&self, filename: &str) -> LoaderResult<RawCostData> {
        let stem = filename.strip_suffix(".json").unwrap_or(filename);
        let path = format!("weights/{}.mpk.zst", stem);
        self.hydrate_mpk(&path).await
    }

    async fn load_keycodes(&self, filename: &str) -> LoaderResult<KeycodeRegistry> {
        let stem = filename.strip_suffix(".json").unwrap_or(filename);
        let path = format!("config/{}.mpk.zst", stem);
        let reg: KeycodeRegistry = self.hydrate_mpk(&path).await?;
        reg.validate().map_err(|e| ForgeError::InvalidData(e))?;
        Ok(reg)
    }

    async fn load_corpus(&self, sources: &[CorpusSource]) -> LoaderResult<Corpus> {
        let mut corpus = Corpus::default();
        
        for src in sources {
            let base = format!("corpora/{}", src.id);
            let parts = ["1grams", "2grams", "3grams", "words"];
            
            let mut segments = Vec::new();

            for part_name in parts {
                let path = format!("{}/{}.mpk.zst", base, part_name);
                if let Ok(vec) = self.fetch_blob(&path).await {
                    let part_res = tokio::task::spawn_blocking(move || {
                        let cursor = std::io::Cursor::new(vec);
                        let decoder = zstd::Decoder::new(cursor).map_err(|e| ForgeError::Internal(e.to_string()))?;
                        let data: Vec<serde_json::Value> = rmp_serde::from_read(decoder).map_err(|e| ForgeError::Internal(e.to_string()))?;
                        Ok::<Vec<serde_json::Value>, ForgeError>(data)
                    }).await.map_err(|e| ForgeError::Internal(e.to_string()))??;
                    
                    segments.push((part_name, part_res));
                }
            }

            for (stem, part) in segments {
                match stem {
                    "1grams" => {
                        for e in part {
                            if let Some(c) = e["char"].as_str().and_then(resolve_corpus_char) {
                                if (c as usize) < 65536 {
                                    corpus.char_freqs[c as usize] +=
                                        (e["freq"].as_u64().unwrap_or(0) as f32 * src.weight).round() as u64;
                                }
                            }
                        }
                    }
                    "2grams" => {
                        for e in part {
                            let c1 = e["char1"].as_str().and_then(resolve_corpus_char).unwrap_or('\0') as u16;
                            let c2 = e["char2"].as_str().and_then(resolve_corpus_char).unwrap_or('\0') as u16;
                            corpus.bigrams.push((c1, c2, (e["freq"].as_u64().unwrap_or(0) as f32 * src.weight).round() as u32));
                        }
                    }
                    "3grams" => {
                        for e in part {
                            let c1 = e["char1"].as_str().and_then(resolve_corpus_char).unwrap_or('\0') as u16;
                            let c2 = e["char2"].as_str().and_then(resolve_corpus_char).unwrap_or('\0') as u16;
                            let c3 = e["char3"].as_str().and_then(resolve_corpus_char).unwrap_or('\0') as u16;
                            corpus.trigrams.push((c1, c2, c3, (e["freq"].as_u64().unwrap_or(0) as f32 * src.weight).round() as u32));
                        }
                    }
                    "words" => {
                        for e in part {
                            if let Some(w) = e["word"].as_str() {
                                corpus.words.push((w.to_string(), (e["freq"].as_u64().unwrap_or(0) as f32 * src.weight).round() as u32));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        let is_std = sources.iter().any(|s| s.id.contains("_std"));
        inject_synthetic_data(&mut corpus, is_std);

        corpus.validate().map_err(|e| ForgeError::InvalidData(format!("Invalid corpus: {}", e)))?;
        Ok(corpus)
    }
}
