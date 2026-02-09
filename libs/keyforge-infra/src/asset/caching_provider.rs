// libs/keyforge-infra/src/asset/caching_provider.rs

use async_trait::async_trait;
use keyforge_adapter::loader::{AssetLoader, LoaderResult};
use keyforge_model::config::CorpusSource;
use keyforge_model::{Asset, Corpus};
use moka::future::Cache;
use std::any::{Any, TypeId};
use std::path::Path;
use std::sync::Arc;

type AssetCache = Cache<(TypeId, String), Arc<dyn Any + Send + Sync>>;

/// A tiered cache provider that wraps an underlying `AssetLoader`.
#[derive(Debug, Clone)]
pub struct CachingProvider<L: AssetLoader> {
    provider: Arc<L>,
    cache: AssetCache,
}

impl<L: AssetLoader> CachingProvider<L> {
    /// Creates a new `CachingProvider`.
    pub fn new(provider: Arc<L>, max_capacity: u64) -> Self {
        let cache = Cache::builder().max_capacity(max_capacity).build();
        Self { provider, cache }
    }

    /// Returns the hash of a corpus.
    ///
    /// # Errors
    ///
    /// This implementation currently always returns a successful placeholder.
    pub fn get_corpus_hash(&self, _id: &str) -> LoaderResult<String> {
        Ok("cached".to_string())
    }
}

#[async_trait]
impl<L: AssetLoader> AssetLoader for CachingProvider<L> {
    async fn load<T: Asset>(&self, id: &str) -> LoaderResult<Arc<T>> {
        let tid = TypeId::of::<T>();
        if let Some(asset) = self.cache.get(&(tid, id.to_string())).await {
            return asset.downcast::<T>().map_err(|_| {
                keyforge_model::error::ForgeError::Internal("Cache downcast failed".into())
            });
        }

        let asset = self.provider.load::<T>(id).await?;
        self.cache
            .insert((tid, id.to_string()), asset.clone())
            .await;
        Ok(asset)
    }

    async fn load_corpus(&self, sources: &[CorpusSource]) -> LoaderResult<Arc<Corpus>> {
        self.provider.load_corpus(sources).await
    }

    async fn get_hash(&self, category: keyforge_model::AssetCategory, id: &str) -> LoaderResult<String> {
        self.provider.get_hash(category, id).await
    }

    fn root(&self) -> &Path {
        self.provider.root()
    }
}
