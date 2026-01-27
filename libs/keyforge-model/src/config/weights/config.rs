// libs/keyforge-model/src/config/weights/config.rs

use super::constants::{
    DEFAULT_BONUS_BIGRAM_ROLL_IN, DEFAULT_BONUS_BIGRAM_ROLL_OUT, DEFAULT_BONUS_INWARD_ROLL,
    DEFAULT_COMFORTABLE_SCISSORS, DEFAULT_COST_MS, DEFAULT_FINGER_PENALTY_SCALE_ARRAY,
    DEFAULT_LOADER_TRIGRAM_LIMIT, DEFAULT_MAX_HAND_IMBALANCE, DEFAULT_PENALTY_HAND_RUN,
    DEFAULT_PENALTY_HIGH_IN_LOW, DEFAULT_PENALTY_HIGH_IN_MED, DEFAULT_PENALTY_IMBALANCE,
    DEFAULT_PENALTY_LATERAL, DEFAULT_PENALTY_LOW_IN_MED, DEFAULT_PENALTY_LOW_IN_PRIME,
    DEFAULT_PENALTY_MED_IN_LOW, DEFAULT_PENALTY_MED_IN_PRIME, DEFAULT_PENALTY_MONOGRAM_STRETCH,
    DEFAULT_PENALTY_REDIRECT, DEFAULT_PENALTY_RING_PINKY, DEFAULT_PENALTY_SCISSOR,
    DEFAULT_PENALTY_SFB_BASE, DEFAULT_PENALTY_SFB_BOTTOM, DEFAULT_PENALTY_SFB_DIAGONAL,
    DEFAULT_PENALTY_SFB_LATERAL, DEFAULT_PENALTY_SFB_LATERAL_WEAK, DEFAULT_PENALTY_SFB_LONG,
    DEFAULT_PENALTY_SFB_OUTWARD_ADDER, DEFAULT_PENALTY_SFR_BAD_ROW, DEFAULT_PENALTY_SFR_LAT,
    DEFAULT_PENALTY_SFR_WEAK_FINGER, DEFAULT_PENALTY_SKIP, DEFAULT_THRESHOLD_REACH_STRETCH,
    DEFAULT_THRESHOLD_SCISSOR_ROW_DIFF, DEFAULT_THRESHOLD_SFB_LONG_ROW_DIFF,
    DEFAULT_TRIGRAM_COVERAGE, DEFAULT_WEIGHT_FINGER_EFFORT, DEFAULT_WEIGHT_LATERAL_TRAVEL,
    DEFAULT_WEIGHT_VERTICAL_TRAVEL, DEFAULT_WEIGHT_WEAK_FINGER_SFB, MAX_LOADER_TRIGRAM_LIMIT,
    MAX_SAFE_WEIGHT,
};
use crate::config::metadata::{ParamType, ParameterMetadata};
use crate::types::Weight;
use crate::validator::Validator;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Weights and penalties defining the "personality" of the scoring engine.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(default)]
pub struct ScoringWeights {
    /// Dynamic weights map.
    #[serde(flatten)]
    pub weights: std::collections::HashMap<String, f32>,

    /// Finger penalty multipliers (Thumb, Index, Middle, Ring, Pinky).
    pub finger_penalty_scale: [f32; 5],
    /// Comma-separated list of comfortable scissor pairs.
    pub comfortable_scissors: String,
}

impl Default for ScoringWeights {
    fn default() -> Self {
        let mut weights = std::collections::HashMap::new();
        Self::insert_sfb_penalties(&mut weights);
        Self::insert_movement_penalties(&mut weights);
        Self::insert_dynamic_rules(&mut weights);
        Self::insert_loader_defaults(&mut weights);

        Self {
            weights,
            finger_penalty_scale: DEFAULT_FINGER_PENALTY_SCALE_ARRAY,
            comfortable_scissors: DEFAULT_COMFORTABLE_SCISSORS.to_string(),
        }
    }
}

impl ScoringWeights {
    fn insert_sfb_penalties(weights: &mut std::collections::HashMap<String, f32>) {
        weights.insert(
            "penalty_sfr_weak_finger".to_string(),
            DEFAULT_PENALTY_SFR_WEAK_FINGER,
        );
        weights.insert(
            "penalty_sfr_bad_row".to_string(),
            DEFAULT_PENALTY_SFR_BAD_ROW,
        );
        weights.insert("penalty_sfr_lat".to_string(), DEFAULT_PENALTY_SFR_LAT);
        weights.insert(
            "penalty_sfb_lateral".to_string(),
            DEFAULT_PENALTY_SFB_LATERAL,
        );
        weights.insert(
            "penalty_sfb_lateral_weak".to_string(),
            DEFAULT_PENALTY_SFB_LATERAL_WEAK,
        );
        weights.insert("penalty_sfb_base".to_string(), DEFAULT_PENALTY_SFB_BASE);
        weights.insert(
            "penalty_sfb_outward_adder".to_string(),
            DEFAULT_PENALTY_SFB_OUTWARD_ADDER,
        );
        weights.insert(
            "penalty_sfb_diagonal".to_string(),
            DEFAULT_PENALTY_SFB_DIAGONAL,
        );
        weights.insert("penalty_sfb_long".to_string(), DEFAULT_PENALTY_SFB_LONG);
        weights.insert("penalty_sfb_bottom".to_string(), DEFAULT_PENALTY_SFB_BOTTOM);
        weights.insert(
            "weight_weak_finger_sfb".to_string(),
            DEFAULT_WEIGHT_WEAK_FINGER_SFB,
        );
        weights.insert(
            "threshold_sfb_long_row_diff".to_string(),
            f32::from(DEFAULT_THRESHOLD_SFB_LONG_ROW_DIFF),
        );
    }

    fn insert_movement_penalties(weights: &mut std::collections::HashMap<String, f32>) {
        weights.insert(
            "threshold_scissor_row_diff".to_string(),
            f32::from(DEFAULT_THRESHOLD_SCISSOR_ROW_DIFF),
        );
        weights.insert(
            "threshold_reach_stretch".to_string(),
            DEFAULT_THRESHOLD_REACH_STRETCH,
        );
        weights.insert("penalty_scissor".to_string(), DEFAULT_PENALTY_SCISSOR);
        weights.insert("penalty_ring_pinky".to_string(), DEFAULT_PENALTY_RING_PINKY);
        weights.insert("penalty_lateral".to_string(), DEFAULT_PENALTY_LATERAL);
        weights.insert(
            "penalty_monogram_stretch".to_string(),
            DEFAULT_PENALTY_MONOGRAM_STRETCH,
        );
        weights.insert("penalty_skip".to_string(), DEFAULT_PENALTY_SKIP);
        weights.insert(
            "weight_vertical_travel".to_string(),
            DEFAULT_WEIGHT_VERTICAL_TRAVEL,
        );
        weights.insert(
            "weight_lateral_travel".to_string(),
            DEFAULT_WEIGHT_LATERAL_TRAVEL,
        );
        weights.insert(
            "weight_finger_effort".to_string(),
            DEFAULT_WEIGHT_FINGER_EFFORT,
        );
    }

    fn insert_dynamic_rules(weights: &mut std::collections::HashMap<String, f32>) {
        weights.insert("penalty_redirect".to_string(), DEFAULT_PENALTY_REDIRECT);
        weights.insert("penalty_hand_run".to_string(), DEFAULT_PENALTY_HAND_RUN);
        weights.insert("bonus_inward_roll".to_string(), DEFAULT_BONUS_INWARD_ROLL);
        weights.insert(
            "bonus_bigram_roll_in".to_string(),
            DEFAULT_BONUS_BIGRAM_ROLL_IN,
        );
        weights.insert(
            "bonus_bigram_roll_out".to_string(),
            DEFAULT_BONUS_BIGRAM_ROLL_OUT,
        );
        weights.insert(
            "penalty_high_in_med".to_string(),
            DEFAULT_PENALTY_HIGH_IN_MED,
        );
        weights.insert(
            "penalty_high_in_low".to_string(),
            DEFAULT_PENALTY_HIGH_IN_LOW,
        );
        weights.insert(
            "penalty_med_in_prime".to_string(),
            DEFAULT_PENALTY_MED_IN_PRIME,
        );
        weights.insert("penalty_med_in_low".to_string(), DEFAULT_PENALTY_MED_IN_LOW);
        weights.insert(
            "penalty_low_in_prime".to_string(),
            DEFAULT_PENALTY_LOW_IN_PRIME,
        );
        weights.insert("penalty_low_in_med".to_string(), DEFAULT_PENALTY_LOW_IN_MED);
        weights.insert("penalty_imbalance".to_string(), DEFAULT_PENALTY_IMBALANCE);
        weights.insert("max_hand_imbalance".to_string(), DEFAULT_MAX_HAND_IMBALANCE);
    }

    fn insert_loader_defaults(weights: &mut std::collections::HashMap<String, f32>) {
        weights.insert("default_cost_ms".to_string(), DEFAULT_COST_MS);
        #[allow(clippy::cast_precision_loss)]
        weights.insert(
            "loader_trigram_limit".to_string(),
            DEFAULT_LOADER_TRIGRAM_LIMIT as f32,
        );
        weights.insert("trigram_coverage".to_string(), DEFAULT_TRIGRAM_COVERAGE);
    }
}

impl Validator for ScoringWeights {
    fn validate(&self) -> Result<(), String> {
        if let Some(&limit) = self.weights.get("loader_trigram_limit") {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            if limit as usize > MAX_LOADER_TRIGRAM_LIMIT {
                return Err(format!(
                    "loader_trigram_limit exceeds safety maximum ({MAX_LOADER_TRIGRAM_LIMIT})"
                ));
            }
        }

        for (key, &val) in &self.weights {
            if val < 0.0 && !key.contains("bonus") {
                return Err(format!("Penalty weight '{key}' cannot be negative"));
            }
            if val > MAX_SAFE_WEIGHT {
                return Err(format!(
                    "Weight '{key}' exceeds safety maximum ({MAX_SAFE_WEIGHT:.0})"
                ));
            }
        }

        for (i, &w) in self.finger_penalty_scale.iter().enumerate() {
            if w < 0.0 {
                return Err(format!("finger_penalty_scale[{i}] cannot be negative"));
            }
        }
        Ok(())
    }
}

impl ScoringWeights {
    /// Returns the metadata schema for all available scoring weights.
    #[must_use]
    pub fn schema() -> Vec<ParameterMetadata> {
        vec![
            ParameterMetadata {
                key: "penalty_sfb_base".to_string(),
                label: "SFB Base Penalty".to_string(),
                description: "Basic cost for any Same Finger Bigram.".to_string(),
                param_type: ParamType::Float,
                min: Some(0.0),
                max: Some(MAX_SAFE_WEIGHT),
                default: DEFAULT_PENALTY_SFB_BASE,
            },
            ParameterMetadata {
                key: "penalty_scissor".to_string(),
                label: "Scissor Penalty".to_string(),
                description: "Penalty for adjacent finger stretches.".to_string(),
                param_type: ParamType::Float,
                min: Some(0.0),
                max: Some(MAX_SAFE_WEIGHT),
                default: DEFAULT_PENALTY_SCISSOR,
            },
            ParameterMetadata {
                key: "weight_vertical_travel".to_string(),
                label: "Vertical Travel Weight".to_string(),
                description: "Multiplier for finger movement distance.".to_string(),
                param_type: ParamType::Float,
                min: Some(0.0),
                max: Some(10.0),
                default: DEFAULT_WEIGHT_VERTICAL_TRAVEL,
            },
        ]
    }

    /// Retrieves a weight by key, falling back to a default value if not found.
    #[must_use]
    pub fn get_weight(&self, key: &str, default: f32) -> Weight {
        Weight(self.weights.get(key).copied().unwrap_or(default))
    }

    /// Gets the penalty for Same Finger Repeat on a weak finger.
    #[must_use]
    pub fn get_penalty_sfr_weak_finger(&self) -> Weight {
        self.get_weight("penalty_sfr_weak_finger", DEFAULT_PENALTY_SFR_WEAK_FINGER)
    }
    /// Gets the penalty for Same Finger Repeat involving a bad row jump.
    #[must_use]
    pub fn get_penalty_sfr_bad_row(&self) -> Weight {
        self.get_weight("penalty_sfr_bad_row", DEFAULT_PENALTY_SFR_BAD_ROW)
    }
    /// Gets the penalty for lateral Same Finger Repeat.
    #[must_use]
    pub fn get_penalty_sfr_lat(&self) -> Weight {
        self.get_weight("penalty_sfr_lat", DEFAULT_PENALTY_SFR_LAT)
    }
    /// Gets the penalty for lateral Same Finger Bigram.
    #[must_use]
    pub fn get_penalty_sfb_lateral(&self) -> Weight {
        self.get_weight("penalty_sfb_lateral", DEFAULT_PENALTY_SFB_LATERAL)
    }
    /// Gets the penalty for lateral SFB on a weak finger.
    #[must_use]
    pub fn get_penalty_sfb_lateral_weak(&self) -> Weight {
        self.get_weight("penalty_sfb_lateral_weak", DEFAULT_PENALTY_SFB_LATERAL_WEAK)
    }
    /// Gets the base penalty for any Same Finger Bigram.
    #[must_use]
    pub fn get_penalty_sfb_base(&self) -> Weight {
        self.get_weight("penalty_sfb_base", DEFAULT_PENALTY_SFB_BASE)
    }
    /// Gets the additional penalty for outward rolling SFBs.
    #[must_use]
    pub fn get_penalty_sfb_outward_adder(&self) -> Weight {
        self.get_weight(
            "penalty_sfb_outward_adder",
            DEFAULT_PENALTY_SFB_OUTWARD_ADDER,
        )
    }
    /// Gets the penalty for diagonal SFBs.
    #[must_use]
    pub fn get_penalty_sfb_diagonal(&self) -> Weight {
        self.get_weight("penalty_sfb_diagonal", DEFAULT_PENALTY_SFB_DIAGONAL)
    }
    /// Gets the penalty for long-distance SFBs.
    #[must_use]
    pub fn get_penalty_sfb_long(&self) -> Weight {
        self.get_weight("penalty_sfb_long", DEFAULT_PENALTY_SFB_LONG)
    }
    /// Gets the penalty for bottom-row SFBs.
    #[must_use]
    pub fn get_penalty_sfb_bottom(&self) -> Weight {
        self.get_weight("penalty_sfb_bottom", DEFAULT_PENALTY_SFB_BOTTOM)
    }
    /// Gets the multiplier for SFBs on weak fingers.
    #[must_use]
    pub fn get_weight_weak_finger_sfb(&self) -> Weight {
        self.get_weight("weight_weak_finger_sfb", DEFAULT_WEIGHT_WEAK_FINGER_SFB)
    }

    /// Gets the row difference threshold for "long" SFBs.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn get_threshold_sfb_long_row_diff(&self) -> i8 {
        self.get_weight(
            "threshold_sfb_long_row_diff",
            f32::from(DEFAULT_THRESHOLD_SFB_LONG_ROW_DIFF),
        )
        .to_f32() as i8
    }
    /// Gets the row difference threshold for scissors.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn get_threshold_scissor_row_diff(&self) -> i8 {
        self.get_weight(
            "threshold_scissor_row_diff",
            f32::from(DEFAULT_THRESHOLD_SCISSOR_ROW_DIFF),
        )
        .to_f32() as i8
    }
    /// Gets the distance threshold for reach stretches.
    #[must_use]
    pub fn get_threshold_reach_stretch(&self) -> Weight {
        self.get_weight("threshold_reach_stretch", DEFAULT_THRESHOLD_REACH_STRETCH)
    }

    /// Gets the penalty for scissor (adjacent finger stretch) movements.
    #[must_use]
    pub fn get_penalty_scissor(&self) -> Weight {
        self.get_weight("penalty_scissor", DEFAULT_PENALTY_SCISSOR)
    }
    /// Gets the penalty for ring-pinky interactions.
    #[must_use]
    pub fn get_penalty_ring_pinky(&self) -> Weight {
        self.get_weight("penalty_ring_pinky", DEFAULT_PENALTY_RING_PINKY)
    }
    /// Gets the penalty for lateral movement.
    #[must_use]
    pub fn get_penalty_lateral(&self) -> Weight {
        self.get_weight("penalty_lateral", DEFAULT_PENALTY_LATERAL)
    }
    /// Gets the penalty for single-key stretches.
    #[must_use]
    pub fn get_penalty_monogram_stretch(&self) -> Weight {
        self.get_weight("penalty_monogram_stretch", DEFAULT_PENALTY_MONOGRAM_STRETCH)
    }
    /// Gets the penalty for skipping a key (hurdle).
    #[must_use]
    pub fn get_penalty_skip(&self) -> Weight {
        self.get_weight("penalty_skip", DEFAULT_PENALTY_SKIP)
    }
    /// Gets the penalty for redirecting flow (e.g., Left -> Right -> Left).
    #[must_use]
    pub fn get_penalty_redirect(&self) -> Weight {
        self.get_weight("penalty_redirect", DEFAULT_PENALTY_REDIRECT)
    }
    /// Gets the penalty for excessive hand alternation runs.
    #[must_use]
    pub fn get_penalty_hand_run(&self) -> Weight {
        self.get_weight("penalty_hand_run", DEFAULT_PENALTY_HAND_RUN)
    }
    /// Gets the bonus (negative cost) for inward rolls.
    #[must_use]
    pub fn get_bonus_inward_roll(&self) -> Weight {
        self.get_weight("bonus_inward_roll", DEFAULT_BONUS_INWARD_ROLL)
    }
    /// Gets the bonus for specific bigram inward rolls.
    #[must_use]
    pub fn get_bonus_bigram_roll_in(&self) -> Weight {
        self.get_weight("bonus_bigram_roll_in", DEFAULT_BONUS_BIGRAM_ROLL_IN)
    }
    /// Gets the bonus for specific bigram outward rolls.
    #[must_use]
    pub fn get_bonus_bigram_roll_out(&self) -> Weight {
        self.get_weight("bonus_bigram_roll_out", DEFAULT_BONUS_BIGRAM_ROLL_OUT)
    }

    /// Gets the penalty for high-frequency keys in medium slots.
    #[must_use]
    pub fn get_penalty_high_in_med(&self) -> Weight {
        self.get_weight("penalty_high_in_med", DEFAULT_PENALTY_HIGH_IN_MED)
    }
    /// Gets the penalty for high-frequency keys in low slots.
    #[must_use]
    pub fn get_penalty_high_in_low(&self) -> Weight {
        self.get_weight("penalty_high_in_low", DEFAULT_PENALTY_HIGH_IN_LOW)
    }
    /// Gets the penalty for medium-frequency keys in prime slots.
    #[must_use]
    pub fn get_penalty_med_in_prime(&self) -> Weight {
        self.get_weight("penalty_med_in_prime", DEFAULT_PENALTY_MED_IN_PRIME)
    }
    /// Gets the penalty for medium-frequency keys in low slots.
    #[must_use]
    pub fn get_penalty_med_in_low(&self) -> Weight {
        self.get_weight("penalty_med_in_low", DEFAULT_PENALTY_MED_IN_LOW)
    }
    /// Gets the penalty for low-frequency keys in prime slots.
    #[must_use]
    pub fn get_penalty_low_in_prime(&self) -> Weight {
        self.get_weight("penalty_low_in_prime", DEFAULT_PENALTY_LOW_IN_PRIME)
    }
    /// Gets the penalty for low-frequency keys in medium slots.
    #[must_use]
    pub fn get_penalty_low_in_med(&self) -> Weight {
        self.get_weight("penalty_low_in_med", DEFAULT_PENALTY_LOW_IN_MED)
    }

    /// Gets the penalty for hand imbalance.
    #[must_use]
    pub fn get_penalty_imbalance(&self) -> Weight {
        self.get_weight("penalty_imbalance", DEFAULT_PENALTY_IMBALANCE)
    }
    /// Gets the maximum allowed hand imbalance ratio.
    #[must_use]
    pub fn get_max_hand_imbalance(&self) -> Weight {
        self.get_weight("max_hand_imbalance", DEFAULT_MAX_HAND_IMBALANCE)
    }

    /// Gets the weight multiplier for vertical travel distance.
    #[must_use]
    pub fn get_weight_vertical_travel(&self) -> Weight {
        self.get_weight("weight_vertical_travel", DEFAULT_WEIGHT_VERTICAL_TRAVEL)
    }
    /// Gets the weight multiplier for lateral travel distance.
    #[must_use]
    pub fn get_weight_lateral_travel(&self) -> Weight {
        self.get_weight("weight_lateral_travel", DEFAULT_WEIGHT_LATERAL_TRAVEL)
    }
    /// Gets the weight multiplier for finger effort.
    #[must_use]
    pub fn get_weight_finger_effort(&self) -> Weight {
        self.get_weight("weight_finger_effort", DEFAULT_WEIGHT_FINGER_EFFORT)
    }

    /// Gets the default cost in milliseconds (if using time-based scoring).
    #[must_use]
    pub fn get_default_cost_ms(&self) -> Weight {
        self.get_weight("default_cost_ms", DEFAULT_COST_MS)
    }
    /// Gets the limit on the number of trigrams to load.
    #[must_use]
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        clippy::cast_sign_loss
    )]
    pub fn get_loader_trigram_limit(&self) -> usize {
        self.get_weight("loader_trigram_limit", DEFAULT_LOADER_TRIGRAM_LIMIT as f32)
            .to_f32() as usize
    }
    /// Gets the required trigram coverage.
    #[must_use]
    pub fn get_trigram_coverage(&self) -> Weight {
        self.get_weight("trigram_coverage", DEFAULT_TRIGRAM_COVERAGE)
    }

    /// Returns the finger penalty scale array.
    #[must_use]
    pub fn get_finger_penalty_scale(&self) -> [f32; 5] {
        self.finger_penalty_scale
    }
    /// Calculates the allowed deviation from perfect hand balance.
    #[must_use]
    pub fn allowed_hand_balance_deviation(&self) -> f32 {
        (self.get_max_hand_imbalance().to_f32() - 0.5).max(0.0)
    }
    /// Parses the comfortable scissors string into a list of pairs.
    #[must_use]
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
}

/// CLI arguments mirroring `ScoringWeights`.
#[cfg(feature = "cli")]
#[derive(clap::Args, Debug, Clone)]
pub struct ScoringWeightsConfig {
    /// Generic scoring weights in KEY=VALUE format.
    /// Example: --weight `penalty_sfb_base=500.0` --weight `penalty_lateral=25.0`
    #[arg(long = "weight", value_parser = crate::config::utils::parse_key_val)]
    pub overrides: Vec<(String, f32)>,

    /// Finger penalty multipliers (Thumb, Index, Middle, Ring, Pinky).
    #[arg(long, value_delimiter = ',', num_args = 5, default_values_t = DEFAULT_FINGER_PENALTY_SCALE_ARRAY)]
    pub finger_penalty_scale: Vec<f32>,

    /// Comma-separated list of comfortable scissor pairs.
    #[arg(long, default_value = DEFAULT_COMFORTABLE_SCISSORS)]
    pub comfortable_scissors: String,
}

#[cfg(feature = "cli")]
impl TryFrom<ScoringWeightsConfig> for ScoringWeights {
    type Error = String;
    fn try_from(args: ScoringWeightsConfig) -> Result<Self, Self::Error> {
        let mut w = ScoringWeights::default();

        // Overlay dynamic CLI overrides
        for (key, value) in args.overrides {
            w.weights.insert(key, value);
        }

        w.finger_penalty_scale = vec_to_array_5(&args.finger_penalty_scale)?;
        w.comfortable_scissors = args.comfortable_scissors;

        w.validate()?;
        Ok(w)
    }
}

#[cfg(feature = "cli")]
fn vec_to_array_5(v: &[f32]) -> Result<[f32; 5], String> {
    if v.len() != 5 {
        return Err(format!("Expected 5 values, got {}", v.len()));
    }
    let mut arr = [0.0; 5];
    for (i, &val) in v.iter().enumerate() {
        arr[i] = val;
    }
    Ok(arr)
}
