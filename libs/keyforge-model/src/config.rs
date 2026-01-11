// libs/keyforge-model/src/config.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.


use crate::constants::{
    MAX_LOADER_TRIGRAM_LIMIT, MAX_OPT_LIMIT_FAST, MAX_SAFE_WEIGHT, MAX_SEARCH_EPOCHS,
    MAX_SEARCH_STEPS, MAX_TEMP, ASSET_COST_MATRIX,
    DEFAULT_SEARCH_EPOCHS, DEFAULT_SEARCH_STEPS, DEFAULT_SEARCH_PATIENCE,
    DEFAULT_SEARCH_PATIENCE_THRESHOLD, DEFAULT_TEMP_MIN, DEFAULT_TEMP_MAX,
    DEFAULT_OPT_LIMIT_FAST, DEFAULT_OPT_LIMIT_SLOW, DEFAULT_REHEATS, DEFAULT_REHEAT_FACTOR,
    DEFAULT_PENALTY_SFR_WEAK_FINGER, DEFAULT_PENALTY_SFR_BAD_ROW, DEFAULT_PENALTY_SFR_LAT,
    DEFAULT_PENALTY_SFB_LATERAL, DEFAULT_PENALTY_SFB_LATERAL_WEAK, DEFAULT_PENALTY_SFB_BASE,
    DEFAULT_PENALTY_SFB_OUTWARD_ADDER, DEFAULT_PENALTY_SFB_DIAGONAL, DEFAULT_PENALTY_SFB_LONG,
    DEFAULT_PENALTY_SFB_BOTTOM, DEFAULT_WEIGHT_WEAK_FINGER_SFB,
    DEFAULT_THRESHOLD_SFB_LONG_ROW_DIFF, DEFAULT_THRESHOLD_SCISSOR_ROW_DIFF, DEFAULT_THRESHOLD_REACH_STRETCH,
    DEFAULT_PENALTY_SCISSOR, DEFAULT_PENALTY_RING_PINKY, DEFAULT_PENALTY_LATERAL,
    DEFAULT_PENALTY_MONOGRAM_STRETCH, DEFAULT_PENALTY_SKIP, DEFAULT_PENALTY_REDIRECT,
    DEFAULT_PENALTY_HAND_RUN, DEFAULT_BONUS_INWARD_ROLL, DEFAULT_BONUS_BIGRAM_ROLL_IN,
    DEFAULT_BONUS_BIGRAM_ROLL_OUT, DEFAULT_PENALTY_HIGH_IN_MED, DEFAULT_PENALTY_HIGH_IN_LOW,
    DEFAULT_PENALTY_MED_IN_PRIME, DEFAULT_PENALTY_MED_IN_LOW, DEFAULT_PENALTY_LOW_IN_PRIME,
    DEFAULT_PENALTY_LOW_IN_MED, DEFAULT_PENALTY_IMBALANCE, DEFAULT_MAX_HAND_IMBALANCE,
    DEFAULT_WEIGHT_VERTICAL_TRAVEL, DEFAULT_WEIGHT_LATERAL_TRAVEL, DEFAULT_WEIGHT_FINGER_EFFORT,
    DEFAULT_COST_MS, DEFAULT_LOADER_TRIGRAM_LIMIT, DEFAULT_TRIGRAM_COVERAGE,
    DEFAULT_TIER_HIGH, DEFAULT_TIER_MED, DEFAULT_TIER_LOW, DEFAULT_CRITICAL_BIGRAMS,
    DEFAULT_FINGER_REPEAT_SCALE_ARRAY, DEFAULT_FINGER_PENALTY_SCALE_ARRAY, DEFAULT_COMFORTABLE_SCISSORS,
};
use crate::types::KeyIndex;
use crate::validator::Validator;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use utoipa::ToSchema;
#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

/// The root configuration aggregate for a KeyForge session.
#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct Config {
    /// Search parameters for the optimization engine.
    pub search: SearchParams,
    /// Weights for the physics scoring engine.
    pub weights: ScoringWeights,
    /// Definitions for layout tiers and critical bigrams.
    pub defs: LayoutDefinitions,
}

impl Validator for Config {
    fn validate(&self) -> Result<(), String> {
        self.search.validate()?;
        self.weights.validate()?;
        self.defs.validate()?;
        Ok(())
    }
}

/// Defines a source for text corpus data.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct CorpusSource {
    /// The identifier or path of the corpus.
    pub id: String,
    /// The weight multiplier for this corpus.
    pub weight: f32,
    /// Optional hash for integrity verification.
    #[serde(default, skip_serializing_if = "crate::utils::is_none")]
    #[cfg_attr(feature = "ts_bindings", ts(optional))]
    pub hash: Option<String>,
}

impl Validator for CorpusSource {
    fn validate(&self) -> Result<(), String> {
        if self.id.trim().is_empty() {
            return Err("Corpus ID cannot be empty".to_string());
        }
        if self.weight <= 0.0 || !self.weight.is_finite() {
            return Err(format!("Invalid weight for corpus '{}': {}", self.id, self.weight));
        }
        Ok(())
    }
}

impl Hash for CorpusSource {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.weight.to_bits().hash(state);
        self.hash.hash(state);
    }
}

impl Default for CorpusSource {
    fn default() -> Self {
        Self {
            id: "text/en_std".to_string(),
            weight: 1.0,
            hash: None,
        }
    }
}

impl FromStr for CorpusSource {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((id, weight_str)) = s.split_once(':') {
            let weight = weight_str
                .parse::<f32>()
                .map_err(|_| format!("invalid weight '{}' for corpus '{}'", weight_str, id))?;

            if weight.is_nan() || weight <= f32::EPSILON {
                return Err(format!(
                    "weight for corpus '{}' must be positive (got {})",
                    id, weight
                ));
            }

            Ok(CorpusSource {
                id: id.trim().to_string(),
                weight,
                hash: None,
            })
        } else {
            Ok(CorpusSource {
                id: s.trim().to_string(),
                weight: 1.0,
                hash: None,
            })
        }
    }
}

/// Parameters controlling the Simulated Annealing algorithm.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct SearchParams {
    /// Number of independent search epochs to run.
    pub search_epochs: usize,
    /// Maximum number of mutation steps per epoch.
    pub search_steps: usize,
    /// Steps without improvement before triggering a reheat.
    pub search_patience: usize,
    /// Threshold for patience reset (improvement must be > this).
    pub search_patience_threshold: f32,
    /// Minimum temperature (stop condition).
    pub temp_min: f32,
    /// Maximum temperature (start condition).
    pub temp_max: f32,
    /// Optimization limit for fast path.
    pub opt_limit_fast: usize,
    /// Optimization limit for slow path.
    pub opt_limit_slow: usize,
    /// Number of times to reheat the system if stuck.
    #[serde(default = "default_reheats")]
    pub reheats: usize,
    /// Factor to multiply temperature by when reheating.
    #[serde(default = "default_reheat_factor")]
    pub reheat_factor: f32,
    /// Random seed for deterministic replay (Optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
}

fn default_reheats() -> usize { DEFAULT_REHEATS }
fn default_reheat_factor() -> f32 { DEFAULT_REHEAT_FACTOR }

impl Default for SearchParams {
    fn default() -> Self {
        Self {
            search_epochs: DEFAULT_SEARCH_EPOCHS,
            search_steps: DEFAULT_SEARCH_STEPS,
            search_patience: DEFAULT_SEARCH_PATIENCE,
            search_patience_threshold: DEFAULT_SEARCH_PATIENCE_THRESHOLD,
            temp_min: DEFAULT_TEMP_MIN,
            temp_max: DEFAULT_TEMP_MAX,
            opt_limit_fast: DEFAULT_OPT_LIMIT_FAST,
            opt_limit_slow: DEFAULT_OPT_LIMIT_SLOW,
            reheats: default_reheats(),
            reheat_factor: default_reheat_factor(),
            seed: None,
        }
    }
}

impl Validator for SearchParams {
    fn validate(&self) -> Result<(), String> {
        if self.search_epochs == 0 { return Err("search_epochs must be > 0".into()); }
        if self.search_epochs > MAX_SEARCH_EPOCHS {
            return Err(format!("search_epochs exceeds limit ({})", MAX_SEARCH_EPOCHS));
        }
        if self.search_steps == 0 { return Err("search_steps must be > 0".into()); }
        if self.search_steps > MAX_SEARCH_STEPS {
            return Err(format!("search_steps exceeds limit ({})", MAX_SEARCH_STEPS));
        }
        if self.opt_limit_fast == 0 { return Err("opt_limit_fast must be > 0".into()); }
        if self.opt_limit_fast > MAX_OPT_LIMIT_FAST {
            return Err(format!("opt_limit_fast exceeds limit ({})", MAX_OPT_LIMIT_FAST));
        }
        if self.opt_limit_slow < self.opt_limit_fast {
            return Err("opt_limit_slow must be >= opt_limit_fast".into());
        }
        if self.temp_min < 0.0 || self.temp_max < 0.0 {
            return Err("Temperature cannot be negative".into());
        }
        if self.temp_max > MAX_TEMP {
            return Err(format!("temp_max exceeds limit ({})", MAX_TEMP));
        }
        if self.temp_min < 0.0001 {
            return Err("temp_min too low (underflow risk)".into());
        }
        if self.temp_min >= self.temp_max {
            return Err("temp_min must be < temp_max".into());
        }
        if self.search_patience_threshold < 0.0 || self.search_patience_threshold > 1.0 {
            return Err("search_patience_threshold must be between 0.0 and 1.0".into());
        }
        Ok(())
    }
}

/// Weights and penalties defining the "personality" of the scoring engine.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(default)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct ScoringWeights {
    /// Penalty for Same Finger Repeat on a weak finger.
    pub penalty_sfr_weak_finger: f32,
    /// Penalty for Same Finger Repeat involving a bad row jump.
    pub penalty_sfr_bad_row: f32,
    /// Penalty for lateral Same Finger Repeat.
    pub penalty_sfr_lat: f32,
    /// Penalty for lateral Same Finger Bigram.
    pub penalty_sfb_lateral: f32,
    /// Penalty for lateral SFB on a weak finger.
    pub penalty_sfb_lateral_weak: f32,
    /// Base penalty for any Same Finger Bigram.
    pub penalty_sfb_base: f32,
    /// Additional penalty for outward rolling SFBs.
    pub penalty_sfb_outward_adder: f32,
    /// Penalty for diagonal SFBs.
    pub penalty_sfb_diagonal: f32,
    /// Penalty for long-distance SFBs.
    pub penalty_sfb_long: f32,
    /// Penalty for bottom-row SFBs.
    pub penalty_sfb_bottom: f32,
    /// Multiplier for SFBs on weak fingers.
    pub weight_weak_finger_sfb: f32,
    /// Row difference threshold for "long" SFBs.
    pub threshold_sfb_long_row_diff: i8,
    /// Row difference threshold for scissors.
    pub threshold_scissor_row_diff: i8,
    /// Distance threshold for reach stretches.
    pub threshold_reach_stretch: f32,
    /// Penalty for scissor (adjacent finger stretch) movements.
    pub penalty_scissor: f32,
    /// Penalty for ring-pinky interactions.
    pub penalty_ring_pinky: f32,
    /// Penalty for lateral movement.
    pub penalty_lateral: f32,
    /// Penalty for single-key stretches.
    pub penalty_monogram_stretch: f32,
    /// Penalty for skipping a key (hurdle).
    pub penalty_skip: f32,
    /// Penalty for redirecting flow (e.g., Left -> Right -> Left).
    pub penalty_redirect: f32,
    /// Penalty for excessive hand alternation runs.
    pub penalty_hand_run: f32,
    /// Bonus (negative cost) for inward rolls.
    pub bonus_inward_roll: f32,
    /// Bonus for specific bigram inward rolls.
    pub bonus_bigram_roll_in: f32,
    /// Bonus for specific bigram outward rolls.
    pub bonus_bigram_roll_out: f32,
    /// Penalty for high-frequency keys in medium slots.
    pub penalty_high_in_med: f32,
    /// Penalty for high-frequency keys in low slots.
    pub penalty_high_in_low: f32,
    /// Penalty for medium-frequency keys in prime slots.
    pub penalty_med_in_prime: f32,
    /// Penalty for medium-frequency keys in low slots.
    pub penalty_med_in_low: f32,
    /// Penalty for low-frequency keys in prime slots.
    pub penalty_low_in_prime: f32,
    /// Penalty for low-frequency keys in medium slots.
    pub penalty_low_in_med: f32,
    /// Penalty for hand imbalance.
    pub penalty_imbalance: f32,
    /// Maximum allowed hand imbalance ratio.
    pub max_hand_imbalance: f32,
    /// Weight multiplier for vertical travel distance.
    pub weight_vertical_travel: f32,
    /// Weight multiplier for lateral travel distance.
    pub weight_lateral_travel: f32,
    /// Weight multiplier for finger effort.
    pub weight_finger_effort: f32,
    /// Default cost in milliseconds (if using time-based scoring).
    pub default_cost_ms: f32,
    /// Limit on the number of trigrams to load.
    pub loader_trigram_limit: usize,
    /// Required trigram coverage (0.0 - 1.0).
    pub trigram_coverage: f32,
    /// Finger penalty multipliers (Thumb, Index, Middle, Ring, Pinky).
    pub finger_penalty_scale: [f32; 5],
    /// Comma-separated list of comfortable scissor pairs.
    pub comfortable_scissors: String,
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            penalty_sfr_weak_finger: DEFAULT_PENALTY_SFR_WEAK_FINGER,
            penalty_sfr_bad_row: DEFAULT_PENALTY_SFR_BAD_ROW,
            penalty_sfr_lat: DEFAULT_PENALTY_SFR_LAT,
            penalty_sfb_lateral: DEFAULT_PENALTY_SFB_LATERAL,
            penalty_sfb_lateral_weak: DEFAULT_PENALTY_SFB_LATERAL_WEAK,
            penalty_sfb_base: DEFAULT_PENALTY_SFB_BASE,
            penalty_sfb_outward_adder: DEFAULT_PENALTY_SFB_OUTWARD_ADDER,
            penalty_sfb_diagonal: DEFAULT_PENALTY_SFB_DIAGONAL,
            penalty_sfb_long: DEFAULT_PENALTY_SFB_LONG,
            penalty_sfb_bottom: DEFAULT_PENALTY_SFB_BOTTOM,
            weight_weak_finger_sfb: DEFAULT_WEIGHT_WEAK_FINGER_SFB,
            threshold_sfb_long_row_diff: DEFAULT_THRESHOLD_SFB_LONG_ROW_DIFF,
            threshold_scissor_row_diff: DEFAULT_THRESHOLD_SCISSOR_ROW_DIFF,
            threshold_reach_stretch: DEFAULT_THRESHOLD_REACH_STRETCH,
            penalty_scissor: DEFAULT_PENALTY_SCISSOR,
            penalty_ring_pinky: DEFAULT_PENALTY_RING_PINKY,
            penalty_lateral: DEFAULT_PENALTY_LATERAL,
            penalty_monogram_stretch: DEFAULT_PENALTY_MONOGRAM_STRETCH,
            penalty_skip: DEFAULT_PENALTY_SKIP,
            penalty_redirect: DEFAULT_PENALTY_REDIRECT,
            penalty_hand_run: DEFAULT_PENALTY_HAND_RUN,
            bonus_inward_roll: DEFAULT_BONUS_INWARD_ROLL,
            bonus_bigram_roll_in: DEFAULT_BONUS_BIGRAM_ROLL_IN,
            bonus_bigram_roll_out: DEFAULT_BONUS_BIGRAM_ROLL_OUT,
            penalty_high_in_med: DEFAULT_PENALTY_HIGH_IN_MED,
            penalty_high_in_low: DEFAULT_PENALTY_HIGH_IN_LOW,
            penalty_med_in_prime: DEFAULT_PENALTY_MED_IN_PRIME,
            penalty_med_in_low: DEFAULT_PENALTY_MED_IN_LOW,
            penalty_low_in_prime: DEFAULT_PENALTY_LOW_IN_PRIME,
            penalty_low_in_med: DEFAULT_PENALTY_LOW_IN_MED,
            penalty_imbalance: DEFAULT_PENALTY_IMBALANCE,
            max_hand_imbalance: DEFAULT_MAX_HAND_IMBALANCE,
            weight_vertical_travel: DEFAULT_WEIGHT_VERTICAL_TRAVEL,
            weight_lateral_travel: DEFAULT_WEIGHT_LATERAL_TRAVEL,
            weight_finger_effort: DEFAULT_WEIGHT_FINGER_EFFORT,
            default_cost_ms: DEFAULT_COST_MS,
            loader_trigram_limit: DEFAULT_LOADER_TRIGRAM_LIMIT,
            trigram_coverage: DEFAULT_TRIGRAM_COVERAGE,
            finger_penalty_scale: DEFAULT_FINGER_PENALTY_SCALE_ARRAY,
            comfortable_scissors: DEFAULT_COMFORTABLE_SCISSORS.to_string(),
        }
    }
}

impl Validator for ScoringWeights {
    fn validate(&self) -> Result<(), String> {
        if self.loader_trigram_limit > MAX_LOADER_TRIGRAM_LIMIT {
            return Err(format!("loader_trigram_limit exceeds safety maximum ({})", MAX_LOADER_TRIGRAM_LIMIT));
        }
        if self.penalty_sfb_base < 0.0 || self.penalty_scissor < 0.0 {
            return Err("Penalties cannot be negative".to_string());
        }
        if self.penalty_sfb_base > MAX_SAFE_WEIGHT || self.penalty_scissor > MAX_SAFE_WEIGHT {
            return Err(format!("Weights cannot exceed {:.0}", MAX_SAFE_WEIGHT));
        }
        for (i, &w) in self.finger_penalty_scale.iter().enumerate() {
            if w < 0.0 { return Err(format!("finger_penalty_scale[{}] cannot be negative", i)); }
        }
        Ok(())
    }
}

impl ScoringWeights {
    /// Returns the finger penalty scale array.
    pub fn get_finger_penalty_scale(&self) -> [f32; 5] {
        self.finger_penalty_scale
    }
    /// Calculates the allowed deviation from perfect hand balance.
    pub fn allowed_hand_balance_deviation(&self) -> f32 {
        (self.max_hand_imbalance - 0.5).max(0.0)
    }
    /// Parses the comfortable scissors string into a list of pairs.
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

/// Definitions for character tiers and critical bigrams.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(default)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct LayoutDefinitions {
    /// Characters considered high priority (Home row candidates).
    pub tier_high_chars: String,
    /// Characters considered medium priority.
    pub tier_med_chars: String,
    /// Characters considered low priority.
    pub tier_low_chars: String,
    /// Bigrams that must be optimized for.
    pub critical_bigrams: String,
    /// Scale factors for finger repeat penalties.
    pub finger_repeat_scale: [f32; 5],
}

impl Default for LayoutDefinitions {
    fn default() -> Self {
        Self {
            tier_high_chars: DEFAULT_TIER_HIGH.to_string(),
            tier_med_chars: DEFAULT_TIER_MED.to_string(),
            tier_low_chars: DEFAULT_TIER_LOW.to_string(),
            critical_bigrams: DEFAULT_CRITICAL_BIGRAMS.to_string(),
            finger_repeat_scale: DEFAULT_FINGER_REPEAT_SCALE_ARRAY,
        }
    }
}

impl Validator for LayoutDefinitions {
    fn validate(&self) -> Result<(), String> {
        if self.tier_high_chars.is_empty() { return Err("tier_high_chars cannot be empty".to_string()); }
        
        for (i, &v) in self.finger_repeat_scale.iter().enumerate() {
            if v < 0.0 { return Err(format!("finger_repeat_scale[{}] cannot be negative", i)); }
        }
        
        Ok(())
    }
}

impl LayoutDefinitions {
    /// Parses the critical bigrams string into a list of byte pairs.
    pub fn get_critical_bigrams(&self) -> Vec<[u8; 2]> {
        self.critical_bigrams.split(',').filter_map(|s| {
            let b = s.trim().as_bytes();
            if b.len() == 2 { Some([b[0], b[1]]) } else { None }
        }).collect()
    }
}

/// Source for the cost matrix data.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, ToSchema)]
#[serde(tag = "type", content = "data")]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub enum CostMatrixSource {
    /// A predefined cost matrix file (e.g. "default_costmatrix.json").
    Predefined(String),
}

impl Default for CostMatrixSource {
    fn default() -> Self { CostMatrixSource::Predefined(ASSET_COST_MATRIX.to_string()) }
}

impl fmt::Display for CostMatrixSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CostMatrixSource::Predefined(s) => write!(f, "{}", s),
        }
    }
}

/// Constraint forcing a key to a specific physical index.
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct KeyConstraint {
    /// The physical index of the key.
    pub index: KeyIndex,
    /// The logical key label/ID to pin.
    pub key: String,
}

impl Validator for KeyConstraint {
    fn validate(&self) -> Result<(), String> {
        if self.key.trim().is_empty() {
            return Err(format!("Constraint for index {} has empty key", self.index));
        }
        Ok(())
    }
}

impl FromStr for KeyConstraint {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.trim().is_empty() { return Err("Empty constraint".to_string()); }
        let (idx_str, key_str) = s.split_once(':').ok_or_else(|| format!("invalid format '{}': expected INDEX:KEY", s))?;
        let index_val = idx_str.trim().parse::<u16>().map_err(|_| format!("invalid index '{}': must be 0-65535", idx_str))?;
        let key_clean = key_str.trim();
        if key_clean.is_empty() { return Err(format!("Empty key in constraint '{}'", s)); }
        Ok(KeyConstraint { index: KeyIndex(index_val), key: key_clean.to_string() })
    }
}
