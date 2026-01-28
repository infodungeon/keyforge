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
use crate::constants::{
    CORPUS_TOKEN_MAP, MAX_KEYCODE_SPACE, STD_CORPUS_BACKSPACE_FACTOR, STD_CORPUS_ERROR_RATE,
    STD_CORPUS_SENTENCE_RATIO,
};
use crate::error::ForgeError;
use crate::validator::Validator;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::sync::Arc;

/// Wrapper for bigram frequency data.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BigramFrequencyTable {
    /// List of bigrams (char1, char2, frequency).
    #[serde(
        serialize_with = "serialize_arc_slice",
        deserialize_with = "deserialize_arc_slice"
    )]
    pub entries: Arc<[(u16, u16, u32)]>,
}

impl From<Arc<[(u16, u16, u32)]>> for BigramFrequencyTable {
    fn from(entries: Arc<[(u16, u16, u32)]>) -> Self {
        Self { entries }
    }
}

impl From<Vec<(u16, u16, u32)>> for BigramFrequencyTable {
    fn from(entries: Vec<(u16, u16, u32)>) -> Self {
        Self {
            entries: Arc::from(entries),
        }
    }
}

/// Wrapper for trigram frequency data.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TrigramFrequencyTable {
    /// List of trigrams (char1, char2, char3, frequency).
    #[serde(
        serialize_with = "serialize_arc_slice",
        deserialize_with = "deserialize_arc_slice"
    )]
    pub entries: Arc<[(u16, u16, u16, u32)]>,
}

impl From<Arc<[(u16, u16, u16, u32)]>> for TrigramFrequencyTable {
    fn from(entries: Arc<[(u16, u16, u16, u32)]>) -> Self {
        Self { entries }
    }
}

impl From<Vec<(u16, u16, u16, u32)>> for TrigramFrequencyTable {
    fn from(entries: Vec<(u16, u16, u16, u32)>) -> Self {
        Self {
            entries: Arc::from(entries),
        }
    }
}

/// Encapsulates the statistical frequency data of a corpus.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FrequencyTables {
    /// List of bigrams (char1, char2, frequency).
    pub bigrams: BigramFrequencyTable,
    /// List of trigrams (char1, char2, char3, frequency).
    pub trigrams: TrigramFrequencyTable,
    /// List of common words and their frequencies.
    #[serde(
        serialize_with = "serialize_arc_slice",
        deserialize_with = "deserialize_arc_slice"
    )]
    pub words: Arc<[(String, u32)]>,
}

/// Represents the statistical data of a language or text source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Corpus {
    /// If true, this corpus represents standard prose.
    #[serde(default)]
    pub is_std: bool,
    /// Frequency of each character (index = char code).
    #[serde(
        serialize_with = "serialize_arc_slice",
        deserialize_with = "deserialize_arc_slice"
    )]
    pub char_freqs: Arc<[u64]>,
    /// Statistical frequency data.
    pub frequencies: FrequencyTables,
}

/// Service for merging multiple corpora into a single weighted aggregate.
#[derive(Debug, Default)]
pub struct CorpusMerger;

impl CorpusMerger {
    /// Merges another corpus into an existing one with a specific weight.
    pub fn merge(base: &mut Corpus, other: &Corpus, weight: f32) {
        let mut new_char_freqs = base.char_freqs.to_vec();
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
        base.char_freqs = Arc::from(new_char_freqs);

        Self::merge_tables(&mut base.frequencies, &other.frequencies, weight);
    }

    fn merge_tables(base: &mut FrequencyTables, other: &FrequencyTables, weight: f32) {
        // Merge Bigrams
        let mut new_bigrams = base.bigrams.entries.to_vec();
        for &(c1, c2, freq) in &*other.bigrams.entries {
            #[allow(
                clippy::cast_precision_loss,
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss
            )]
            let merged_freq = (freq as f32 * weight).round() as u32;
            new_bigrams.push((c1, c2, merged_freq));
        }
        new_bigrams.sort_unstable_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
        base.bigrams.entries = Arc::from(new_bigrams);

        // Merge Trigrams
        let mut new_trigrams = base.trigrams.entries.to_vec();
        for &(c1, c2, c3, freq) in &*other.trigrams.entries {
            #[allow(
                clippy::cast_precision_loss,
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss
            )]
            let merged_freq = (freq as f32 * weight).round() as u32;
            new_trigrams.push((c1, c2, c3, merged_freq));
        }
        new_trigrams.sort_unstable_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));
        base.trigrams.entries = Arc::from(new_trigrams);

        // Merge Words
        let mut new_words = base.words.to_vec();
        for (word, freq) in &*other.words {
            #[allow(
                clippy::cast_precision_loss,
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss
            )]
            let merged_freq = (*freq as f32 * weight).round() as u32;
            new_words.push((word.clone(), merged_freq));
        }
        base.words = Arc::from(new_words);
    }
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
            is_std: false,
            char_freqs: Arc::from(vec![0; MAX_KEYCODE_SPACE]),
            frequencies: FrequencyTables::default(),
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
    ///
    /// # Errors
    /// Returns a `ForgeError` if the character frequency map is not sized correctly.
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

    /// Populates this corpus from raw n-gram segments with weighted frequencies.
    ///
    /// # Errors
    /// Returns `ForgeError` if the input data is invalid.
    pub fn populate_from_segments(
        &mut self,
        weight: f32,
        segments: Vec<(&str, Vec<Value>)>, // Changed from Vec<(&str, Vec<Value>)> to Vec<(&str, Vec<Value>)> to match the original code
    ) -> Result<(), ForgeError> {
        for (stem, part) in segments {
            match stem {
                "1grams" => self.parse_monograms(weight, &part)?,
                "2grams" => self.parse_bigrams(weight, &part)?,
                "3grams" => self.parse_trigrams(weight, &part)?,
                "words" => self.parse_words(weight, &part)?,
                _ => {} // Changed from _ => {} to _ => {} to match the original code
            }
        }
        Ok(())
    }

    #[allow(clippy::cast_precision_loss, clippy::cast_sign_loss)]
    fn parse_monograms(&mut self, weight: f32, part: &[Value]) -> Result<(), ForgeError> {
        let mut freqs = self.char_freqs.to_vec();
        for e in part {
            if let Some(c) = e["char"].as_str().and_then(Self::resolve_corpus_char) {
                if (c as u32) > 0xFFFF {
                    return Err(ForgeError::InvalidData(format!(
                        "Character outside BMP not supported: {c}"
                    )));
                }
                let c_u16 = c as u16;
                let freq = e["freq"].as_u64().ok_or_else(|| {
                    ForgeError::InvalidData(format!("Missing frequency in 1gram entry: {e:?}"))
                })?;
                #[allow(
                    clippy::cast_precision_loss,
                    clippy::cast_sign_loss,
                    clippy::cast_possible_truncation
                )]
                {
                    freqs[c_u16 as usize] += (freq as f32 * weight).round() as u64;
                }
            }
        }
        self.char_freqs = Arc::from(freqs);
        Ok(())
    }

    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation
    )]
    fn parse_bigrams(&mut self, weight: f32, part: &[Value]) -> Result<(), ForgeError> {
        let mut bigrams = self.frequencies.bigrams.entries.to_vec();
        for e in part {
            let freq = e["freq"].as_u64().ok_or_else(|| {
                ForgeError::InvalidData(format!("Missing frequency in 2gram entry: {e:?}"))
            })?;
            let c1_char = e["char1"]
                .as_str()
                .and_then(Self::resolve_corpus_char)
                .ok_or_else(|| {
                    ForgeError::InvalidData(format!(
                        "Missing or invalid char1 in 2gram entry: {e:?}"
                    ))
                })?;
            let c2_char = e["char2"]
                .as_str()
                .and_then(Self::resolve_corpus_char)
                .ok_or_else(|| {
                    ForgeError::InvalidData(format!(
                        "Missing or invalid char2 in 2gram entry: {e:?}"
                    ))
                })?;

            if (c1_char as u32) > 0xFFFF || (c2_char as u32) > 0xFFFF {
                return Err(ForgeError::InvalidData(format!(
                    "Character outside BMP not supported: {c1_char} or {c2_char}"
                )));
            }

            bigrams.push((
                c1_char as u16,
                c2_char as u16,
                (freq as f32 * weight).round() as u32,
            ));
        }
        self.frequencies.bigrams.entries = Arc::from(bigrams);
        Ok(())
    }

    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation
    )]
    fn parse_trigrams(&mut self, weight: f32, part: &[Value]) -> Result<(), ForgeError> {
        let mut trigrams = self.frequencies.trigrams.entries.to_vec();
        for e in part {
            let freq = e["freq"].as_u64().ok_or_else(|| {
                ForgeError::InvalidData(format!("Missing frequency in 3gram entry: {e:?}"))
            })?;
            let c1_char = e["char1"]
                .as_str()
                .and_then(Self::resolve_corpus_char)
                .ok_or_else(|| {
                    ForgeError::InvalidData(format!(
                        "Missing or invalid char1 in 3gram entry: {e:?}"
                    ))
                })?;
            let c2_char = e["char2"]
                .as_str()
                .and_then(Self::resolve_corpus_char)
                .ok_or_else(|| {
                    ForgeError::InvalidData(format!(
                        "Missing or invalid char2 in 3gram entry: {e:?}"
                    ))
                })?;
            let c3_char = e["char3"]
                .as_str()
                .and_then(Self::resolve_corpus_char)
                .ok_or_else(|| {
                    ForgeError::InvalidData(format!(
                        "Missing or invalid char3 in 3gram entry: {e:?}"
                    ))
                })?;

            if (c1_char as u32) > 0xFFFF || (c2_char as u32) > 0xFFFF || (c3_char as u32) > 0xFFFF {
                return Err(ForgeError::InvalidData(format!(
                    "Character outside BMP not supported: {c1_char}, {c2_char}, or {c3_char}"
                )));
            }

            trigrams.push((
                c1_char as u16,
                c2_char as u16,
                c3_char as u16,
                (freq as f32 * weight).round() as u32,
            ));
        }
        self.frequencies.trigrams.entries = Arc::from(trigrams);
        Ok(())
    }

    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation
    )]
    fn parse_words(&mut self, weight: f32, part: &[Value]) -> Result<(), ForgeError> {
        let mut words = self.frequencies.words.to_vec();
        for e in part {
            let freq = e["freq"].as_u64().ok_or_else(|| {
                ForgeError::InvalidData(format!("Missing frequency in word entry: {e:?}"))
            })?;
            if let Some(w) = e["word"].as_str() {
                words.push((w.to_string(), (freq as f32 * weight).round() as u32));
            }
        }
        self.frequencies.words = Arc::from(words);
        Ok(())
    }

    /// Resolves a corpus token string to a character.
    #[must_use]
    pub fn resolve_corpus_char(token: &str) -> Option<char> {
        for (key, val) in CORPUS_TOKEN_MAP {
            if token == *key {
                return Some((*val).to_ascii_lowercase());
            }
        }

        if token.len() >= 2
            && token.len().is_multiple_of(2)
            && token.chars().all(|c| c.is_ascii_hexdigit())
        {
            let mut bytes = Vec::with_capacity(token.len() / 2);
            for i in (0..token.len()).step_by(2) {
                if let Ok(byte) = u8::from_str_radix(&token[i..i + 2], 16) {
                    bytes.push(byte);
                } else {
                    return None;
                }
            }
            if let Ok(s) = String::from_utf8(bytes) {
                return s.chars().next().map(|c| c.to_ascii_lowercase());
            }
        }

        if token.chars().count() == 1 {
            token.chars().next().map(|c| c.to_ascii_lowercase())
        } else {
            None
        }
    }

    /// Injects synthetic data (Enter, Backspace) for standard prose corpora.
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    pub fn inject_synthetic_data(&mut self) {
        if !self.is_std {
            return;
        }

        let mut freqs = self.char_freqs.to_vec();
        let total_chars: u64 = freqs.iter().sum();
        let sentence_count: u64 = freqs['.' as usize] + freqs['?' as usize] + freqs['!' as usize];

        if total_chars == 0 {
            return;
        }

        let enter_count = (sentence_count as f32 / STD_CORPUS_SENTENCE_RATIO).round() as u64;
        let bksp_count = (total_chars as f32 * STD_CORPUS_ERROR_RATE * STD_CORPUS_BACKSPACE_FACTOR)
            .round() as u64;

        freqs['\n' as usize] += enter_count;
        freqs['\x08' as usize] += bksp_count;
        self.char_freqs = Arc::from(freqs);

        let mut bigrams = self.frequencies.bigrams.entries.to_vec();
        if bksp_count > 0 {
            let mut new_bigrams = Vec::new();
            for (char_code, &freq) in self.char_freqs.iter().enumerate() {
                if freq > 0 && char_code != '\x08' as usize && char_code != '\n' as usize {
                    let ratio = freq as f32 / total_chars as f32;
                    let share = (bksp_count as f32 * ratio).round() as u32;
                    if share > 0 {
                        new_bigrams.push((('\x08' as u16), char_code as u16, share));
                        new_bigrams.push((char_code as u16, ('\x08' as u16), share));
                    }
                }
            }
            bigrams.extend(new_bigrams);
        }

        if enter_count > 0 {
            let puncts = ['.', '?', '!'];
            let total_punct = sentence_count.max(1);

            for p in puncts {
                let p_freq = self.char_freqs[p as usize];
                if p_freq > 0 {
                    let ratio = p_freq as f32 / total_punct as f32;
                    let share = (enter_count as f32 * ratio).round() as u32;
                    if share > 0 {
                        bigrams.push((p as u16, '\n' as u16, share));
                    }
                }
            }
        }
        bigrams.sort_unstable_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
        self.frequencies.bigrams.entries = Arc::from(bigrams);

        let mut trigrams = self.frequencies.trigrams.entries.to_vec();
        if bksp_count > 0 {
            let total_bigrams: u64 = self
                .frequencies
                .bigrams
                .entries
                .iter()
                .map(|(_, _, f)| u64::from(*f))
                .sum();
            if total_bigrams > 0 {
                let mut new_trigrams = Vec::new();
                for (a, b, freq) in &*self.frequencies.bigrams.entries {
                    if *a == '\x08' as u16
                        || *b == '\x08' as u16
                        || *a == '\n' as u16
                        || *b == '\n' as u16
                    {
                        continue;
                    }

                    let ratio = *freq as f32 / total_bigrams as f32;
                    let share = (bksp_count as f32 * ratio).round() as u32;

                    if share > 0 {
                        new_trigrams.push((*a, *b, '\x08' as u16, share));
                    }
                }
                trigrams.extend(new_trigrams);
            }
        }

        if enter_count > 0 {
            let puncts = ['.', '?', '!'];
            let mut new_trigrams = Vec::new();

            let punct_bigrams: Vec<_> = self
                .frequencies
                .bigrams
                .entries
                .iter()
                .filter(|(_, b, _)| puncts.contains(&(*b as u8 as char)))
                .collect();

            let total_punct_bigrams: u64 =
                punct_bigrams.iter().map(|(_, _, f)| u64::from(*f)).sum();

            if total_punct_bigrams > 0 {
                for (a, b, freq) in punct_bigrams {
                    let ratio = *freq as f32 / total_punct_bigrams as f32;
                    let share = (enter_count as f32 * ratio).round() as u32;

                    if share > 0 {
                        new_trigrams.push((*a, *b, '\n' as u16, share));
                    }
                }
                trigrams.extend(new_trigrams);
            }
        }

        trigrams.sort_unstable_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));
        self.frequencies.trigrams.entries = Arc::from(trigrams);
    }
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;

    #[test]
    fn test_corpus_merger() {
        let mut base = Corpus::default();
        let other = Corpus {
            is_std: true,
            char_freqs: Arc::from(vec![100; MAX_KEYCODE_SPACE]),
            frequencies: FrequencyTables {
                bigrams: BigramFrequencyTable {
                    entries: Arc::from(vec![(1, 2, 50)]),
                },
                ..Default::default()
            },
        };

        CorpusMerger::merge(&mut base, &other, 0.5);

        assert_eq!(base.char_freqs[0], 50);
        assert_eq!(base.frequencies.bigrams.entries.len(), 1);
        assert_eq!(base.frequencies.bigrams.entries[0].2, 25);
    }
}
