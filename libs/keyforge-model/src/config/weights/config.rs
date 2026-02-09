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
use crate::types::Score;
use crate::validator::Validator;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Weights and penalties defining the "personality" of the scoring engine.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(default)]
pub struct ScoringWeights {
    /// Dynamic weights map.
    pub weights: std::collections::HashMap<String, Score>,

    /// Finger penalty multipliers (Thumb, Index, Middle, Ring, Pinky).
    pub finger_penalty_scale: [Score; 5],
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
    fn insert_sfb_penalties(weights: &mut std::collections::HashMap<String, Score>) {
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
            Score::from_scaled_i64(i64::from(DEFAULT_THRESHOLD_SFB_LONG_ROW_DIFF) * 1_000_000),
        );
    }

    fn insert_movement_penalties(weights: &mut std::collections::HashMap<String, Score>) {
        weights.insert(
            "threshold_scissor_row_diff".to_string(),
            Score::from_scaled_i64(i64::from(DEFAULT_THRESHOLD_SCISSOR_ROW_DIFF) * 1_000_000),
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

    fn insert_dynamic_rules(weights: &mut std::collections::HashMap<String, Score>) {
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

    fn insert_loader_defaults(weights: &mut std::collections::HashMap<String, Score>) {
        weights.insert("default_cost_ms".to_string(), DEFAULT_COST_MS);
        weights.insert(
            "loader_trigram_limit".to_string(),
            Score::from_scaled_i64(
                i64::from(u32::try_from(DEFAULT_LOADER_TRIGRAM_LIMIT).unwrap_or(0)) * 1_000_000,
            ),
        );
        weights.insert("trigram_coverage".to_string(), DEFAULT_TRIGRAM_COVERAGE);
    }
}

impl Validator for ScoringWeights {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn validate(&self) -> Result<(), String> {
        if let Some(&limit) = self.weights.get("loader_trigram_limit") {
            if usize::try_from(limit.raw() / 1_000_000).unwrap_or(0) > MAX_LOADER_TRIGRAM_LIMIT {
                return Err(format!(
                    "loader_trigram_limit exceeds safety maximum ({MAX_LOADER_TRIGRAM_LIMIT})"
                ));
            }
        }

        for (key, &val) in &self.weights {
            if val < Score::ZERO && !key.contains("bonus") {
                return Err(format!("Penalty weight '{key}' cannot be negative"));
            }
            if val > MAX_SAFE_WEIGHT {
                return Err(format!(
                    "Weight '{key}' exceeds safety maximum ({MAX_SAFE_WEIGHT})"
                ));
            }
        }

        for (i, &w) in self.finger_penalty_scale.iter().enumerate() {
            if w < Score::ZERO {
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
                max: Some(MAX_SAFE_WEIGHT.to_f32()),
                default: DEFAULT_PENALTY_SFB_BASE.to_f32(),
            },
            ParameterMetadata {
                key: "penalty_scissor".to_string(),
                label: "Scissor Penalty".to_string(),
                description: "Penalty for adjacent finger stretches.".to_string(),
                param_type: ParamType::Float,
                min: Some(0.0),
                max: Some(MAX_SAFE_WEIGHT.to_f32()),
                default: DEFAULT_PENALTY_SCISSOR.to_f32(),
            },
            ParameterMetadata {
                key: "weight_vertical_travel".to_string(),
                label: "Vertical Travel Weight".to_string(),
                description: "Multiplier for finger movement distance.".to_string(),
                param_type: ParamType::Float,
                min: Some(0.0),
                max: Some(10.0),
                default: DEFAULT_WEIGHT_VERTICAL_TRAVEL.to_f32(),
            },
        ]
    }
}
