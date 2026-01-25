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
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::sync::Arc;

/// Metadata describing a text corpus.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CorpusMetadata {
    /// If true, this corpus represents standard prose and supports synthetic data injection.
    #[serde(default)]
    pub is_std: bool,
}

/// Represents the statistical data of a language or text source.
/// Contains frequency data for characters, bigrams, and trigrams.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Corpus {
    /// Metadata about the corpus.
    #[serde(default)]
    pub meta: CorpusMetadata,
    /// Frequency of each character (index = char code).
    /// Must be exactly `MAX_KEYCODE_SPACE` elements long to cover all u16 values.
    #[serde(
        serialize_with = "serialize_arc_slice",
        deserialize_with = "deserialize_arc_slice"
    )]
    pub char_freqs: Arc<[u64]>,
    /// List of bigrams (char1, char2, frequency).
    #[serde(
        serialize_with = "serialize_arc_slice",
        deserialize_with = "deserialize_arc_slice"
    )]
    pub bigrams: Arc<[(u16, u16, u32)]>,
    /// List of trigrams (char1, char2, char3, frequency).
    #[serde(
        serialize_with = "serialize_arc_slice",
        deserialize_with = "deserialize_arc_slice"
    )]
    pub trigrams: Arc<[(u16, u16, u16, u32)]>,
    /// List of common words and their frequencies.
    #[serde(
        serialize_with = "serialize_arc_slice",
        deserialize_with = "deserialize_arc_slice"
    )]
    pub words: Arc<[(String, u32)]>,
}

fn serialize_arc_slice<S, T>(val: &Arc<[T]>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Serialize,
{
    (**val).serialize(s)
}

fn deserialize_arc_slice<'de, D, T>(d: D) -> Result<Arc<[T]>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    let v = Vec::<T>::deserialize(d)?;
    Ok(Arc::from(v))
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
        let mut new_char_freqs = self.char_freqs.to_vec();
        for (i, &freq) in other.char_freqs.iter().enumerate() {
            if i < new_char_freqs.len() {
                #[allow(
                    clippy::cast_precision_loss,
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss
                )]
                let merged_freq = (freq as f32 * weight).round() as u64;
                new_char_freqs[i] += merged_freq;
            }
        }
        self.char_freqs = Arc::from(new_char_freqs);

        let mut new_bigrams = self.bigrams.to_vec();
        for &(c1, c2, freq) in &*other.bigrams {
            #[allow(
                clippy::cast_precision_loss,
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss
            )]
            let merged_freq = (freq as f32 * weight).round() as u32;
            new_bigrams.push((c1, c2, merged_freq));
        }
        new_bigrams.sort_unstable_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
        self.bigrams = Arc::from(new_bigrams);

        let mut new_trigrams = self.trigrams.to_vec();
        for &(c1, c2, c3, freq) in &*other.trigrams {
            #[allow(
                clippy::cast_precision_loss,
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss
            )]
            let merged_freq = (freq as f32 * weight).round() as u32;
            new_trigrams.push((c1, c2, c3, merged_freq));
        }
        new_trigrams.sort_unstable_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));
        self.trigrams = Arc::from(new_trigrams);

        let mut new_words = self.words.to_vec();
        for (word, freq) in &*other.words {
            #[allow(
                clippy::cast_precision_loss,
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss
            )]
            let merged_freq = (*freq as f32 * weight).round() as u32;
            new_words.push((word.clone(), merged_freq));
        }
        self.words = Arc::from(new_words);
    }
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;

    #[test]
    fn test_corpus_lifecycle() {
        let mut c = Corpus::default();
        let mut freqs = c.char_freqs.to_vec();
        freqs['a' as usize] = 100;
        c.char_freqs = Arc::from(freqs);

        c.bigrams = Arc::from(vec![('a' as u16, 'b' as u16, 50)]);
        c.trigrams = Arc::from(vec![('a' as u16, 'b' as u16, 'c' as u16, 10)]);
        c.words = Arc::from(vec![("test".to_string(), 5)]);

        let json = serde_json::to_string(&c).expect("Failed to serialize Corpus");
        let recovered: Corpus = serde_json::from_str(&json).expect("Failed to deserialize Corpus");

        assert_eq!(recovered.char_freqs['a' as usize], 100);
        assert_eq!(recovered.bigrams.len(), 1);
        assert_eq!(recovered.bigrams[0], ('a' as u16, 'b' as u16, 50));
    }
}
