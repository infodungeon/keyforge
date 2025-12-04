pub mod builder;
pub mod costs;
pub mod engine;
pub mod flow;
pub mod loader;
pub mod metrics;
pub mod physics;
pub mod types;

pub use self::builder::ScorerBuildParams;
use self::loader::TrigramRef;
pub use self::types::ScoreDetails;
use crate::config::{LayoutDefinitions, ScoringWeights};
use crate::consts::KEY_CODE_RANGE;
// FIXED: Removed unused Config and KfResult imports
use crate::geometry::KeyboardGeometry;

#[derive(Clone)]
pub struct Scorer {
    pub key_count: usize,
    pub weights: ScoringWeights,
    pub defs: LayoutDefinitions,
    pub geometry: KeyboardGeometry,

    pub tier_penalty_matrix: [[f32; 3]; 3],

    // FLATTENED MATRICES (size = key_count * key_count)
    pub full_cost_matrix: Vec<f32>,
    pub raw_user_matrix: Vec<f32>,

    // FLATTENED TRIGRAMS (size = key_count^3)
    pub trigram_cost_table: Vec<f32>,

    // DYNAMIC ARRAYS (size = key_count)
    pub slot_monogram_costs: Vec<f32>,
    pub slot_tier_map: Vec<u8>,

    pub finger_scales: [f32; 5],

    pub bigram_starts: Vec<usize>,
    pub bigrams_others: Vec<u8>,
    pub bigrams_freqs: Vec<f32>,
    pub bigrams_self_first: Vec<bool>,
    pub trigram_starts: Vec<usize>,
    pub trigrams_flat: Vec<TrigramRef>,
    pub char_freqs: [f32; 256],
    pub char_tier_map: [u8; 256],
    pub critical_mask: [bool; 256],

    pub freq_matrix: Vec<f32>,

    pub active_chars: Vec<usize>,
}

impl Scorer {
    // Note: 'fn new' is implemented in scorer/builder.rs

    /// Calculates the score for a specific layout mapping.
    pub fn score_full(&self, pos_map: &[u8; KEY_CODE_RANGE], limit: usize) -> (f32, f32, f32) {
        engine::score_full(self, pos_map, limit)
    }

    pub fn score_details(&self, pos_map: &[u8; KEY_CODE_RANGE], limit: usize) -> ScoreDetails {
        engine::score_details(self, pos_map, limit)
    }

    pub fn get_element_costs(&self, pos_map: &[u8; KEY_CODE_RANGE]) -> Vec<f32> {
        engine::calculate_key_costs(self, pos_map)
    }

    #[inline(always)]
    pub fn idx(&self, row: usize, col: usize) -> usize {
        row * self.key_count + col
    }
}
