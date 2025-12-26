use serde::{Deserialize, Serialize};
pub mod keyboard;
pub mod layout;
pub mod corpus;
pub mod rubric;
pub mod keycodes;

pub use keyboard::{Keyboard, KeyNode};
pub use layout::Layout;
pub use corpus::Corpus;
pub use rubric::Rubric;
pub use keycodes::KeycodeRegistry;

// Re-add SearchConfig here if not present
#[derive(Debug, Clone)]
pub struct SearchConfig {
    pub steps: usize,
    pub start_temp: f32,
    pub end_temp: f32,
    pub seed: u64,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            steps: 100_000,
            start_temp: 100.0,
            end_temp: 0.01,
            seed: 42,
        }
    }
}

#[derive(Debug, Clone, Default)]
#[derive(Serialize, Deserialize)]
pub struct AnalysisReport {
    pub score: f32,
    pub distance: f32,
    pub sfb_total: f32,     // Total frequency of Same Finger Bigrams
    pub sfb_ratio: f32,     // SFB / Total Bigrams
    pub hand_balance: f32,  // -1.0 (Left) to 1.0 (Right). 0.0 is perfect.
    pub scissors: f32,      // Frequency of scissor actions
    pub redirects: f32,     // Frequency of redirects
    pub rolls: f32,         // Frequency of rolls
}
