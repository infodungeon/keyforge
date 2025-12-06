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
use crate::consts::KEY_CODE_RANGE;
use crate::error::KfResult;
use keyforge_protocol::config::{Config, LayoutDefinitions, ScoringWeights}; // UPDATED
use keyforge_protocol::geometry::KeyboardGeometry; // UPDATED

#[derive(Clone)]
pub struct Scorer {
    pub key_count: usize,
    pub weights: ScoringWeights,
    pub defs: LayoutDefinitions,
    pub geometry: KeyboardGeometry,

    pub tier_penalty_matrix: [[f32; 3]; 3],

    pub full_cost_matrix: Vec<f32>,
    pub raw_user_matrix: Vec<f32>,

    pub trigram_cost_table: Vec<f32>,

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

    pub words: Vec<(String, u64)>,

    pub char_tier_map: [u8; 256],
    pub critical_mask: [bool; 256],

    pub freq_matrix: Vec<f32>,

    pub active_chars: Vec<usize>,
}

impl Scorer {
    pub fn new(
        cost_path: &str,
        corpus_dir: &str,
        geometry: &KeyboardGeometry,
        config: Config,
        debug: bool,
    ) -> KfResult<Self> {
        ScorerBuildParams::load_from_disk(
            cost_path,
            corpus_dir,
            geometry.clone(),
            Some(config.weights),
            Some(config.defs),
            debug,
        )
    }

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

    pub fn estimate_memory_footprint(&self) -> usize {
        let tri_size = self.trigram_cost_table.len() * 4;
        let bi_size = self.full_cost_matrix.len() * 4;
        let vec_overhead = self.bigrams_others.len() * (1 + 4 + 1);
        tri_size + bi_size + vec_overhead
    }

    pub fn check_cache_fit(&self, available_l2_kb: usize) -> (bool, usize) {
        let hot_data_size = self.trigram_cost_table.len() * 4;
        let needed_kb = hot_data_size / 1024;
        let effective_limit = (available_l2_kb as f32 * 0.8) as usize;
        (needed_kb <= effective_limit, needed_kb)
    }
}
