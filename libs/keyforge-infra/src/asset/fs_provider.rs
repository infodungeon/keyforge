// libs/keyforge-infra/src/asset/fs_provider.rs

use crate::asset::AssetServerProvider;
use crate::error::InfraResult;
use crate::net::sync::ServerManifest;
use async_trait::async_trait;
use bytes::Bytes;
use keyforge_adapter::loader::{AssetLoader, LoaderResult};
use keyforge_model::config::CorpusSource;
use keyforge_model::{Asset, AssetCategory, Corpus};
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

    /// Returns the SHA-256 hash of a local corpus file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read.
    pub fn get_corpus_hash(&self, id: &str) -> LoaderResult<String> {
        use sha2::{Digest, Sha256};
        let full_path = self.root.join(id);
        let data = std::fs::read(full_path).map_err(|e| {
            keyforge_model::error::ForgeError::Io(format!("Failed to hash asset {id}: {e}"))
        })?;
        let mut hasher = Sha256::new();
        hasher.update(&data);
        Ok(hex::encode(hasher.finalize()))
    }
}

#[async_trait]
impl AssetServerProvider for FsProvider {
    async fn get_manifest(&self) -> InfraResult<ServerManifest> {
        let mut manifest = ServerManifest::default();
        if let Ok(entries) = std::fs::read_dir(&self.root) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if let Ok(hash) = self.get_corpus_hash(&name) {
                            manifest.files.insert(name, hash);
                        }
                    }
                }
            }
        }
        Ok(manifest)
    }

    async fn get_file_content(&self, path: &str) -> InfraResult<Option<Bytes>> {
        let full_path = self.root.join(path);
        if full_path.exists() {
            let data = std::fs::read(full_path).map_err(crate::error::InfraError::Io)?;
            Ok(Some(Bytes::from(data)))
        } else {
            Ok(None)
        }
    }
}

#[async_trait]
impl AssetLoader for FsProvider {
    async fn load<T: Asset>(&self, id: &str) -> LoaderResult<Arc<T>> {
        let category = T::category();
        let mut final_id = id.to_string();

        if category == AssetCategory::CostModel {
            final_id = "weights/cost_matrix.json".to_string();
        }

        let full_path = self.root.join(&final_id);
        let mut data = if full_path.exists() {
            std::fs::read(&full_path).map_err(|e| {
                keyforge_model::error::ForgeError::Io(format!(
                    "Failed to load asset {final_id}: {e}"
                ))
            })?
        } else if category == AssetCategory::CostModel {
            // Try .mpk.zst if .json is missing for CostModel
            let mpk_path = self.root.join("weights/cost_matrix.mpk.zst");
            final_id = "weights/cost_matrix.mpk.zst".to_string();
            std::fs::read(mpk_path).map_err(|e| {
                keyforge_model::error::ForgeError::Io(format!("Failed to load cost matrix: {e}"))
            })?
        } else {
            return Err(keyforge_model::error::ForgeError::NotFound(final_id));
        };

        // Handle compressed assets
        if final_id.ends_with(".zst") || final_id.ends_with(".mpk.zst") {
            data = zstd::decode_all(std::io::Cursor::new(data)).map_err(|e| {
                keyforge_model::error::ForgeError::Io(format!(
                    "Failed to decompress {final_id}: {e}"
                ))
            })?;
        }

        if final_id.ends_with(".json") {
            serde_json::from_slice(&data).map(Arc::new).map_err(|e| {
                keyforge_model::error::ForgeError::Serde(format!("Failed to parse {final_id}: {e}"))
            })
        } else {
            rmp_serde::from_slice(&data).map(Arc::new).map_err(|e| {
                keyforge_model::error::ForgeError::InvalidData(format!(
                    "Failed to parse {final_id}: {e}"
                ))
            })
        }
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
