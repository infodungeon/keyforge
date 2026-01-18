// libs/keyforge-core/src/loader.rs

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

use keyforge_model::config::CorpusSource;
use keyforge_model::error::ForgeError;
use keyforge_model::{Asset, Corpus};
use std::sync::Arc;
use std::fmt::Debug;

/// A specialized result type for asset loading operations.
pub type LoaderResult<T> = Result<T, ForgeError>;

/// A trait for types that can load `KeyForge` assets from an external source.
///
/// This is the primary abstraction for IO, allowing core logic to remain 
/// agnostic to the filesystem, network, or embedded storage.
#[async_trait::async_trait]
pub trait AssetLoader: Send + Sync + Debug {
    /// Generic asset loader.
    async fn load<T: Asset>(&self, id: &str) -> LoaderResult<Arc<T>>
    where
        Self: Sized;

    /// Loads one or more corpora and merges them into a single bundle.
    /// 
    /// Corpus is currently special as it often requires merging multiple sources.
    async fn load_corpus(&self, sources: &[CorpusSource]) -> LoaderResult<Arc<Corpus>>;
}
