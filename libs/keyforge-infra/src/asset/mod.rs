// libs/keyforge-infra/src/asset/mod.rs

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

/// Typed cache storage for asset objects.
pub mod cache;
/// Tiered caching provider for high-performance asset reads.
pub mod caching_provider;
/// Filesystem-based asset provider for local development and testing.
pub mod fs_provider;
/// High-level orchestration for fetching and ensuring asset presence.
pub mod manager;
/// Logic for secure path resolution and asset location.
pub mod resolver;
/// Distributed asset provider backed by an external data store (e.g., Valkey).
pub mod valkey_provider;

pub use valkey_provider::ValkeyProvider;

use crate::net::sync::ServerManifest;
use crate::error::InfraResult;

/// A trait for asset providers that can serve raw file content and manifests.
///
/// This trait is 'dyn compatible' (object safe) because it does not contain
/// generic methods, unlike the base `AssetLoader`.
#[async_trait::async_trait]
pub trait AssetServerProvider: Send + Sync + std::fmt::Debug {
    /// Returns a manifest of all available system assets.
    async fn get_manifest(&self) -> InfraResult<ServerManifest>;
    /// Returns the raw byte content of a file at the given relative path.
    async fn get_file_content(&self, path: &str) -> InfraResult<Option<bytes::Bytes>>;
}

/// Path prefix for keyboard definition files.
pub const ASSET_PATH_KEYBOARDS: &str = "keyboards/models/";
/// Path prefix for corpus files.
pub const ASSET_PATH_CORPORA: &str = "corpora/";
/// Path prefix for cost matrix files.
pub const ASSET_PATH_WEIGHTS: &str = "weights/";
/// Path prefix for configuration files.
pub const ASSET_PATH_CONFIG: &str = "config/";
/// Path prefix for keymap extra assets.
pub const ASSET_PATH_KEYMAP_EXTRAS: &str = "keymap_extras/";