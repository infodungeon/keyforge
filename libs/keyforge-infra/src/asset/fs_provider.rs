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
use keyforge_compute::loader::{AssetLoader, LoaderResult};
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
use crate::error::{InfraError, InfraResult};
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
            return Err(ForgeError::InvalidData(format!(
                "File {} exceeds size limit of {MAX_INPUT_FILE_SIZE} bytes",
                path.display()
            )));
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
        let system_json = self
            .resolver
            .root
            .join("system")
            .join(cat_str)
            .join(format!("{stem}.json"));
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

        let is_std = corpus.meta.is_std;
        inject_synthetic_data(&mut corpus, is_std);

        corpus.post_load()?;
        Ok(Arc::new(corpus))
    }
}

#[async_trait::async_trait]
impl AssetServerProvider for FsProvider {
    async fn get_manifest(&self) -> InfraResult<ServerManifest> {
        let root = self.resolver.root.clone();
        tokio::task::spawn_blocking(move || {
            crate::net::sync::generate_manifest(&root.join("system"))
        })
        .await
        .map_err(|e| InfraError::Io(std::io::Error::other(e)))?
    }

    async fn get_file_content(&self, path: &str) -> InfraResult<Option<bytes::Bytes>> {
        let safe_path = self.resolver.safe_join(path).map_err(InfraError::Config)?;

        if safe_path.exists() {
            let content = tokio::fs::read(safe_path).await?;
            Ok(Some(bytes::Bytes::from(content)))
        } else {
            Ok(None)
        }
    }
}
