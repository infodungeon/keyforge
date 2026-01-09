// libs/keyforge-infra/src/asset/fs_provider.rs

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


use crate::util::corpus::inject_synthetic_data;
use keyforge_model::error::ForgeError;
use keyforge_core::loader::{AssetLoader, LoaderResult, RawCostData};
use keyforge_model::Corpus;
use keyforge_model::config::CorpusSource;
use keyforge_model::constants::MAX_INPUT_FILE_SIZE;
use keyforge_model::geometry::KeyboardDefinition;
use keyforge_model::keycodes::KeycodeRegistry;
use sha2::Digest;
use keyforge_model::validator::Validator;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// An asset provider that loads data directly from the local filesystem.
///
/// It supports loading both system-level assets (stored in zstd-compressed MessagePack)
/// and user-level assets (stored as plain JSON).
#[derive(Clone)]
pub struct FsProvider {
    /// The root directory where all assets (system and user) are located.
    pub root: PathBuf,
}

impl FsProvider {
    /// Creates a new `FsProvider` with the specified root path.
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    async fn check_size(&self, path: &Path) -> LoaderResult<()> {
        let meta = tokio::fs::metadata(path).await?;
        if meta.len() > MAX_INPUT_FILE_SIZE {
            return Err(ForgeError::InvalidData(format!(
                "File {:?} exceeds size limit of {} bytes",
                path, MAX_INPUT_FILE_SIZE
            )));
        }
        Ok(())
    }

    async fn load_binary<T: serde::de::DeserializeOwned + Send + 'static>(&self, path: &Path) -> LoaderResult<T> {
        self.check_size(path).await?;
        let path = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let file = File::open(&path)?;
            let decoder = zstd::Decoder::new(file).map_err(|e| ForgeError::Internal(e.to_string()))?;
            rmp_serde::from_read(decoder).map_err(|e| ForgeError::Internal(e.to_string()))
        })
        .await
        .map_err(|e| ForgeError::Internal(e.to_string()))?
    }

    async fn load_json<T: serde::de::DeserializeOwned + Send + 'static>(&self, path: &Path) -> LoaderResult<T> {
        self.check_size(path).await?;
        let content = tokio::fs::read_to_string(path).await?;
        tokio::task::spawn_blocking(move || {
            serde_json::from_str(&content).map_err(ForgeError::Serde)
        })
        .await
        .map_err(|e| ForgeError::Internal(e.to_string()))?
    }

    fn resolve_system_path(&self, category: &str, stem: &str) -> Option<PathBuf> {
        let sub = match category {
            "keyboards" => "keyboards/models",
            "weights" => "weights",
            "config" => "config",
            "keymap_extras" => "keymap_extras",
            _ => category,
        };

        let p = self.root.join("system").join(sub).join(format!("{}.mpk.zst", stem));
        if p.exists() { return Some(p); }

        let p_direct = self.root.join("system").join(category).join(format!("{}.mpk.zst", stem));
        if p_direct.exists() { return Some(p_direct); }

        None
    }

    fn resolve_user_path(&self, category: &str, stem: &str) -> Option<PathBuf> {
        let p = self.root.join("user").join(category).join(format!("{}.json", stem));
        p.exists().then_some(p)
    }

    /// Calculates a stable hash for a corpus by hashing its constituent parts.
    ///
    /// This is used to detect if a corpus has changed locally or if it matches
    /// the version expected by the Hive.
    pub async fn get_corpus_hash(&self, id: &str) -> LoaderResult<String> {
        let files = ["1grams", "2grams", "3grams", "words"];
        let is_system = self.root.join("system/corpora").join(id).exists();
        let base = if is_system {
            self.root.join("system/corpora").join(id)
        } else {
            self.root.join("user/corpora").join(id)
        };
        let ext = if is_system { "mpk.zst" } else { "json" };
        
        let mut hasher = sha2::Sha256::new();
        for f in files {
            let path = base.join(format!("{}.{}", f, ext));
            if path.exists() {
                let content = tokio::fs::read(&path).await?;
                hasher.update(&content);
            }
        }
        Ok(hex::encode(hasher.finalize()))
    }
}

#[derive(serde::Deserialize)]
struct CostEntry {
    #[serde(alias = "from")]
    from_key: String,
    #[serde(alias = "to")]
    to_key: String,
    #[serde(alias = "cost")]
    cost_ms: f32,
}

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum CostFormat {
    Wrapped { entries: Vec<CostEntry> },
    Direct(Vec<CostEntry>),
}

#[async_trait::async_trait]
impl AssetLoader for FsProvider {
    async fn load_keyboard(&self, name: &str) -> LoaderResult<Arc<KeyboardDefinition>> {
        let stem = name.strip_suffix(".json").unwrap_or(name);
        if let Some(p) = self.resolve_system_path("keyboards", stem) {
            let kb: KeyboardDefinition = self.load_binary(&p).await?;
            kb.validate().map_err(|e| ForgeError::InvalidData(format!("Invalid system keyboard '{}': {}", name, e)))?;
            return Ok(Arc::new(kb));
        }
        if let Some(p) = self.resolve_user_path("keyboards", stem) {
            let kb: KeyboardDefinition = self.load_json(&p).await?;
            kb.validate().map_err(|e| ForgeError::InvalidData(format!("Invalid user keyboard '{}': {}", name, e)))?;
            return Ok(Arc::new(kb));
        }
        Err(ForgeError::NotFound(name.to_string()))
    }

    async fn load_corpus(&self, sources: &[CorpusSource]) -> LoaderResult<Arc<Corpus>> {
        let mut corpus = Corpus::default();
        for src in sources {
            let is_system = self.root.join("system/corpora").join(&src.id).exists();
            let base = if is_system {
                self.root.join("system/corpora").join(&src.id)
            } else {
                self.root.join("user/corpora").join(&src.id)
            };
            let ext = if is_system { "mpk.zst" } else { "json" };

            let mut segments = Vec::new();
            for stem in ["1grams", "2grams", "3grams", "words"] {
                let p = base.join(format!("{}.{}", stem, ext));
                if p.exists() {
                    let part: Vec<serde_json::Value> = if is_system {
                        self.load_binary(&p).await?
                    } else {
                        self.load_json(&p).await?
                    };
                    segments.push((stem, part));
                }
            }

            crate::util::corpus::populate_corpus_from_segments(&mut corpus, src.weight, segments)?;
        }

        let is_std = sources.iter().any(|s| s.id.contains("_std"));
        inject_synthetic_data(&mut corpus, is_std);

        corpus.validate().map_err(|e| ForgeError::InvalidData(format!("Invalid corpus: {}", e)))?;
        Ok(Arc::new(corpus))
    }

    async fn load_cost_matrix(&self, filename: &str) -> LoaderResult<Arc<RawCostData>> {
        let stem = filename.strip_suffix(".json").unwrap_or(filename);
        if let Some(p) = self.resolve_system_path("weights", stem) {
            let data: RawCostData = self.load_binary(&p).await?;
            return Ok(Arc::new(data));
        }
        if let Some(p) = self.resolve_user_path("weights", stem) {
            let format: CostFormat = self.load_json(&p).await?;
            let entries = match format {
                CostFormat::Wrapped { entries } => entries,
                CostFormat::Direct(v) => v,
            };
            return Ok(Arc::new(RawCostData {
                entries: entries
                    .into_iter()
                    .map(|e| keyforge_core::loader::CostEntry {
                        from: e.from_key,
                        to: e.to_key,
                        cost: e.cost_ms,
                    })
                    .collect(),
            }));
        }
        Err(ForgeError::NotFound(filename.to_string()))
    }

    async fn load_keycodes(&self, filename: &str) -> LoaderResult<Arc<KeycodeRegistry>> {
        let stem = filename.strip_suffix(".json").unwrap_or(filename);
        if let Some(p) = self.resolve_system_path("config", stem) {
            let defs = self.load_binary(&p).await?;
            let reg = KeycodeRegistry::new(defs);
            reg.validate().map_err(|e| ForgeError::InvalidData(format!("Invalid system keycodes: {}", e)))?;
            return Ok(Arc::new(reg));
        }
        let p = self.resolve_user_path("config", stem).ok_or(ForgeError::NotFound(filename.to_string()))?;
        let defs = self.load_json(&p).await?;
        let reg = KeycodeRegistry::new(defs);
        reg.validate().map_err(|e| ForgeError::InvalidData(format!("Invalid user keycodes: {}", e)))?;
        Ok(Arc::new(reg))
    }
}
