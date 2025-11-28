use clap::Args;

#[derive(Args, Debug, Clone)]
pub struct Config {
    #[command(flatten)]
    pub search: SearchParams,
    #[command(flatten)]
    pub weights: ScoringWeights,
    #[command(flatten)]
    pub defs: LayoutDefinitions,
}

#[derive(Args, Debug, Clone)]
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

#[derive(Args, Debug, Clone)]
pub struct ScoringWeights {
    // === SFR (Repeats) ===
    #[arg(long, default_value_t = 20.0)]
    pub penalty_sfr_weak_finger: f32,
    #[arg(long, default_value_t = 25.0)]
    pub penalty_sfr_bad_row: f32,
    #[arg(long, default_value_t = 40.0)]
    pub penalty_sfr_lat: f32,

    // === SFB (Bigrams) ===

    // SZR35 TUNING: Lateral (Center Col) is worse than Bottom (Curl)
    // Previously 35.0 -> Now 65.0 (Hate the center reach)
    #[arg(long, default_value_t = 65.0)]
    pub penalty_sfb_lateral: f32,

    // Weak Finger Lateral (Pinky Stretch)
    #[arg(long, default_value_t = 250.0)]
    pub penalty_sfb_lateral_weak: f32,

    // Base SFB (Vertical)
    #[arg(long, default_value_t = 200.0)]
    pub penalty_sfb_base: f32,

    #[arg(long, default_value_t = 10.0)]
    pub penalty_sfb_outward_adder: f32,

    // Diagonal
    #[arg(long, default_value_t = 240.0)]
    pub penalty_sfb_diagonal: f32,

    // Long Jump (2-row)
    #[arg(long, default_value_t = 280.0)]
    pub penalty_sfb_long: f32,

    // SZR35 TUNING: Bottom SFB is cheaper because columns are staggered
    // Previously 110.0 -> Now 45.0 (Curling is okay)
    #[arg(long, default_value_t = 45.0)]
    pub penalty_sfb_bottom: f32,

    #[arg(long, default_value_t = 2.7)]
    pub weight_weak_finger_sfb: f32,

    // === OTHER ===
    #[arg(long, default_value_t = 25.0)]
    pub penalty_scissor: f32,
    #[arg(long, default_value_t = 1.3)]
    pub penalty_ring_pinky: f32,

    // Non-SFB Lateral Reaches (General hate for center column)
    #[arg(long, default_value_t = 50.0)]
    pub penalty_lateral: f32,

    // === FLOW ===
    #[arg(long, default_value_t = 20.0)]
    pub penalty_skip: f32,
    #[arg(long, default_value_t = 15.0)]
    pub penalty_redirect: f32,
    #[arg(long, default_value_t = 5.0)]
    pub penalty_hand_run: f32,
    #[arg(long, default_value_t = 60.0)]
    pub bonus_inward_roll: f32,
    #[arg(long, default_value_t = 30.0)]
    pub bonus_bigram_roll_in: f32,
    #[arg(long, default_value_t = 15.0)]
    pub bonus_bigram_roll_out: f32,

    // === TIER ===
    #[arg(long, default_value_t = 5.0)]
    pub penalty_high_in_med: f32,
    #[arg(long, default_value_t = 20.0)]
    pub penalty_high_in_low: f32,
    #[arg(long, default_value_t = 2.0)]
    pub penalty_med_in_prime: f32,
    #[arg(long, default_value_t = 10.0)]
    pub penalty_med_in_low: f32,
    #[arg(long, default_value_t = 15.0)]
    pub penalty_low_in_prime: f32,
    #[arg(long, default_value_t = 2.0)]
    pub penalty_low_in_med: f32,

    // === BALANCE & SYSTEM ===
    #[arg(long, default_value_t = 200.0)]
    pub penalty_imbalance: f32,
    #[arg(long, default_value_t = 0.55)]
    pub max_hand_imbalance: f32,

    // SZR35 TUNING: Distance matters less, Flow matters more
    #[arg(long, default_value_t = 1.0)]
    pub weight_geo_dist: f32,

    #[arg(long, default_value_t = 1.5)]
    pub weight_finger_effort: f32,

    #[arg(long, default_value_t = 200_000_000.0)]
    pub corpus_scale: f32,
    #[arg(long, default_value_t = 120.0)]
    pub default_cost_ms: f32,

    #[arg(long, default_value = "0.0,1.0,1.1,1.3,1.6")]
    pub finger_penalty_scale: String,
}

#[derive(Args, Debug, Clone)]
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

impl ScoringWeights {
    pub fn get_finger_penalty_scale(&self) -> [f32; 5] {
        parse_f32_array::<5>(&self.finger_penalty_scale, "finger_penalty_scale")
    }
}

impl LayoutDefinitions {
    pub fn get_finger_repeat_scale(&self) -> [f32; 5] {
        parse_f32_array::<5>(&self.finger_repeat_scale, "finger_repeat_scale")
    }

    pub fn get_critical_bigrams(&self) -> Vec<[u8; 2]> {
        self.critical_bigrams
            .split(',')
            .map(|s| {
                let b = s.trim().as_bytes();
                if b.len() != 2 {
                    panic!("Critical bigram '{}' is not 2 chars", s);
                }
                [b[0], b[1]]
            })
            .collect()
    }
}

fn parse_f32_array<const N: usize>(s: &str, name: &str) -> [f32; N] {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != N {
        panic!("--{} requires {} values", name, N);
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
