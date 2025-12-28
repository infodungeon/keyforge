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
