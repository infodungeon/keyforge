// libs/keyforge-model/src/config/weights.rs

use crate::config::metadata::{ParamType, ParameterMetadata};
use crate::types::Weight;
use crate::validator::Validator;
use serde::{Deserialize, Serialize};
#[cfg(feature = "ts_bindings")]
use ts_rs::TS;
use utoipa::ToSchema;

// --- Default Values (Scoring) ---

/// Maximum safe weight value to prevent overflow.
pub const MAX_SAFE_WEIGHT: f32 = 100_000_000.0;
/// Maximum number of trigrams to load from corpus.
pub const MAX_LOADER_TRIGRAM_LIMIT: usize = 50_000;

/// Default penalty for Same Finger Repeat on a weak finger.
pub const DEFAULT_PENALTY_SFR_WEAK_FINGER: f32 = 20.0;
/// Default penalty for Same Finger Repeat involving a bad row jump.
pub const DEFAULT_PENALTY_SFR_BAD_ROW: f32 = 25.0;
/// Default penalty for lateral Same Finger Repeat.
pub const DEFAULT_PENALTY_SFR_LAT: f32 = 40.0;
/// Default penalty for lateral Same Finger Bigram.
pub const DEFAULT_PENALTY_SFB_LATERAL: f32 = 65.0;
/// Default penalty for lateral SFB on a weak finger.
pub const DEFAULT_PENALTY_SFB_LATERAL_WEAK: f32 = 160.0;
/// Default base penalty for any Same Finger Bigram.
pub const DEFAULT_PENALTY_SFB_BASE: f32 = 400.0;
/// Default additional penalty for outward rolling SFBs.
pub const DEFAULT_PENALTY_SFB_OUTWARD_ADDER: f32 = 10.0;
/// Default penalty for diagonal SFBs.
pub const DEFAULT_PENALTY_SFB_DIAGONAL: f32 = 240.0;
/// Default penalty for long-distance SFBs.
pub const DEFAULT_PENALTY_SFB_LONG: f32 = 280.0;
/// Default penalty for bottom-row SFBs.
pub const DEFAULT_PENALTY_SFB_BOTTOM: f32 = 45.0;
/// Default multiplier for SFBs on weak fingers.
pub const DEFAULT_WEIGHT_WEAK_FINGER_SFB: f32 = 2.7;

/// Default row difference threshold for "long" SFBs.
pub const DEFAULT_THRESHOLD_SFB_LONG_ROW_DIFF: i8 = 2;
/// Default row difference threshold for scissors.
pub const DEFAULT_THRESHOLD_SCISSOR_ROW_DIFF: i8 = 2;
/// Default distance threshold for reach stretches.
pub const DEFAULT_THRESHOLD_REACH_STRETCH: f32 = 1.2;

/// Default penalty for scissor (adjacent finger stretch) movements.
pub const DEFAULT_PENALTY_SCISSOR: f32 = 25.0;
/// Default penalty for ring-pinky interactions.
pub const DEFAULT_PENALTY_RING_PINKY: f32 = 1.3;
/// Default penalty for lateral movement.
pub const DEFAULT_PENALTY_LATERAL: f32 = 50.0;
/// Default penalty for single-key stretches.
pub const DEFAULT_PENALTY_MONOGRAM_STRETCH: f32 = 20.0;
/// Default penalty for skipping a key (hurdle).
pub const DEFAULT_PENALTY_SKIP: f32 = 20.0;
/// Default penalty for redirecting flow (e.g., Left -> Right -> Left).
pub const DEFAULT_PENALTY_REDIRECT: f32 = 65.0;
/// Default penalty for excessive hand alternation runs.
pub const DEFAULT_PENALTY_HAND_RUN: f32 = 5.0;
/// Default bonus (negative cost) for inward rolls.
pub const DEFAULT_BONUS_INWARD_ROLL: f32 = 40.0;
/// Default bonus for specific bigram inward rolls.
pub const DEFAULT_BONUS_BIGRAM_ROLL_IN: f32 = 35.0;
/// Default bonus for specific bigram outward rolls.
pub const DEFAULT_BONUS_BIGRAM_ROLL_OUT: f32 = 25.0;
/// Default penalty for high-frequency keys in medium slots.
pub const DEFAULT_PENALTY_HIGH_IN_MED: f32 = 12.0;
/// Default penalty for high-frequency keys in low slots.
pub const DEFAULT_PENALTY_HIGH_IN_LOW: f32 = 20.0;
/// Default penalty for medium-frequency keys in prime slots.
pub const DEFAULT_PENALTY_MED_IN_PRIME: f32 = 2.0;
/// Default penalty for medium-frequency keys in low slots.
pub const DEFAULT_PENALTY_MED_IN_LOW: f32 = 2.0;
/// Default penalty for low-frequency keys in prime slots.
pub const DEFAULT_PENALTY_LOW_IN_PRIME: f32 = 15.0;
/// Default penalty for low-frequency keys in medium slots.
pub const DEFAULT_PENALTY_LOW_IN_MED: f32 = 2.0;

/// Default penalty for hand imbalance.
pub const DEFAULT_PENALTY_IMBALANCE: f32 = 200.0;
/// Default maximum allowed hand imbalance ratio.
pub const DEFAULT_MAX_HAND_IMBALANCE: f32 = 0.55;
/// Default weight multiplier for vertical travel distance.
pub const DEFAULT_WEIGHT_VERTICAL_TRAVEL: f32 = 1.0;
/// Default weight multiplier for lateral travel distance.
pub const DEFAULT_WEIGHT_LATERAL_TRAVEL: f32 = 3.5;
/// Default weight multiplier for finger effort.
pub const DEFAULT_WEIGHT_FINGER_EFFORT: f32 = 2.2;
/// Default penalty for keys missing from the cost model.
pub const DEFAULT_PENALTY_MISSING_KEY: f32 = 100.0;

/// Default cost in milliseconds (if using time-based scoring).
pub const DEFAULT_COST_MS: f32 = 120.0;
/// Default limit on the number of trigrams to load.
pub const DEFAULT_LOADER_TRIGRAM_LIMIT: usize = 3000;
/// Default required trigram coverage.
pub const DEFAULT_TRIGRAM_COVERAGE: f32 = 0.99;

/// Default scale factors for finger penalties as an array.
pub const DEFAULT_FINGER_PENALTY_SCALE_ARRAY: [f32; 5] = [0.0, 1.0, 1.1, 1.3, 1.6];
/// Default comfortable scissor pairs (Indices).
pub const DEFAULT_COMFORTABLE_SCISSORS: &str = "21,23,34";

/// Weights and penalties defining the "personality" of the scoring engine.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(default)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
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

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;

    #[test]
    fn test_scoring_weights_validation() {
        let mut w = ScoringWeights::default();
        assert!(w.validate().is_ok());

        // 1. Negative penalty
        w.weights.insert("penalty_sfb_base".into(), -1.0);
        assert!(w.validate().is_err());

        // 2. Weight overflow
        w.weights
            .insert("penalty_sfb_base".into(), MAX_SAFE_WEIGHT * 2.0);
        assert!(w.validate().is_err());

        // 3. Negative finger scale
        w = ScoringWeights::default();
        w.finger_penalty_scale[0] = -0.5;
        assert!(w.validate().is_err());

        // 4. Trigram limit safety
        w = ScoringWeights::default();
        w.weights.insert(
            "loader_trigram_limit".into(),
            (MAX_LOADER_TRIGRAM_LIMIT + 1) as f32,
        );
        assert!(w.validate().is_err());
    }

    #[test]
    fn test_scoring_weights_getters_all() {
        let w = ScoringWeights::default();
        // Just call them all to ensure coverage and they return defaults
        assert_eq!(
            w.get_penalty_sfr_weak_finger(),
            DEFAULT_PENALTY_SFR_WEAK_FINGER.into()
        );
        assert_eq!(w.get_penalty_sfr_bad_row(), DEFAULT_PENALTY_SFR_BAD_ROW.into());
        assert_eq!(w.get_penalty_sfr_lat(), DEFAULT_PENALTY_SFR_LAT.into());
        assert_eq!(w.get_penalty_sfb_lateral(), DEFAULT_PENALTY_SFB_LATERAL.into());
        assert_eq!(
            w.get_penalty_sfb_lateral_weak(),
            DEFAULT_PENALTY_SFB_LATERAL_WEAK.into()
        );
        assert_eq!(w.get_penalty_sfb_base(), DEFAULT_PENALTY_SFB_BASE.into());
        assert_eq!(
            w.get_penalty_sfb_outward_adder(),
            DEFAULT_PENALTY_SFB_OUTWARD_ADDER.into()
        );
        assert_eq!(w.get_penalty_sfb_diagonal(), DEFAULT_PENALTY_SFB_DIAGONAL.into());
        assert_eq!(w.get_penalty_sfb_long(), DEFAULT_PENALTY_SFB_LONG.into());
        assert_eq!(w.get_penalty_sfb_bottom(), DEFAULT_PENALTY_SFB_BOTTOM.into());
        assert_eq!(
            w.get_weight_weak_finger_sfb(),
            DEFAULT_WEIGHT_WEAK_FINGER_SFB.into()
        );
        assert_eq!(
            w.get_threshold_sfb_long_row_diff(),
            DEFAULT_THRESHOLD_SFB_LONG_ROW_DIFF
        );
        assert_eq!(
            w.get_threshold_scissor_row_diff(),
            DEFAULT_THRESHOLD_SCISSOR_ROW_DIFF
        );
        assert_eq!(
            w.get_threshold_reach_stretch(),
            DEFAULT_THRESHOLD_REACH_STRETCH.into()
        );
        assert_eq!(w.get_penalty_scissor(), DEFAULT_PENALTY_SCISSOR.into());
        assert_eq!(w.get_penalty_ring_pinky(), DEFAULT_PENALTY_RING_PINKY.into());
        assert_eq!(w.get_penalty_lateral(), DEFAULT_PENALTY_LATERAL.into());
        assert_eq!(
            w.get_penalty_monogram_stretch(),
            DEFAULT_PENALTY_MONOGRAM_STRETCH.into()
        );
        assert_eq!(w.get_penalty_skip(), DEFAULT_PENALTY_SKIP.into());
        assert_eq!(w.get_penalty_redirect(), DEFAULT_PENALTY_REDIRECT.into());
        assert_eq!(w.get_penalty_hand_run(), DEFAULT_PENALTY_HAND_RUN.into());
        assert_eq!(w.get_bonus_inward_roll(), DEFAULT_BONUS_INWARD_ROLL.into());
        assert_eq!(w.get_bonus_bigram_roll_in(), DEFAULT_BONUS_BIGRAM_ROLL_IN.into());
        assert_eq!(w.get_bonus_bigram_roll_out(), DEFAULT_BONUS_BIGRAM_ROLL_OUT.into());
        assert_eq!(w.get_penalty_high_in_med(), DEFAULT_PENALTY_HIGH_IN_MED.into());
        assert_eq!(w.get_penalty_high_in_low(), DEFAULT_PENALTY_HIGH_IN_LOW.into());
        assert_eq!(w.get_penalty_med_in_prime(), DEFAULT_PENALTY_MED_IN_PRIME.into());
        assert_eq!(w.get_penalty_med_in_low(), DEFAULT_PENALTY_MED_IN_LOW.into());
        assert_eq!(w.get_penalty_low_in_prime(), DEFAULT_PENALTY_LOW_IN_PRIME.into());
        assert_eq!(w.get_penalty_low_in_med(), DEFAULT_PENALTY_LOW_IN_MED.into());
        assert_eq!(w.get_penalty_imbalance(), DEFAULT_PENALTY_IMBALANCE.into());
        assert_eq!(w.get_max_hand_imbalance(), DEFAULT_MAX_HAND_IMBALANCE.into());
        assert_eq!(
            w.get_weight_vertical_travel(),
            DEFAULT_WEIGHT_VERTICAL_TRAVEL.into()
        );
        assert_eq!(w.get_weight_lateral_travel(), DEFAULT_WEIGHT_LATERAL_TRAVEL.into());
        assert_eq!(w.get_weight_finger_effort(), DEFAULT_WEIGHT_FINGER_EFFORT.into());
        assert_eq!(w.get_default_cost_ms(), DEFAULT_COST_MS.into());
        assert_eq!(w.get_loader_trigram_limit(), DEFAULT_LOADER_TRIGRAM_LIMIT);
        assert_eq!(w.get_trigram_coverage(), DEFAULT_TRIGRAM_COVERAGE.into());
        assert_eq!(
            w.get_finger_penalty_scale(),
            DEFAULT_FINGER_PENALTY_SCALE_ARRAY
        );
        assert_eq!(
            w.allowed_hand_balance_deviation(),
            DEFAULT_MAX_HAND_IMBALANCE - 0.5
        );

        let scissors = w.get_comfortable_scissors();
        assert!(!scissors.is_empty());
    }

    #[test]
    fn test_scoring_weights_schema() {
        let s = ScoringWeights::schema();
        assert!(!s.is_empty());
    }

    #[test]
    fn test_scoring_weights_json_serde() {
        let w = ScoringWeights::default();
        let json_str = serde_json::to_string(&w).unwrap();
        let w2: ScoringWeights = serde_json::from_str(&json_str).unwrap();

        assert_eq!(w.get_penalty_sfb_base(), w2.get_penalty_sfb_base());
        assert_eq!(w.comfortable_scissors, w2.comfortable_scissors);
    }

    #[cfg(feature = "cli")]
    #[test]
    fn test_scoring_weights_config_conversion() {
        let config = ScoringWeightsConfig {
            overrides: vec![("penalty_sfb_base".to_string(), 600.0)],
            finger_penalty_scale: vec![0.0, 1.0, 1.0, 1.0, 1.0],
            comfortable_scissors: "12,23".to_string(),
        };
        let w = ScoringWeights::try_from(config).unwrap();
        assert_eq!(w.get_penalty_sfb_base(), Weight(600.0));
        assert_eq!(w.finger_penalty_scale, [0.0, 1.0, 1.0, 1.0, 1.0]);
        assert_eq!(w.comfortable_scissors, "12,23");
    }
}

#[keyforge_testing_macros::kf_test]
mod fuzz {
    use super::*;
    use proptest::prelude::*;

    fn weights_strategy() -> impl Strategy<Value = ScoringWeights> {
        (any::<f32>(), any::<f32>(), any::<f32>(), any::<usize>()).prop_map(
            |(sfb, scis, redir, limit)| {
                let mut w = ScoringWeights::default();
                w.weights.insert("penalty_sfb_base".to_string(), sfb);
                w.weights.insert("penalty_scissor".to_string(), scis);
                w.weights.insert("penalty_redirect".to_string(), redir);
                w.weights
                    .insert("loader_trigram_limit".to_string(), limit as f32);
                w
            },
        )
    }

    proptest! {
        #[test]
        fn fuzz_weights_validation(w in weights_strategy()) {
            let _ = w.validate();
        }
    }
}