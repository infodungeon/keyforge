// libs/keyforge-model/src/corpus.rs

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


//! Text corpus data structures and validation.
//!
//! A `Corpus` provides the statistical foundation for layout optimization,
//! including character, bigram, and trigram frequencies.

use crate::error::ForgeError;
use serde::{Deserialize, Serialize};
use crate::validator::Validator;

/// Represents the statistical data of a language or text source.
/// Contains frequency data for characters, bigrams, and trigrams.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Corpus {
    /// Frequency of each character (index = char code).
    /// Must be exactly 65536 elements long to cover all u16 values.
    /// Changed to u64 to support large corpora (>4B chars).
    pub char_freqs: Vec<u64>,
    /// List of bigrams (char1, char2, frequency).
    pub bigrams: Vec<(u16, u16, u32)>,
    /// List of trigrams (char1, char2, char3, frequency).
    pub trigrams: Vec<(u16, u16, u16, u32)>,
    /// List of common words and their frequencies.
    pub words: Vec<(String, u32)>,
}

impl Default for Corpus {
    fn default() -> Self {
        Self {
            char_freqs: vec![0; 65536],
            bigrams: Vec::new(),
            trigrams: Vec::new(),
            words: Vec::new(),
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
    pub fn validate(&self) -> Result<(), ForgeError> {
        // 1. Char Freqs must cover full u16 range (0..65535)
        // The physics engine uses direct indexing: ctx.char_freqs[code as usize]
        if self.char_freqs.len() != 65536 {
            return Err(ForgeError::InvalidData(format!(
                "Corpus char_freqs length must be 65536, found {}",
                self.char_freqs.len()
            )));
        }

        // 2. Bigrams/Trigrams
        // Since u16 indices are always < 65536, they are safe indices into char_freqs.
        
        Ok(())
    }

    /// Merges another corpus into this one with a specific weight.
    pub fn merge(&mut self, other: &Self, weight: f32) {
        // 1. Merge character frequencies
        for (i, &freq) in other.char_freqs.iter().enumerate() {
            if i < self.char_freqs.len() {
                self.char_freqs[i] += (freq as f32 * weight).round() as u64;
            }
        }

        // 2. Merge bigrams
        for &(c1, c2, freq) in &other.bigrams {
            self.bigrams.push((c1, c2, (freq as f32 * weight).round() as u32));
        }

        // 3. Merge trigrams
        for &(c1, c2, c3, freq) in &other.trigrams {
            self.trigrams.push((c1, c2, c3, (freq as f32 * weight).round() as u32));
        }

        // 4. Merge words
        for (word, freq) in &other.words {
            self.words.push((word.clone(), (*freq as f32 * weight).round() as u32));
        }

        // Keep bigrams/trigrams sorted for the engine
        self.bigrams.sort_unstable_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
        self.trigrams.sort_unstable_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));
    }

    // Internal helper to keep the ForgeError return type for existing callers
    fn validate_internal(&self) -> Result<(), ForgeError> {
        self.validate()
    }
}