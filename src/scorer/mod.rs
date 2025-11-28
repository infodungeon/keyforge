pub mod costs;
pub mod engine;
pub mod flow;
pub mod loader;
pub mod physics;
pub mod setup;
pub mod types;

use self::loader::TrigramRef;
pub use self::types::ScoreDetails;
use crate::config::{Config, LayoutDefinitions, ScoringWeights};
use crate::geometry::KeyboardGeometry;

pub struct Scorer {
    pub weights: ScoringWeights,
    pub defs: LayoutDefinitions,

    // Runtime definition of hardware (Standard or loaded from JSON)
    pub geometry: KeyboardGeometry,

    pub tier_penalty_matrix: [[f32; 3]; 3],

    // --- Data Tables (Fast Lookups) ---
    // User Data + Penalties (Used by Optimizer)
    pub full_cost_matrix: [[f32; 30]; 30],
    // Pure User Data (Used by Validator/Report)
    pub raw_user_matrix: [[f32; 30]; 30],

    pub trigram_cost_table: Vec<f32>,
    pub slot_monogram_costs: [f32; 30],
    pub finger_scales: [f32; 5],

    // --- N-Gram Data (CSR Format) ---
    pub bigram_starts: Vec<usize>,
    pub bigrams_others: Vec<u8>,
    pub bigrams_freqs: Vec<f32>,
    pub bigrams_self_first: Vec<bool>,

    pub trigram_starts: Vec<usize>,
    pub trigrams_flat: Vec<TrigramRef>,

    pub char_freqs: [f32; 256],

    // --- Mappings ---
    pub char_tier_map: [u8; 256],
    pub slot_tier_map: [u8; 30],
    pub critical_mask: [bool; 256],
    pub freq_matrix: [[f32; 256]; 256],
}

impl Scorer {
    pub fn new(
        cost_path: &str,
        ngrams_path: &str,
        geometry_path: &Option<String>,
        config: Config,
        debug: bool,
    ) -> Self {
        // Load geometry from file if provided, else use standard
        let geometry = if let Some(path) = geometry_path {
            if debug {
                println!("📐 Loading Geometry from: {}", path);
            }
            KeyboardGeometry::load_from_file(path)
        } else {
            if debug {
                println!("📐 Using Standard 30-key Geometry");
            }
            KeyboardGeometry::standard()
        };

        setup::build_scorer(
            cost_path,
            ngrams_path,
            config.weights,
            config.defs,
            geometry,
            debug,
        )
    }

    /// Optimized scoring for the search loop (Fast)
    pub fn score_full(&self, pos_map: &[u8; 256], limit: usize) -> (f32, f32, f32) {
        engine::score_full(self, pos_map, limit)
    }

    /// Detailed scoring for the validation report (Rich Data)
    pub fn score_debug(&self, pos_map: &[u8; 256], limit: usize) -> ScoreDetails {
        engine::score_debug(self, pos_map, limit)
    }
}
