// libs/keyforge-infra/src/asset/fs_provider.rs

use crate::asset::AssetServerProvider;
use crate::error::InfraResult;
use crate::net::sync::ServerManifest;
use async_trait::async_trait;
use bytes::Bytes;
use keyforge_adapter::loader::{AssetLoader, LoaderResult};
use keyforge_model::config::CorpusSource;
use keyforge_model::{Asset, Corpus};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;

/// A local filesystem-based asset provider.
#[derive(Debug, Clone)]
pub struct FsProvider {
    root: PathBuf,
}

impl FsProvider {
    /// Creates a new `FsProvider` rooted at the given directory.
    #[must_use]
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Returns a placeholder hash for local files.
    ///
    /// # Errors
    ///
    /// This implementation currently always returns a successful placeholder.
    pub fn get_corpus_hash(&self, _id: &str) -> LoaderResult<String> {
        Ok("local-dev".to_string())
    }

    async fn resolve_path<T: Asset>(&self, id: &str) -> LoaderResult<PathBuf> {
        let category = T::category().as_str();
        // Search order: User -> System -> Root Fallback
        let base_paths = vec![
            self.root.join("user").join(category).join(id),
            self.root.join("system").join(category).join(id),
            self.root.join(id),
        ];

        let extensions = ["json", "mpk", "mpk.zst"];

        for base in base_paths {
            // Check exact match (e.g. if ID has extension)
            if base.exists() && base.is_file() {
                return Ok(base);
            }

            // Check with extensions
            for ext in extensions {
                let path = base.with_extension(ext);
                if path.exists() && path.is_file() {
                    return Ok(path);
                }
            }
        }

        Err(keyforge_model::error::ForgeError::NotFound(id.to_string()))
    }
}

#[async_trait]
impl AssetServerProvider for FsProvider {
    async fn get_manifest(&self) -> InfraResult<ServerManifest> {
        Ok(ServerManifest::default())
    }

    async fn get_file_content(&self, path: &str) -> InfraResult<Option<Bytes>> {
        let full_path = self.root.join(path);
        if full_path.exists() {
            let data = fs::read(full_path).await.map_err(|e| {
                keyforge_model::error::ForgeError::Io(format!("Failed to read {path}: {e}"))
            })?;
            Ok(Some(Bytes::from(data)))
        } else {
            Ok(None)
        }
    }
}

#[async_trait]
impl AssetLoader for FsProvider {
    async fn load<T: Asset>(&self, id: &str) -> LoaderResult<Arc<T>> {
        let path = self.resolve_path::<T>(id).await?;
        
        let content = fs::read(&path).await.map_err(|e| {
            keyforge_model::error::ForgeError::Io(format!("Failed to read {}: {e}", path.display()))
        })?;

        let path_str = path.to_string_lossy();
        if path_str.ends_with(".zst") || path_str.ends_with(".mpk.zst") {
             let decoder = zstd::Decoder::new(&content[..]).map_err(|e| {
                keyforge_model::error::ForgeError::Io(format!("Zstd decoder error: {e}"))
            })?;
            let asset: T = rmp_serde::from_read(decoder).map_err(|e| {
                keyforge_model::error::ForgeError::Serde(format!("MsgPack decode error: {e}"))
            })?;
            return Ok(Arc::new(asset));
        }

        // Default to JSON
        let asset: T = serde_json::from_slice(&content).map_err(|e| {
            keyforge_model::error::ForgeError::Serde(format!("JSON decode error: {e}"))
        })?;
        
        Ok(Arc::new(asset))
    }

    async fn load_corpus(&self, sources: &[CorpusSource]) -> LoaderResult<Arc<Corpus>> {
        let mut blended = Corpus::default();
        for src in sources {
            let corpus = self.load::<Corpus>(&src.id).await?;
            blended.merge(&corpus, src.weight);
        }
        Ok(Arc::new(blended))
    }

    fn root(&self) -> &Path {
        &self.root
    }
}
