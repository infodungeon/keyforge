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
use keyforge_model::geometry::KeyboardDefinition;
use keyforge_model::keycodes::KeycodeRegistry;
use keyforge_model::Corpus;
use keyforge_model::cost_model::CostModel;
use std::sync::Arc;
use std::fmt::Debug;

/// A specialized result type for asset loading operations.
pub type LoaderResult<T> = Result<T, ForgeError>;

/// A trait for types that can load KeyForge assets from an external source.
///
/// This is the primary abstraction for IO, allowing core logic to remain 
/// agnostic to the filesystem, network, or embedded storage.
#[async_trait::async_trait]
pub trait AssetLoader: Send + Sync + Debug {
    /// Loads a keyboard definition by name.
    async fn load_keyboard(&self, name: &str) -> LoaderResult<Arc<KeyboardDefinition>>;
    /// Loads one or more corpora and merges them into a single bundle.
    async fn load_corpus(&self, sources: &[CorpusSource]) -> LoaderResult<Arc<Corpus>>;
    /// Loads the full physics cost model (New Standard).
    async fn load_cost_model(&self, filename: &str) -> LoaderResult<Arc<CostModel>>;
    /// Loads a keycode registry for mapping between labels and codes.
    async fn load_keycodes(&self, filename: &str) -> LoaderResult<Arc<KeycodeRegistry>>;
}
