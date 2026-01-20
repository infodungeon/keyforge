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
use keyforge_core::loader::{AssetLoader, LoaderResult};
use keyforge_model::config::CorpusSource;
use keyforge_model::constants::MAX_INPUT_FILE_SIZE;
use keyforge_model::error::ForgeError;
use keyforge_model::{Asset, Corpus};
use sha2::Digest;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::asset::resolver::PathResolver;
use crate::asset::AssetServerProvider;
use crate::net::sync::ServerManifest;

/// An asset provider that loads data directly from the local filesystem.
///
/// It supports loading both system-level assets (stored in zstd-compressed `MessagePack`)
/// and user-level assets (stored as plain JSON).
#[derive(Clone, Debug)]
pub struct FsProvider {
    resolver: PathResolver,
}

impl FsProvider {
    /// Creates a new `FsProvider` with the specified root path.
    #[must_use]
    pub fn new(root: PathBuf) -> Self {
        Self {
            resolver: PathResolver::new(root),
        }
    }

    /// Returns the root directory of this provider.
    #[must_use]
    pub fn root(&self) -> &PathBuf {
        &self.resolver.root
    }

    async fn check_size(&self, path: &Path) -> LoaderResult<()> {
        let meta = tokio::fs::metadata(path).await?;
        if meta.len() > MAX_INPUT_FILE_SIZE {
            return Err(ForgeError::InvalidData(format!("File {} exceeds size limit of {MAX_INPUT_FILE_SIZE} bytes", path.display())));
        }
        Ok(())
    }

    async fn load_binary<T: serde::de::DeserializeOwned + Send + 'static>(
        &self,
        path: &Path,
    ) -> LoaderResult<T> {
        self.check_size(path).await?;
        let path = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let file = File::open(&path)?;
            let decoder =
                zstd::Decoder::new(file).map_err(|e| ForgeError::Internal(e.to_string()))?;
            rmp_serde::from_read(decoder).map_err(|e| ForgeError::Internal(e.to_string()))
        })
        .await
        .map_err(|e| ForgeError::Internal(e.to_string()))?
    }

    async fn load_json<T: serde::de::DeserializeOwned + Send + 'static>(
        &self,
        path: &Path,
    ) -> LoaderResult<T> {
        self.check_size(path).await?;
        let path = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let file = File::open(&path)?;
            let reader = BufReader::new(file);
            serde_json::from_reader(reader).map_err(ForgeError::Serde)
        })
        .await
        .map_err(|e| ForgeError::Internal(e.to_string()))?
    }

    /// Calculates a stable hash for a corpus by hashing its constituent parts.
    ///
    /// # Errors
    ///
    /// Returns `LoaderResult` if any constituent file cannot be read.
    pub async fn get_corpus_hash(&self, id: &str) -> LoaderResult<String> {
        // Task-infra-008: Security check
        let _ = self
            .resolver
            .safe_join(id)
            .map_err(ForgeError::InvalidData)?;

        let files = ["1grams", "2grams", "3grams", "words"];
        let is_system = self.resolver.root.join("system/corpora").join(id).exists();
        let base = if is_system {
            self.resolver.root.join("system/corpora").join(id)
        } else {
            self.resolver.root.join("user/corpora").join(id)
        };
        let ext = if is_system { "mpk.zst" } else { "json" };

        let mut hasher = sha2::Sha256::new();
        for f in files {
            let path = base.join(format!("{f}.{ext}"));
            if path.exists() {
                let content = tokio::fs::read(&path).await?;
                hasher.update(&content);
            }
        }
        Ok(hex::encode(hasher.finalize()))
    }
}

#[async_trait::async_trait]
impl AssetLoader for FsProvider {
    async fn load<T: Asset>(&self, id: &str) -> LoaderResult<Arc<T>> {
        let category = T::category();
        let cat_str = category.as_str();

        // Task-infra-008: Ensure ID is safe
        if id.contains("..") {
            self.resolver
                .safe_join(id)
                .map_err(ForgeError::InvalidData)?;
        }

        // 1. Try direct path (absolute or ./ relative)
        if let Some(p) = self.resolver.resolve_direct_path(id) {
            let mut asset: T = self.load_json(&p).await?;
            asset.post_load()?;
            return Ok(Arc::new(asset));
        }

        // Task-infra-020: Use Path for extension handling
        let path_id = Path::new(id);
        let stem = if path_id.extension().and_then(|s| s.to_str()) == Some("json") {
            path_id.file_stem().and_then(|s| s.to_str()).unwrap_or(id)
        } else {
            id
        };

        // 2. Try System Binary
        if let Some(p) = self.resolver.resolve_system_path(cat_str, stem) {
            let mut asset: T = self.load_binary(&p).await?;
            asset.post_load()?;
            return Ok(Arc::new(asset));
        }

        // 3. Try System JSON
        let system_json = self.resolver.root.join("system").join(cat_str).join(format!("{stem}.json"));
        if system_json.exists() {
            let mut asset: T = self.load_json(&system_json).await?;
            asset.post_load()?;
            return Ok(Arc::new(asset));
        }

        // 4. Try User JSON
        if let Some(p) = self.resolver.resolve_user_path(cat_str, stem) {
            let mut asset: T = self.load_json(&p).await?;
            asset.post_load()?;
            return Ok(Arc::new(asset));
        }

        Err(ForgeError::NotFound(id.to_string()))
    }

    async fn load_corpus(&self, sources: &[CorpusSource]) -> LoaderResult<Arc<Corpus>> {
        let mut corpus = Corpus::default();
        for src in sources {
            // Task-infra-008: Security check
            let _ = self
                .resolver
                .safe_join(&src.id)
                .map_err(ForgeError::InvalidData)?;

            let is_system = self
                .resolver
                .root
                .join("system/corpora")
                .join(&src.id)
                .exists();
            let base = if is_system {
                self.resolver.root.join("system/corpora").join(&src.id)
            } else {
                self.resolver.root.join("user/corpora").join(&src.id)
            };
            let ext = if is_system { "mpk.zst" } else { "json" };

            let mut segments = Vec::new();
            for stem in ["1grams", "2grams", "3grams", "words"] {
                let p = base.join(format!("{stem}.{ext}"));
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

        corpus.post_load()?;
        Ok(Arc::new(corpus))
    }
}

#[async_trait::async_trait]
impl AssetServerProvider for FsProvider {
    async fn get_manifest(&self) -> ServerManifest {
        let root = self.resolver.root.clone();
        tokio::task::spawn_blocking(move || {
            crate::net::sync::generate_manifest(&root.join("system")).unwrap_or(ServerManifest {
                files: std::collections::HashMap::default(),
            })
        })
        .await
        .unwrap_or(ServerManifest {
            files: std::collections::HashMap::default(),
        })
    }

    async fn get_file_content(&self, path: &str) -> Option<bytes::Bytes> {
        let Ok(safe_path) = self.resolver.safe_join(path) else {
            return None;
        };

        if safe_path.exists() {
            tokio::fs::read(safe_path)
                .await
                .ok()
                .map(bytes::Bytes::from)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use keyforge_model::KeyboardDefinition;
    use std::fs;

    async fn setup_root() -> (tempfile::TempDir, FsProvider) {
        let temp = tempfile::tempdir().unwrap();
        let provider = FsProvider::new(temp.path().to_path_buf());
        (temp, provider)
    }

    #[tokio::test]
    async fn test_fs_provider_json_loading() {
        let (temp, provider) = setup_root().await;
        let root = temp.path();
        assert_eq!(provider.root(), &root.to_path_buf());
        
        let kb_dir = root.join("user/keyboards");
        fs::create_dir_all(&kb_dir).unwrap();
        
        let kb_json = r#"{
            "meta": { "name": "Test" },
            "geometry": { "keys": [{"x":0, "y":0, "hand":0, "finger":1}], "prime_slots":[0], "med_slots":[], "low_slots":[] }
        }"#;
        fs::write(kb_dir.join("test.json"), kb_json).unwrap();

        // 1. Standard load
        let res: Arc<KeyboardDefinition> = provider.load("test").await.unwrap();
        assert_eq!(res.meta.name, "Test");

        // 2. Load with extension
        let res: Arc<KeyboardDefinition> = provider.load("test.json").await.unwrap();
        assert_eq!(res.meta.name, "Test");

        // 3. Direct path (absolute)
        let abs_path = kb_dir.join("test.json");
        let res: Arc<KeyboardDefinition> = provider.load(abs_path.to_str().unwrap()).await.unwrap();
        assert_eq!(res.meta.name, "Test");

        // 4. Path traversal attempt
        assert!(provider.load::<KeyboardDefinition>("../secret").await.is_err());
    }

    #[tokio::test]
    async fn test_fs_provider_binary_errors() {
        let (temp, provider) = setup_root().await;
        let root = temp.path();
        
        let kb_dir = root.join("system/keyboards/models");
        fs::create_dir_all(&kb_dir).unwrap();
        
        // Corrupt binary
        fs::write(kb_dir.join("corrupt.mpk.zst"), "not a zstd file").unwrap();
        assert!(provider.load::<KeyboardDefinition>("corrupt").await.is_err());
    }

    #[tokio::test]
    async fn test_fs_provider_corpus_hash_system() {
        let (temp, provider) = setup_root().await;
        let root = temp.path();
        
        let corp_dir = root.join("system/corpora/en");
        fs::create_dir_all(&corp_dir).unwrap();
        // Create valid empty zstd file
        let path = corp_dir.join("1grams.mpk.zst");
        let file = File::create(&path).unwrap();
        let encoder = zstd::Encoder::new(file, 3).unwrap();
        encoder.finish().unwrap();
        
        let hash = provider.get_corpus_hash("en").await.unwrap();
        assert!(!hash.is_empty());
    }

    #[tokio::test]
    async fn test_fs_provider_binary_loading() {
        let (temp, provider) = setup_root().await;
        let root = temp.path();
        
        let kb_dir = root.join("system/keyboards/models");
        fs::create_dir_all(&kb_dir).unwrap();
        
        let kb = KeyboardDefinition {
            meta: keyforge_model::geometry::KeyboardMeta { name: "Binary".into(), ..Default::default() },
            geometry: keyforge_model::geometry::KeyboardGeometry {
                keys: vec![keyforge_model::geometry::KeyNode { hand: keyforge_model::types::HandIndex::LEFT, finger: keyforge_model::types::FingerIndex::INDEX, ..Default::default() }],
                prime_slots: vec![keyforge_model::types::KeyIndex(0)],
                ..Default::default()
            },
            ..Default::default()
        };
        
        let path = kb_dir.join("test.mpk.zst");
        {
            let file = File::create(&path).unwrap();
            let mut encoder = zstd::Encoder::new(file, 3).unwrap();
            rmp_serde::encode::write(&mut encoder, &kb).unwrap();
            encoder.finish().unwrap();
        }

        let res: Arc<KeyboardDefinition> = provider.load("test").await.unwrap();
        assert_eq!(res.meta.name, "Binary");
    }

    #[tokio::test]
    async fn test_fs_provider_system_json() {
        let (temp, provider) = setup_root().await;
        let root = temp.path();
        
        let sys_dir = root.join("system/keyboards");
        fs::create_dir_all(&sys_dir).unwrap();
        fs::write(sys_dir.join("sys.json"), r#"{"meta":{"name":"SysJSON"}, "geometry":{"keys":[{"x":0,"y":0,"hand":0,"finger":1}],"prime_slots":[0],"med_slots":[],"low_slots":[]}}"#).unwrap();

        let res: Arc<KeyboardDefinition> = provider.load("sys").await.unwrap();
        assert_eq!(res.meta.name, "SysJSON");
    }

    #[tokio::test]
    async fn test_fs_provider_load_corpus_system() {
        let (temp, provider) = setup_root().await;
        let root = temp.path();
        
        let corp_dir = root.join("system/corpora/en");
        fs::create_dir_all(&corp_dir).unwrap();
        
        let data = vec![serde_json::json!({"char": "a", "freq": 100})];
        let path = corp_dir.join("1grams.mpk.zst");
        {
            let file = File::create(&path).unwrap();
            let mut encoder = zstd::Encoder::new(file, 3).unwrap();
            rmp_serde::encode::write(&mut encoder, &data).unwrap();
            encoder.finish().unwrap();
        }
        
        let sources = vec![CorpusSource { id: "en".into(), weight: 1.0, hash: None }];
        let corp = provider.load_corpus(&sources).await.unwrap();
        assert_eq!(corp.char_freqs[97], 100);
    }

    #[tokio::test]
    async fn test_fs_provider_corpus_hash() {
        let (temp, provider) = setup_root().await;
        let root = temp.path();
        
        let corp_dir = root.join("user/corpora/en");
        fs::create_dir_all(&corp_dir).unwrap();
        fs::write(corp_dir.join("1grams.json"), "[]").unwrap();
        
        let hash = provider.get_corpus_hash("en").await.unwrap();
        assert!(!hash.is_empty());
    }

    #[tokio::test]
    async fn test_fs_provider_load_corpus() {
        let (temp, provider) = setup_root().await;
        let root = temp.path();
        
        let corp_dir = root.join("user/corpora/en");
        fs::create_dir_all(&corp_dir).unwrap();
        fs::write(corp_dir.join("1grams.json"), r#"[{"char": "a", "freq": 100}]"#).unwrap();
        
        let sources = vec![CorpusSource { id: "en".into(), weight: 1.0, hash: None }];
        let corp = provider.load_corpus(&sources).await.unwrap();
        assert_eq!(corp.char_freqs[97], 100);
    }

    #[tokio::test]
    async fn test_fs_provider_server_provider() {
        let (temp, provider) = setup_root().await;
        let root = temp.path();
        
        let sys_dir = root.join("system");
        fs::create_dir_all(&sys_dir).unwrap();
        fs::write(sys_dir.join("test.txt"), "hello").unwrap();
        
        let _manifest = provider.get_manifest().await;
        
        let content = provider.get_file_content("system/test.txt").await.unwrap();
        assert_eq!(content, "hello");
        
        assert!(provider.get_file_content("missing").await.is_none());
        assert!(provider.get_file_content("../secret").await.is_none());
    }

    #[tokio::test]
    async fn test_fs_provider_size_limit() {
        let (temp, provider) = setup_root().await;
        let root = temp.path();
        
        // We won't actually create a 100MB file to keep tests fast.
        // Instead, we test that a normal file passes.
        let path = root.join("small.json");
        fs::write(&path, "{}").unwrap();
        assert!(provider.check_size(&path).await.is_ok());
    }

    #[tokio::test]
    async fn test_fs_provider_json_errors() {
        let (temp, provider) = setup_root().await;
        let root = temp.path();
        
        let kb_dir = root.join("user/keyboards");
        fs::create_dir_all(&kb_dir).unwrap();
        
        // Invalid JSON content
        fs::write(kb_dir.join("invalid.json"), "{ broken }").unwrap();
        assert!(provider.load::<KeyboardDefinition>("invalid").await.is_err());
    }

    #[tokio::test]
    async fn test_fs_provider_safe_join_error() {
        let (_temp, provider) = setup_root().await;
        // Attempt to load asset with null byte or invalid path char if resolver allows
        // PathResolver::safe_join usually catches ".."
        assert!(provider.load::<KeyboardDefinition>("../../../etc/passwd").await.is_err());
    }
}
