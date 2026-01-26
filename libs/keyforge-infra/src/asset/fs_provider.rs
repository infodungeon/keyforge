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
}

#[async_trait]
impl AssetServerProvider for FsProvider {
    async fn get_manifest(&self) -> InfraResult<ServerManifest> {
        Ok(ServerManifest::default())
    }

    async fn get_file_content(&self, path: &str) -> InfraResult<Option<Bytes>> {
        let full_path = self.root.join(path);
        if full_path.exists() {
            let data = std::fs::read(full_path)?;
            Ok(Some(Bytes::from(data)))
        } else {
            Ok(None)
        }
    }
}

#[async_trait]
impl AssetLoader for FsProvider {
    async fn load<T: Asset>(&self, id: &str) -> LoaderResult<Arc<T>> {
        let data = self
            .get_file_content(id)
            .await
            .map_err(|e| {
                keyforge_model::error::ForgeError::Io(std::io::Error::other(e.to_string()))
            })?
            .ok_or_else(|| keyforge_model::error::ForgeError::NotFound(id.to_string()))?;

        serde_json::from_slice(&data)
            .map(Arc::new)
            .map_err(Into::into)
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
