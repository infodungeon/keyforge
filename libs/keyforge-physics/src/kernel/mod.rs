pub mod compiler;
pub mod compute;
pub mod mechanics;
pub mod types;

use self::types::{KeyCode, FingerIndex, HandIndex, RowIndex, ColIndex, Score};

#[derive(Debug)]
#[allow(dead_code)]
pub struct EngineContext {
    pub(crate) key_count: usize,
    pub(crate) hands: Vec<HandIndex>,
    pub(crate) fingers: Vec<FingerIndex>,
    pub(crate) rows: Vec<RowIndex>,
    pub(crate) cols: Vec<ColIndex>,
    pub(crate) cost_matrix: Vec<Score>,
    pub(crate) char_freqs: Vec<u32>,
    pub(crate) bigram_starts: Vec<usize>,
    pub(crate) bigram_others: Vec<KeyCode>,
    pub(crate) bigram_freqs: Vec<u32>,
    pub(crate) bigram_rev_starts: Vec<usize>,
    pub(crate) bigram_rev_others: Vec<KeyCode>,
    pub(crate) bigram_rev_freqs: Vec<u32>,
    pub(crate) trigram_starts: Vec<usize>,
    pub(crate) trigram_others1: Vec<KeyCode>,
    pub(crate) trigram_others2: Vec<KeyCode>,
    pub(crate) trigram_freqs: Vec<u32>,
    pub(crate) trigram_mid_starts: Vec<usize>,
    pub(crate) trigram_mid_others1: Vec<KeyCode>,
    pub(crate) trigram_mid_others2: Vec<KeyCode>,
    pub(crate) trigram_mid_freqs: Vec<u32>,
    pub(crate) trigram_end_starts: Vec<usize>,
    pub(crate) trigram_end_others1: Vec<KeyCode>,
    pub(crate) trigram_end_others2: Vec<KeyCode>,
    pub(crate) trigram_end_freqs: Vec<u32>,
    pub(crate) penalty_redirect: Score,
    pub(crate) penalty_skip: Score,
    pub(crate) bonus_roll: Score,
}
