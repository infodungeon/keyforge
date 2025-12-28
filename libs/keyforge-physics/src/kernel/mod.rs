pub mod compiler;
pub mod compute;
pub mod mechanics;
pub mod types;

use self::types::{FingerIndex, HandIndex};

/// The read-only, optimized mathematical context.
#[derive(Debug)]
#[allow(dead_code)]
pub struct EngineContext {
    pub(crate) key_count: usize,

    // --- GEOMETRY LOOKUPS (Indexed by Key Index) ---
    pub(crate) hands: Vec<HandIndex>,
    pub(crate) fingers: Vec<FingerIndex>,
    pub(crate) rows: Vec<i8>,
    pub(crate) cols: Vec<i8>,

    // --- COST MATRIX (Flattened) ---
    pub(crate) cost_matrix: Vec<i64>,

    // --- N-GRAM LOOKUPS (CSR + SoA Format) ---
    pub(crate) char_freqs: Vec<u32>,
    
    // Bigrams (Forward: c1 -> c2)
    pub(crate) bigram_starts: Vec<usize>,
    pub(crate) bigram_others: Vec<u16>,
    pub(crate) bigram_freqs: Vec<u32>,

    // Bigrams (Reverse: c2 -> c1)
    pub(crate) bigram_rev_starts: Vec<usize>,
    pub(crate) bigram_rev_others: Vec<u16>,
    pub(crate) bigram_rev_freqs: Vec<u32>,

    // Trigrams (Start: c1 -> c2, c3)
    pub(crate) trigram_starts: Vec<usize>,
    pub(crate) trigram_others1: Vec<u16>,
    pub(crate) trigram_others2: Vec<u16>,
    pub(crate) trigram_freqs: Vec<u32>,

    // Trigrams (Mid: c2 -> c1, c3)
    pub(crate) trigram_mid_starts: Vec<usize>,
    pub(crate) trigram_mid_others1: Vec<u16>,
    pub(crate) trigram_mid_others2: Vec<u16>,
    pub(crate) trigram_mid_freqs: Vec<u32>,

    // Trigrams (End: c3 -> c1, c2)
    pub(crate) trigram_end_starts: Vec<usize>,
    pub(crate) trigram_end_others1: Vec<u16>,
    pub(crate) trigram_end_others2: Vec<u16>,
    pub(crate) trigram_end_freqs: Vec<u32>,

    // --- RUBRIC CACHE ---
    pub(crate) penalty_redirect: i64,
    pub(crate) penalty_skip: i64,
    pub(crate) bonus_roll: i64,
}
