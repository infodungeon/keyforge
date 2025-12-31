// Copyright (c) 2025 KeyForge Contributors
//
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
use crate::serde_utils::deserialize_limited_vec;
use serde::{Deserialize, Serialize};
use crate::validator::Validator;

/// Represents the statistical data of a language or text source.
/// Contains frequency data for characters, bigrams, and trigrams.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Corpus {
    /// Frequency of each character (index = char code).
    /// Must be exactly 65536 elements long to cover all u16 values.
    #[serde(deserialize_with = "deserialize_limited_vec")]
    pub char_freqs: Vec<u32>,
    /// List of bigrams (char1, char2, frequency).
    #[serde(deserialize_with = "deserialize_limited_vec")]
    pub bigrams: Vec<(u16, u16, u32)>,
    /// List of trigrams (char1, char2, char3, frequency).
    #[serde(deserialize_with = "deserialize_limited_vec")]
    pub trigrams: Vec<(u16, u16, u16, u32)>,
    /// List of common words and their frequencies.
    #[serde(deserialize_with = "deserialize_limited_vec")]
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
        // We could check for duplicates here if strictness is required, but O(N) validation is preferred.
        
        Ok(())
    }

    // Internal helper to keep the ForgeError return type for existing callers
    fn validate_internal(&self) -> Result<(), ForgeError> {
        self.validate()
    }
}
