// libs/keyforge-model/src/asset.rs

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

use serde::de::DeserializeOwned;
use std::fmt::Debug;
use crate::error::ForgeError;

/// Categories of assets supported by the `KeyForge` system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetCategory {
    /// Physical keyboard geometry and metadata.
    Keyboard,
    /// Physics model containing static costs and dynamic rules.
    CostModel,
    /// Registry mapping logical labels to physical keycodes.
    Keycodes,
    /// Linguistic statistical data (N-grams).
    Corpus,
}

impl AssetCategory {
    /// Returns the standard subdirectory name for this asset category.
    #[must_use] 
    pub fn as_str(&self) -> &'static str {
        match self {
            AssetCategory::Keyboard => "keyboards",
            AssetCategory::CostModel => "weights",
            AssetCategory::Keycodes => "config",
            AssetCategory::Corpus => "corpora",
        }
    }
}

/// A trait for data structures that can be managed as system or user assets.
pub trait Asset: DeserializeOwned + Send + Sync + 'static + Debug {
    /// Returns the category of this asset type.
    fn category() -> AssetCategory;

    /// Default extension for this asset type (e.g., "json", "toml")
    #[must_use] 
    fn default_extension() -> &'static str {
        "json"
    }

    /// Hook called after the asset is successfully deserialized.
    /// Used for validation or rebuilding internal lookups.
    ///
    /// # Errors
    ///
    /// Returns a `ForgeError` if post-processing or validation fails.
    fn post_load(&mut self) -> Result<(), ForgeError> {
        Ok(())
    }
}