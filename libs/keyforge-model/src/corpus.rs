use crate::serde_utils::deserialize_limited_vec;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Corpus {
    #[serde(with = "BigArray")]
    pub char_freqs: [u32; 256],
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
            char_freqs: [0; 256],
            bigrams: Vec::new(),
            trigrams: Vec::new(),
            words: Vec::new(),
        }
    }
}
