// libs/keyforge-model/src/corpus.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You    may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Text corpus data structures and validation.
//!
//! A `Corpus` provides the statistical foundation for layout optimization,
//! including character, bigram, and trigram frequencies.

use crate::asset::{Asset, AssetCategory};
use crate::constants::MAX_KEYCODE_SPACE;
use crate::error::ForgeError;
use crate::validator::Validator;
use std::sync::Arc;

/// Metadata describing a text corpus.
#[derive(Debug, Clone, Default)]
pub struct CorpusMetadata {
    /// If true, this corpus represents standard prose and supports synthetic data injection.
    pub is_std: bool,
}

/// Represents the statistical data of a language or text source.
/// Contains frequency data for characters, bigrams, and trigrams.
#[derive(Debug, Clone)]
pub struct Corpus {
    /// Metadata about the corpus.
    pub meta: CorpusMetadata,
    /// Frequency of each character (index = char code).
    /// Must be exactly `MAX_KEYCODE_SPACE` elements long to cover all u16 values.
    pub char_freqs: Arc<[u64]>,
    /// List of bigrams (char1, char2, frequency).
    pub bigrams: Arc<[(u16, u16, u32)]>,
    /// List of trigrams (char1, char2, char3, frequency).
    pub trigrams: Arc<[(u16, u16, u16, u32)]>,
    /// List of common words and their frequencies.
    pub words: Arc<[(String, u32)]>,
}

impl Asset for Corpus {
    fn category() -> AssetCategory {
        AssetCategory::Corpus
    }

    fn post_load(&mut self) -> Result<(), ForgeError> {
        self.validate_internal()
    }
}

impl Default for Corpus {
    fn default() -> Self {
        Self {
            meta: CorpusMetadata::default(),
            char_freqs: Arc::from(vec![0; MAX_KEYCODE_SPACE]),
            bigrams: Arc::from(vec![]),
            trigrams: Arc::from(vec![]),
            words: Arc::from(vec![]),
        }
    }
}

impl Validator for Corpus {
    fn validate(&self) -> Result<(), String> {
        self.validate_internal().map_err(|e| e.to_string())
    }
}

impl Corpus {
    /// Validates the integrity of the Corpus.
    /// Ensures that frequency maps are sized correctly to prevent panics in the Physics engine.
    ///
    /// # Errors
    /// Returns a `ForgeError` if the character frequency map is not exactly `MAX_KEYCODE_SPACE` elements.
    pub fn validate_internal(&self) -> Result<(), ForgeError> {
        if self.char_freqs.len() != MAX_KEYCODE_SPACE {
            return Err(ForgeError::InvalidData(format!(
                "Corpus char_freqs length must be {}, found {}",
                MAX_KEYCODE_SPACE,
                self.char_freqs.len()
            )));
        }
        Ok(())
    }

    /// Merges another corpus into this one with a specific weight.
    pub fn merge(&mut self, other: &Self, weight: f32) {
        let w_fixed = crate::types::FixedWeight::from_f32(weight).unwrap_or_default();

        let mut new_char_freqs = self.char_freqs.to_vec();
        for (i, &freq) in other.char_freqs.iter().enumerate() {
            if i < new_char_freqs.len() {
                let f_score =
                    crate::types::Score::from_scaled_i64(i64::try_from(freq).unwrap_or(i64::MAX));
                let merged = f_score.saturating_mul_weight(w_fixed);
                let merged_freq = u64::try_from(merged.raw()).unwrap_or(0);
                new_char_freqs[i] = new_char_freqs[i].saturating_add(merged_freq);
            }
        }
        self.char_freqs = Arc::from(new_char_freqs);

        // [TODO] Merge bigrams, trigrams etc.
    }

    /// Hook called after the asset is successfully deserialized.
    /// Used for validation or rebuilding internal lookups.
    ///
    /// # Errors
    /// Returns a `ForgeError` if the internal validation fails.
    pub fn post_load(&mut self) -> Result<(), ForgeError> {
        self.validate_internal()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_corpus_basic() {
        let corpus = Corpus::default();
        assert_eq!(corpus.char_freqs.len(), MAX_KEYCODE_SPACE);
        assert!(corpus.validate_internal().is_ok());
    }
}
