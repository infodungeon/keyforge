// libs/keyforge-infra/src/asset/caching_provider.rs

use async_trait::async_trait;
use keyforge_adapter::loader::{AssetLoader, LoaderResult};
use keyforge_boundary::SafePath;
use keyforge_model::config::CorpusSource;
// use keyforge_model::{Asset, Corpus}; // Original
use keyforge_adapter::model::Asset as AssetWrapper; // Add this
use keyforge_model::{Asset as AssetTrait, Corpus};
use moka::future::Cache;
use serde::de::DeserializeOwned;
use std::any::{Any, TypeId};
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
    async fn load<T: AssetTrait + DeserializeOwned>(
        &self,
        id: &str,
    ) -> LoaderResult<AssetWrapper<T>> {
        let tid = TypeId::of::<T>();
        if let Some(arc_any) = self.cache.get(&(tid, id.to_string())).await {
            // arc_any is Arc<dyn Any + Send + Sync>
            // We expect it to contain Arc<AssetWrapper<T>> (or just AssetWrapper<T> wrapped in Arc?)
            // Let's assume we store Arc<AssetWrapper<T>> in the cache (matches provider return type wrapped in Arc for type erasure)

            // Wait, provider.load returns AssetWrapper<T>.
            // We want to store something in Arc<dyn Any>.
            // If we store AssetWrapper<T>, then downcast returns Arc<AssetWrapper<T>> (reference to value in Arc).
            // Then we clone it.

            if let Ok(arc_wrapper) = arc_any.downcast::<AssetWrapper<T>>() {
                return Ok((*arc_wrapper).clone());
            }

            return Err(keyforge_model::error::ForgeError::Internal(
                "Cache downcast failed".into(),
            ));
        }

        let asset = self.provider.load::<T>(id).await?;
        // Store in cache. asset is AssetWrapper<T>.
        // We wrap in Arc to make it Arc<dyn Any>.
        self.cache
            .insert((tid, id.to_string()), Arc::new(asset.clone()))
            .await;
        Ok(asset)
    }

    async fn load_corpus(&self, sources: &[CorpusSource]) -> LoaderResult<Arc<Corpus>> {
        self.provider.load_corpus(sources).await
    }

    async fn get_hash(
        &self,
        category: keyforge_model::AssetCategory,
        id: &str,
    ) -> LoaderResult<String> {
        self.provider.get_hash(category, id).await
    }

    fn root(&self) -> &SafePath {
        self.provider.root()
    }
}
