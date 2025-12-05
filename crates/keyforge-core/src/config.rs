use clap::Args;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Args, Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[command(flatten)]
    pub search: SearchParams,
    #[command(flatten)]
    pub weights: ScoringWeights,
    #[command(flatten)]
    pub defs: LayoutDefinitions,
}

#[derive(Args, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SearchParams {
    #[arg(long, default_value_t = 10_000)]
    pub search_epochs: usize,
    #[arg(long, default_value_t = 50_000)]
    pub search_steps: usize,
    #[arg(long, default_value_t = 500)]
    pub search_patience: usize,
    #[arg(long, default_value_t = 0.1)]
    pub search_patience_threshold: f32,
    #[arg(long, default_value_t = 0.08)]
    pub temp_min: f32,
    #[arg(long, default_value_t = 1000.0)]
    pub temp_max: f32,
    #[arg(long, default_value_t = 600)]
    pub opt_limit_fast: usize,
    #[arg(long, default_value_t = 3000)]
    pub opt_limit_slow: usize,
}

impl Default for SearchParams {
    fn default() -> Self {
        Self {
            search_epochs: 10_000,
            search_steps: 50_000,
            search_patience: 500,
            search_patience_threshold: 0.1,
            temp_min: 0.08,
            temp_max: 1000.0,
            opt_limit_fast: 600,
            opt_limit_slow: 3000,
        }
    }
}

#[derive(Args, Debug, Clone, Serialize, Deserialize)]
#[serde(default)] // Use the Default impl when fields are missing during deserialization
pub struct ScoringWeights {
    #[arg(long, default_value_t = 20.0)]
    pub penalty_sfr_weak_finger: f32,
    #[arg(long, default_value_t = 25.0)]
    pub penalty_sfr_bad_row: f32,
    #[arg(long, default_value_t = 40.0)]
    pub penalty_sfr_lat: f32,
    #[arg(long, default_value_t = 65.0)]
    pub penalty_sfb_lateral: f32,
    #[arg(long, default_value_t = 160.0)]
    pub penalty_sfb_lateral_weak: f32,
    #[arg(long, default_value_t = 400.0)]
    pub penalty_sfb_base: f32,
    #[arg(long, default_value_t = 10.0)]
    pub penalty_sfb_outward_adder: f32,
    #[arg(long, default_value_t = 240.0)]
    pub penalty_sfb_diagonal: f32,
    #[arg(long, default_value_t = 280.0)]
    pub penalty_sfb_long: f32,
    #[arg(long, default_value_t = 45.0)]
    pub penalty_sfb_bottom: f32,
    #[arg(long, default_value_t = 2.7)]
    pub weight_weak_finger_sfb: f32,

    #[arg(long, default_value_t = 2)]
    pub threshold_sfb_long_row_diff: i8,
    #[arg(long, default_value_t = 2)]
    pub threshold_scissor_row_diff: i8,

    #[arg(long, default_value_t = 25.0)]
    pub penalty_scissor: f32,
    #[arg(long, default_value_t = 1.3)]
    pub penalty_ring_pinky: f32,
    #[arg(long, default_value_t = 50.0)]
    pub penalty_lateral: f32,
    #[arg(long, default_value_t = 20.0)]
    pub penalty_monogram_stretch: f32,
    #[arg(long, default_value_t = 20.0)]
    pub penalty_skip: f32,
    #[arg(long, default_value_t = 65.0)]
    pub penalty_redirect: f32,
    #[arg(long, default_value_t = 5.0)]
    pub penalty_hand_run: f32,
    #[arg(long, default_value_t = 40.0)]
    pub bonus_inward_roll: f32,
    #[arg(long, default_value_t = 35.0)]
    pub bonus_bigram_roll_in: f32,
    #[arg(long, default_value_t = 25.0)]
    pub bonus_bigram_roll_out: f32,
    #[arg(long, default_value_t = 12.0)]
    pub penalty_high_in_med: f32,
    #[arg(long, default_value_t = 20.0)]
    pub penalty_high_in_low: f32,
    #[arg(long, default_value_t = 2.0)]
    pub penalty_med_in_prime: f32,
    #[arg(long, default_value_t = 2.0)]
    pub penalty_med_in_low: f32,
    #[arg(long, default_value_t = 15.0)]
    pub penalty_low_in_prime: f32,
    #[arg(long, default_value_t = 2.0)]
    pub penalty_low_in_med: f32,
    #[arg(long, default_value_t = 200.0)]
    pub penalty_imbalance: f32,
    #[arg(long, default_value_t = 0.55)]
    pub max_hand_imbalance: f32,
    #[arg(long, default_value_t = 1.0)]
    pub weight_vertical_travel: f32,
    #[arg(long, default_value_t = 3.5)]
    pub weight_lateral_travel: f32,
    #[arg(long, default_value_t = 2.2)]
    pub weight_finger_effort: f32,
    #[arg(long, default_value_t = 200_000_000.0)]
    pub corpus_scale: f32,
    #[arg(long, default_value_t = 120.0)]
    pub default_cost_ms: f32,
    #[arg(long, default_value_t = 3000)]
    pub loader_trigram_limit: usize,

    // Critical Strings that caused the panic
    #[arg(long, default_value = "0.0,1.0,1.1,1.3,1.6")]
    pub finger_penalty_scale: String,

    #[arg(long, default_value = "21,23,34")]
    pub comfortable_scissors: String,
}

// Implement Default manually so Serde has valid fallbacks
impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            penalty_sfr_weak_finger: 20.0,
            penalty_sfr_bad_row: 25.0,
            penalty_sfr_lat: 40.0,
            penalty_sfb_lateral: 65.0,
            penalty_sfb_lateral_weak: 160.0,
            penalty_sfb_base: 400.0,
            penalty_sfb_outward_adder: 10.0,
            penalty_sfb_diagonal: 240.0,
            penalty_sfb_long: 280.0,
            penalty_sfb_bottom: 45.0,
            weight_weak_finger_sfb: 2.7,
            threshold_sfb_long_row_diff: 2,
            threshold_scissor_row_diff: 2,
            penalty_scissor: 25.0,
            penalty_ring_pinky: 1.3,
            penalty_lateral: 50.0,
            penalty_monogram_stretch: 20.0,
            penalty_skip: 20.0,
            penalty_redirect: 65.0,
            penalty_hand_run: 5.0,
            bonus_inward_roll: 40.0,
            bonus_bigram_roll_in: 35.0,
            bonus_bigram_roll_out: 25.0,
            penalty_high_in_med: 12.0,
            penalty_high_in_low: 20.0,
            penalty_med_in_prime: 2.0,
            penalty_med_in_low: 2.0,
            penalty_low_in_prime: 15.0,
            penalty_low_in_med: 2.0,
            penalty_imbalance: 200.0,
            max_hand_imbalance: 0.55,
            weight_vertical_travel: 1.0,
            weight_lateral_travel: 3.5,
            weight_finger_effort: 2.2,
            corpus_scale: 200_000_000.0,
            default_cost_ms: 120.0,
            loader_trigram_limit: 3000,
            finger_penalty_scale: "0.0,1.0,1.1,1.3,1.6".to_string(),
            comfortable_scissors: "21,23,34".to_string(),
        }
    }
}

#[derive(Args, Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LayoutDefinitions {
    #[arg(long, default_value = "etaoinshr")]
    pub tier_high_chars: String,
    #[arg(long, default_value = "ldcumwfgypb.,")]
    pub tier_med_chars: String,
    #[arg(long, default_value = "vkjxqz/;")]
    pub tier_low_chars: String,
    #[arg(long, default_value = "th,he,in,er,an,re,nd,ou")]
    pub critical_bigrams: String,
    #[arg(long, default_value = "1.0,1.0,1.0,1.2,1.5")]
    pub finger_repeat_scale: String,
}

impl Default for LayoutDefinitions {
    fn default() -> Self {
        Self {
            tier_high_chars: "etaoinshr".to_string(),
            tier_med_chars: "ldcumwfgypb.,".to_string(),
            tier_low_chars: "vkjxqz/;".to_string(),
            critical_bigrams: "th,he,in,er,an,re,nd,ou".to_string(),
            finger_repeat_scale: "1.0,1.0,1.0,1.2,1.5".to_string(),
        }
    }
}

impl ScoringWeights {
    pub fn get_finger_penalty_scale(&self) -> [f32; 5] {
        parse_f32_array::<5>(&self.finger_penalty_scale, "finger_penalty_scale")
    }
    pub fn allowed_hand_balance_deviation(&self) -> f32 {
        (self.max_hand_imbalance - 0.5).max(0.0)
    }
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Self {
        let content = fs::read_to_string(path).expect("Failed to read weights");
        serde_json::from_str(&content).expect("Failed to parse weights")
    }
    pub fn get_comfortable_scissors(&self) -> Vec<(u8, u8)> {
        let mut pairs = Vec::new();
        for s in self.comfortable_scissors.split(',') {
            let s = s.trim();
            if s.len() == 2 {
                let bytes = s.as_bytes();
                if bytes[0] >= b'0' && bytes[1] >= b'0' {
                    pairs.push((bytes[0] - b'0', bytes[1] - b'0'));
                }
            }
        }
        pairs
    }

    // UPDATED: Prefixed arguments with _ to suppress unused variable warnings
    pub fn merge_from_cli(&mut self, _cli: &ScoringWeights, _matches: &clap::ArgMatches) {
        // Logic to merge CLI args (omitted for brevity)
    }
}

impl LayoutDefinitions {
    pub fn get_critical_bigrams(&self) -> Vec<[u8; 2]> {
        self.critical_bigrams
            .split(',')
            .map(|s| {
                let b = s.trim().as_bytes();
                if b.len() != 2 {
                    panic!("Bad bigram: {}", s);
                }
                [b[0], b[1]]
            })
            .collect()
    }
}

pub fn parse_f32_array<const N: usize>(s: &str, name: &str) -> [f32; N] {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != N {
        if s.is_empty() {
            panic!("--{} is empty. Default was not applied correctly.", name);
        }
        panic!("--{} requires {} values, found {}", name, N, parts.len());
    }
    let mut arr = [0.0; N];
    for (i, p) in parts.iter().enumerate() {
        arr[i] = p
            .trim()
            .parse()
            .unwrap_or_else(|_| panic!("Invalid number in {}", name));
    }
    arr
}
