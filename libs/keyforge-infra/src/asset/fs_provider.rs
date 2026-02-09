// libs/keyforge-infra/src/asset/fs_provider.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You    may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::asset::AssetServerProvider;
use crate::error::InfraResult;
use crate::net::sync::ServerManifest;
use async_trait::async_trait;
use keyforge_adapter::loader::{AssetLoader, LoaderResult};
use keyforge_model::Asset;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;

/// Local filesystem asset provider.
///
/// Implements `AssetLoader` for local file access and `AssetServerProvider`
/// for serving assets over HTTP.
#[derive(Debug, Clone)]
pub struct FsProvider {
    root: PathBuf,
}

impl FsProvider {
    /// Creates a new `FsProvider` with the specified root directory.
    #[must_use]
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    fn resolve_path(&self, id: &str) -> LoaderResult<PathBuf> {
        let path = self.root.join(id);
        if path.exists() {
            Ok(path)
        } else {
            // Try extensions
            let json = path.with_extension("json");
            if json.exists() {
                return Ok(json);
            }
            let mpk = path.with_extension("mpk");
            if mpk.exists() {
                return Ok(mpk);
            }
            let mpk_zst = path.with_extension("mpk.zst");
            if mpk_zst.exists() {
                return Ok(mpk_zst);
            }

            Err(keyforge_model::error::ForgeError::NotFound(id.to_string()))
        }
    }
}

#[async_trait]
impl AssetLoader for FsProvider {
    async fn load<T: Asset>(&self, id: &str) -> LoaderResult<Arc<T>> {
        let path = self.resolve_path(id)?;
        let content = fs::read(&path).await.map_err(|e| {
            keyforge_model::error::ForgeError::Io(format!("Failed to read {id}: {e}"))
        })?;

        let final_id = path.to_string_lossy().to_string();

        if final_id.to_lowercase().ends_with(".zst")
            || final_id.to_lowercase().ends_with(".mpk.zst")
        {
            let decoder = zstd::Decoder::new(&content[..]).map_err(|e| {
                keyforge_model::error::ForgeError::Io(format!("Zstd decoder error: {e}"))
            })?;
            let asset: T = rmp_serde::from_read(decoder).map_err(|e| {
                keyforge_model::error::ForgeError::Serde(format!("MsgPack decode error: {e}"))
            })?;
            return Ok(Arc::new(asset));
        }

        if final_id.to_lowercase().ends_with(".json") {
            let asset: T = serde_json::from_slice(&content).map_err(|e| {
                keyforge_model::error::ForgeError::Serde(format!("JSON decode error: {e}"))
            })?;
            return Ok(Arc::new(asset));
        }

        Err(keyforge_model::error::ForgeError::Serialization(format!(
            "Unsupported asset format for {id}"
        )))
    }

    async fn load_corpus(
        &self,
        sources: &[keyforge_model::config::CorpusSource],
    ) -> LoaderResult<Arc<keyforge_model::Corpus>> {
        let mut corpus = keyforge_model::Corpus::default();
        for source in sources {
            let part = self.load::<keyforge_model::Corpus>(&source.id).await?;
            corpus.merge(&part, source.weight);
        }
        Ok(Arc::new(corpus))
    }

    async fn get_hash(&self, _category: keyforge_model::AssetCategory, id: &str) -> LoaderResult<String> {
        let path = self.resolve_path(id)?;
        crate::util::common::calculate_file_hash(&path)
            .map_err(|e| keyforge_model::error::ForgeError::Io(e.to_string()))
    }

    fn root(&self) -> &Path {
        &self.root
    }
}

#[async_trait]
impl AssetServerProvider for FsProvider {
    async fn get_manifest(&self) -> InfraResult<ServerManifest> {
        // FsProvider doesn't currently support generating a full manifest on the fly.
        // It returns an empty manifest.
        Ok(ServerManifest {
            files: HashMap::new(),
        })
    }

    async fn get_file_content(&self, path: &str) -> InfraResult<Option<bytes::Bytes>> {
        let full_path = self.root.join(path);
        if !full_path.exists() || !full_path.is_file() {
            return Ok(None);
        }

        let content = fs::read(full_path).await?;
        Ok(Some(bytes::Bytes::from(content)))
    }
}
