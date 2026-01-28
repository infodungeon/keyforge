// libs/keyforge-infra/src/asset/fs_provider.rs

use async_trait::async_trait;
use keyforge_adapter::loader::{AssetLoader, LoaderResult};
use keyforge_model::config::CorpusSource;
use keyforge_model::corpus::CorpusMerger;
use keyforge_model::error::ForgeError;
use keyforge_model::{Asset, AssetCategory, Corpus};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;

/// A file-system based implementation of `AssetLoader`.
#[derive(Debug, Clone)]
pub struct FsProvider {
    root: PathBuf,
}

impl FsProvider {
    /// Creates a new `FsProvider` with the specified root directory.
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }

    fn get_path<T: Asset>(&self, id: &str) -> PathBuf {
        let category_dir = match T::category() {
            AssetCategory::Keyboard => "keyboards",
            AssetCategory::Corpus => "corpora",
            AssetCategory::CostModel => "weights",
            AssetCategory::Layout => "layouts",
            AssetCategory::Rubric => "rubrics",
            AssetCategory::Keycodes => "config",
        };
        self.root.join(category_dir).join(format!("{id}.json"))
    }
}

#[async_trait]
impl AssetLoader for FsProvider {
    async fn load<T: Asset>(&self, id: &str) -> LoaderResult<Arc<T>> {
        let path = self.get_path::<T>(id);
        let content = fs::read_to_string(&path).await.map_err(|e| {
            ForgeError::NotFound(format!("Asset {id} not found at {}: {e}", path.display()))
        })?;

        let mut asset = serde_json::from_str::<T>(&content)
            .map_err(|e| ForgeError::InvalidData(format!("Failed to parse {id}: {e}")))?;

        asset.post_load()?;
        Ok(Arc::new(asset))
    }

    async fn load_corpus(&self, sources: &[CorpusSource]) -> LoaderResult<Arc<Corpus>> {
        if sources.is_empty() {
            return Err(ForgeError::InvalidData("No corpus sources provided".into()));
        }

        let mut blended = Corpus::default();
        let mut found_any = false;

        for src in sources {
            let corpus = self.load::<Corpus>(&src.id).await?;
            CorpusMerger::merge(&mut blended, &corpus, src.weight);
            found_any = true;
        }

        if !found_any {
            return Err(ForgeError::NotFound("None of the corpora found".into()));
        }

        // Standard post-processing for prose
        blended.inject_synthetic_data();

        Ok(Arc::new(blended))
    }

    fn root(&self) -> &Path {
        &self.root
    }
}

#[async_trait]
impl crate::asset::AssetServerProvider for FsProvider {
    async fn get_manifest(&self) -> crate::error::InfraResult<crate::net::sync::ServerManifest> {
        // Naive implementation for local dev: scan the root directory recursively
        let mut files = std::collections::HashMap::new();
        let mut dir_stack = vec![self.root.clone()];

        while let Some(dir) = dir_stack.pop() {
            if let Ok(mut entries) = fs::read_dir(&dir).await {
                while let Some(entry) = entries.next_entry().await.unwrap_or(None) {
                    let path = entry.path();
                    if path.is_dir() {
                        dir_stack.push(path);
                    } else if let Ok(rel_path) = path.strip_prefix(&self.root) {
                        let path_str = rel_path.to_string_lossy().into_owned();
                        files.insert(path_str, "local".to_string());
                    }
                }
            }
        }

        Ok(crate::net::sync::ServerManifest { files })
    }

    async fn get_file_content(
        &self,
        path: &str,
    ) -> crate::error::InfraResult<Option<bytes::Bytes>> {
        let full_path = self.root.join(path);
        if let Ok(data) = fs::read(&full_path).await {
            Ok(Some(bytes::Bytes::from(data)))
        } else {
            Ok(None)
        }
    }
}
