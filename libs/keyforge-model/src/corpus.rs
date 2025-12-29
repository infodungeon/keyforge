use crate::error::KeyForgeError;
use crate::serde_utils::deserialize_limited_vec;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Corpus {
    #[serde(deserialize_with = "deserialize_limited_vec")]
    pub char_freqs: Vec<u32>,
    #[serde(deserialize_with = "deserialize_limited_vec")]
    pub bigrams: Vec<(u16, u16, u32)>,
    #[serde(deserialize_with = "deserialize_limited_vec")]
    pub trigrams: Vec<(u16, u16, u16, u32)>,
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

impl Corpus {
    /// Validates the integrity of the Corpus.
    /// Ensures that frequency maps are sized correctly to prevent panics in the Physics engine.
    pub fn validate(&self) -> Result<(), KeyForgeError> {
        // 1. Char Freqs must cover full u16 range (0..65535)
        // The physics engine uses direct indexing: ctx.char_freqs[code as usize]
        if self.char_freqs.len() != 65536 {
            return Err(KeyForgeError::InvalidData(format!(
                "Corpus char_freqs length must be 65536, found {}",
                self.char_freqs.len()
            )));
        }

        // 2. Bigrams/Trigrams
        // Since u16 indices are always < 65536, they are safe indices into char_freqs.
        // We could check for duplicates here if strictness is required, but O(N) validation is preferred.
        
        Ok(())
    }
}
