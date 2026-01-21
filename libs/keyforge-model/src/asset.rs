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

use crate::error::ForgeError;
use serde::de::DeserializeOwned;
use std::fmt::Debug;

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
    /// User preferences and scoring weights.
    Rubric,
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
            AssetCategory::Rubric => "rubrics",
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

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(serde::Deserialize, Debug)]
    struct MockAsset;
    impl Asset for MockAsset {
        fn category() -> AssetCategory { AssetCategory::Keyboard }
    }

    #[test]
    fn test_asset_post_load_default() {
        let mut asset = MockAsset;
        assert!(asset.post_load().is_ok());
    }

    #[test]
    fn test_asset_default_extension() {
        assert_eq!(MockAsset::default_extension(), "json");
    }

    #[test]
    fn test_asset_category_as_str() {
        assert_eq!(AssetCategory::Keyboard.as_str(), "keyboards");
        assert_eq!(AssetCategory::CostModel.as_str(), "weights");
        assert_eq!(AssetCategory::Keycodes.as_str(), "config");
        assert_eq!(AssetCategory::Corpus.as_str(), "corpora");
        assert_eq!(AssetCategory::Rubric.as_str(), "rubrics");
    }

    #[test]
    fn test_asset_category_derive() {
        let c = AssetCategory::Keyboard;
        let c2 = c;
        assert_eq!(c, c2);
        assert_eq!(AssetCategory::Rubric.as_str(), "rubrics");
    }

    #[test]
    fn test_dummy_asset_extension() {
        #[derive(Debug, serde::Deserialize, serde::Serialize)]
        struct DummyAsset;
        impl Asset for DummyAsset {
            fn category() -> AssetCategory { AssetCategory::Rubric }
        }
        assert_eq!(DummyAsset::default_extension(), "json");
    }
}
