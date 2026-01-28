// libs/keyforge-infra/src/asset/fs_provider.rs

use crate::error::InfraResult;
use async_trait::async_trait;
use keyforge_adapter::loader::{AssetLoader, LoaderResult};
use keyforge_model::config::CorpusSource;
use keyforge_model::{Asset, Corpus};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// A local filesystem provider for assets.
#[derive(Debug, Clone)]
pub struct FsProvider {
    root: PathBuf,
}

impl FsProvider {
    /// Creates a new `FsProvider` with the given data root.
    #[must_use]
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Resolves a path relative to the root.
    #[must_use]
    pub fn resolve(&self, path: &str) -> PathBuf {
        self.root.join(path)
    }

    /// Reads the content of a file relative to the root.
    ///
    /// # Errors
    /// Returns `InfraError` if the file cannot be read.
    pub async fn get_file_content(&self, path: &str) -> InfraResult<Option<Vec<u8>>> {
        let full_path = self.resolve(path);
        if full_path.exists() {
            let data = tokio::fs::read(&full_path).await?;
            Ok(Some(data))
        } else {
            Ok(None)
        }
    }
}

#[async_trait]
impl AssetLoader for FsProvider {
    async fn load<T: Asset>(&self, id: &str) -> LoaderResult<Arc<T>> {
        let category = T::category();
        let stem = id.strip_suffix(".json").unwrap_or(id);

        // Try standard paths and extensions
        let mut candidates = Vec::new();
        let sub = category.as_str();

        for prefix in ["system", "user"] {
            // 1. Direct category path
            candidates.push(format!("{prefix}/{sub}/{stem}.json"));
            candidates.push(format!("{prefix}/{sub}/{stem}.mpk.zst"));

            // 2. Models subfolder (common for keyboards)
            candidates.push(format!("{prefix}/{sub}/models/{stem}.json"));
            candidates.push(format!("{prefix}/{sub}/models/{stem}.mpk.zst"));
        }
        candidates.push(id.to_string());

        for path in candidates {
            if let Ok(Some(data)) = self.get_file_content(&path).await {
                if path.ends_with(".mpk.zst") {
                    let decoder = zstd::Decoder::new(&data[..])
                        .map_err(keyforge_model::error::ForgeError::Io)?;
                    return rmp_serde::from_read(decoder).map(Arc::new).map_err(|e| {
                        keyforge_model::error::ForgeError::InvalidData(e.to_string())
                    });
                }
                return serde_json::from_slice(&data)
                    .map(Arc::new)
                    .map_err(Into::into);
            }
        }

        Err(keyforge_model::error::ForgeError::NotFound(id.to_string()))
    }

    async fn load_corpus(&self, sources: &[CorpusSource]) -> LoaderResult<Arc<Corpus>> {
        let mut blended = Corpus::default();
        for src in sources {
            // 1. Try loading as a single bundled asset first
            if let Ok(corpus) = self.load::<Corpus>(&src.id).await {
                blended.merge(&corpus, src.weight);
            } else {
                // 2. Try loading as a directory-style corpus (segments)
                let base_path = format!("system/corpora/{}", src.id);
                let mut found_segments = false;
                let mut segments = Vec::new();

                for part in ["1grams", "2grams", "3grams", "words"] {
                    // Try .json then .mpk.zst
                    let part_json = format!("{base_path}/{part}.json");
                    let part_mpk = format!("{base_path}/{part}.mpk.zst");

                    if let Ok(Some(data)) = self.get_file_content(&part_json).await {
                        let part_val: Vec<serde_json::Value> = serde_json::from_slice(&data)
                            .map_err(Into::<keyforge_model::error::ForgeError>::into)?;
                        segments.push((part, part_val));
                        found_segments = true;
                    } else if let Ok(Some(compressed)) = self.get_file_content(&part_mpk).await {
                        let decoder = zstd::Decoder::new(&compressed[..])
                            .map_err(keyforge_model::error::ForgeError::Io)?;
                        let part_val: Vec<serde_json::Value> = rmp_serde::from_read(decoder)
                            .map_err(|e| {
                                keyforge_model::error::ForgeError::InvalidData(e.to_string())
                            })?;
                        segments.push((part, part_val));
                        found_segments = true;
                    }
                }

                if found_segments {
                    crate::util::corpus::populate_corpus_from_segments(
                        &mut blended,
                        src.weight,
                        segments,
                    )?;
                } else {
                    return Err(keyforge_model::error::ForgeError::NotFound(src.id.clone()));
                }
            }
        }
        Ok(Arc::new(blended))
    }

    fn root(&self) -> &Path {
        &self.root
    }
}

use crate::asset::{AssetServerProvider, ServerManifest};

use std::collections::HashMap;

#[async_trait]
impl AssetServerProvider for FsProvider {
    async fn get_manifest(&self) -> InfraResult<ServerManifest> {
        // Basic manifest implementation for local FS provider
        // This is a placeholder; a full implementation would walk the directory
        Ok(ServerManifest {
            files: HashMap::new(),
        })
    }

    async fn get_file_content(&self, path: &str) -> InfraResult<Option<bytes::Bytes>> {
        let res = self.get_file_content(path).await?;
        Ok(res.map(bytes::Bytes::from))
    }
}
