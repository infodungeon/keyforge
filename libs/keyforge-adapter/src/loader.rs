// libs/keyforge-adapter/src/loader.rs

use crate::model::Asset as AssetWrapper;
use async_trait::async_trait;
use keyforge_boundary::SafePath;
use keyforge_model::config::CorpusSource;
use keyforge_model::error::ForgeError;
use keyforge_model::{Asset, Corpus};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};

/// A specialized result type for asset loading operations.
pub type LoaderResult<T> = Result<T, ForgeError>;

/// A trait for types that can load `KeyForge` assets from an external source.
#[async_trait]
pub trait AssetLoader: Send + Sync + Debug {
    /// Generic asset loader.
    async fn load<T: Asset + serde::de::DeserializeOwned>(
        &self,
        id: &str,
    ) -> LoaderResult<AssetWrapper<T>>;

    /// Loads one or more corpora and merges them into a single bundle.
    async fn load_corpus(&self, sources: &[CorpusSource]) -> LoaderResult<Arc<Corpus>>;

    /// Returns the content hash of an asset without loading the full object.
    async fn get_hash(
        &self,
        category: keyforge_model::AssetCategory,
        id: &str,
    ) -> LoaderResult<String>;

    /// Returns the root directory of the asset source.
    fn root(&self) -> &SafePath;
}

/// An in-memory implementation of `AssetLoader`.
#[derive(Debug)]
pub struct InMemoryLoader {
    #[allow(clippy::type_complexity)]
    assets: RwLock<HashMap<TypeId, HashMap<String, Arc<dyn Any + Send + Sync>>>>,
    root: SafePath,
}

impl Default for InMemoryLoader {
    fn default() -> Self {
        Self {
            assets: RwLock::new(HashMap::new()),
            root: SafePath::from_trusted_root_path(std::path::PathBuf::from(".")),
        }
    }
}

#[async_trait]
impl AssetLoader for InMemoryLoader {
    async fn load<T: Asset + serde::de::DeserializeOwned>(
        &self,
        id: &str,
    ) -> LoaderResult<AssetWrapper<T>> {
        self.load_any::<T>(id)
    }

    async fn load_corpus(&self, sources: &[CorpusSource]) -> LoaderResult<Arc<Corpus>> {
        let mut blended = Corpus::default();
        let mut found_any = false;
        for src in sources {
            if let Ok(corpus) = self.load_any::<Corpus>(&src.id) {
                blended.merge(&corpus.content, src.weight);
                found_any = true;
            } else {
                return Err(ForgeError::NotFound(src.id.clone()));
            }
        }
        if !found_any {
            return Err(ForgeError::NotFound("Empty corpus source list".into()));
        }
        Ok(Arc::new(blended))
    }

    async fn get_hash(
        &self,
        _category: keyforge_model::AssetCategory,
        _id: &str,
    ) -> LoaderResult<String> {
        Ok("in-memory-hash".to_string())
    }

    fn root(&self) -> &SafePath {
        &self.root
    }
}

impl InMemoryLoader {
    fn load_any<T: Asset>(&self, id: &str) -> LoaderResult<AssetWrapper<T>> {
        let tid = TypeId::of::<T>();
        let type_name = std::any::type_name::<T>();
        let maps = self
            .assets
            .read()
            .map_err(|e| ForgeError::Internal(format!("Lock poisoned: {e}")))?;
        let map = maps.get(&tid).ok_or_else(|| {
            ForgeError::NotFound(format!("No assets of type '{type_name}' registered"))
        })?;
        let res = map
            .get(id)
            .cloned()
            .ok_or_else(|| ForgeError::NotFound(id.to_string()))?;
        res.downcast::<AssetWrapper<T>>()
            .map(|wrapper| (*wrapper).clone()) // Clone the wrapper (cheap, as it holds Arc)
            .map_err(|_| ForgeError::Internal(format!("Downcast failed for {type_name}")))
    }

    /// Creates a new `InMemoryLoader`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Generic injection of an asset into the in-memory loader.
    pub fn inject<T: Asset + serde::de::DeserializeOwned + serde::Serialize>(
        &self,
        id: &str,
        asset: T,
    ) {
        let tid = TypeId::of::<T>();
        // Compute hash for injection
        // This requires T to be Serialize, which we added to bounds
        let content_hash = match serde_json::to_vec(&asset) {
            Ok(bytes) => {
                use sha2::Digest;
                let mut hasher = sha2::Sha256::new();
                hasher.update(&bytes);
                hasher.finalize().into()
            }
            Err(_) => [0u8; 32], // Fallback if serialization fails (shouldn't happen for test data)
        };

        let wrapper = AssetWrapper::new(Arc::new(asset), content_hash);

        if let Ok(mut maps) = self.assets.write() {
            maps.entry(tid)
                .or_default()
                .insert(id.to_string(), Arc::new(wrapper));
        }
    }
}
