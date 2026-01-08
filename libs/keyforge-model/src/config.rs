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
    MAX_SEARCH_STEPS, MAX_TEMP, EFFORT_THUMB, EFFORT_INDEX, EFFORT_MIDDLE, EFFORT_RING, EFFORT_PINKY, ASSET_COST_MATRIX,
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
    /// Parameters controlling the annealing search process.
    pub search: SearchParams,
    /// Weights and penalties for the scoring engine.
    pub weights: ScoringWeights,
    /// Definitions for layout generation (character tiers, etc.).
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
    /// The identifier or filename of the corpus (e.g., "text/en_std").
    pub id: String,
    /// The weight multiplier for this corpus (default: 1.0).
    pub weight: f32,
    /// Optional hash for integrity verification.
    #[serde(default, skip_serializing_if = "crate::serde_utils::is_none")]
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
    /// Number of epochs (independent runs) to perform.
    pub search_epochs: usize,
    /// Maximum number of mutation steps per epoch.
    pub search_steps: usize,
    /// Number of steps without improvement before triggering a reheat.
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
    /// Number of times to reheat the system if stuck in a local minimum.
    #[serde(default = "default_reheats")]
    pub reheats: usize,
    /// Factor to multiply temperature by when reheating.
    #[serde(default = "default_reheat_factor")]
    pub reheat_factor: f32,
}

fn default_reheats() -> usize { 3 }
fn default_reheat_factor() -> f32 { 0.5 }

impl Default for SearchParams {
    fn default() -> Self {
        Self {
            search_epochs: 10_000,
            search_steps: 100_000,
            search_patience: 500,
            search_patience_threshold: 0.1,
            temp_min: 0.005,
            temp_max: 20.0,
            opt_limit_fast: 100,
            opt_limit_slow: 1500,
            reheats: default_reheats(),
            reheat_factor: default_reheat_factor(),
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
    /// Comma-separated string of finger penalty multipliers.
    pub finger_penalty_scale: String,
    /// Comma-separated string of comfortable scissor pairs.
    pub comfortable_scissors: String,
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            penalty_sfr_weak_finger: 0.0,
            penalty_sfr_bad_row: 0.0,
            penalty_sfr_lat: 0.0,
            penalty_sfb_lateral: 0.0,
            penalty_sfb_lateral_weak: 0.0,
            penalty_sfb_base: 0.0,
            penalty_sfb_outward_adder: 0.0,
            penalty_sfb_diagonal: 0.0,
            penalty_sfb_long: 0.0,
            penalty_sfb_bottom: 0.0,
            weight_weak_finger_sfb: 0.0,
            threshold_sfb_long_row_diff: 0,
            threshold_scissor_row_diff: 0,
            threshold_reach_stretch: 0.0,
            penalty_scissor: 0.0,
            penalty_ring_pinky: 0.0,
            penalty_lateral: 0.0,
            penalty_monogram_stretch: 0.0,
            penalty_skip: 0.0,
            penalty_redirect: 0.0,
            penalty_hand_run: 0.0,
            bonus_inward_roll: 0.0,
            bonus_bigram_roll_in: 0.0,
            bonus_bigram_roll_out: 0.0,
            penalty_high_in_med: 0.0,
            penalty_high_in_low: 0.0,
            penalty_med_in_prime: 0.0,
            penalty_med_in_low: 0.0,
            penalty_low_in_prime: 0.0,
            penalty_low_in_med: 0.0,
            penalty_imbalance: 0.0,
            max_hand_imbalance: 0.0,
            weight_vertical_travel: 0.0,
            weight_lateral_travel: 0.0,
            weight_finger_effort: 0.0,
            default_cost_ms: 0.0,
            loader_trigram_limit: 0,
            trigram_coverage: 0.0,
            finger_penalty_scale: format!("{}, {}, {}, {}, {}", EFFORT_THUMB, EFFORT_INDEX, EFFORT_MIDDLE, EFFORT_RING, EFFORT_PINKY),
            comfortable_scissors: "".to_string(),
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
        if !self.finger_penalty_scale.is_empty() {
            if let Err(e) = parse_f32_array::<5>(&self.finger_penalty_scale) {
                return Err(format!("Invalid finger_penalty_scale: {}", e));
            }
        }
        Ok(())
    }
}

impl ScoringWeights {
    /// Parses the finger penalty scale string into an array.
    pub fn get_finger_penalty_scale(&self) -> [f32; 5] {
        if self.finger_penalty_scale.is_empty() { return [0.0; 5]; }
        parse_f32_array::<5>(&self.finger_penalty_scale).unwrap_or([0.0; 5])
    }
    /// Calculates the allowed deviation from perfect hand balance (0.5).
    pub fn allowed_hand_balance_deviation(&self) -> f32 {
        (self.max_hand_imbalance - 0.5).max(0.0)
    }
    /// Parses the comfortable scissors string into a vector of finger pairs.
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
    /// Characters considered high priority.
    pub tier_high_chars: String,
    /// Characters considered medium priority.
    pub tier_med_chars: String,
    /// Characters considered low priority.
    pub tier_low_chars: String,
    /// Bigrams that must be optimized for.
    pub critical_bigrams: String,
    /// Scale factors for finger repeat penalties.
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

impl Validator for LayoutDefinitions {
    fn validate(&self) -> Result<(), String> {
        if self.tier_high_chars.is_empty() { return Err("tier_high_chars cannot be empty".to_string()); }
        
        // Validate numeric strings
        if let Err(e) = parse_f32_array::<5>(&self.finger_repeat_scale) {
            return Err(format!("Invalid finger_repeat_scale: {}", e));
        }
        
        Ok(())
    }
}

impl LayoutDefinitions {
    /// Parses the critical bigrams string into a vector of byte arrays.
    pub fn get_critical_bigrams(&self) -> Vec<[u8; 2]> {
        self.critical_bigrams.split(',').filter_map(|s| {
            let b = s.trim().as_bytes();
            if b.len() == 2 { Some([b[0], b[1]]) } else { None }
        }).collect()
    }
}

fn parse_f32_array<const N: usize>(s: &str) -> Result<[f32; N], String> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != N { return Err(format!("Expected {} values, found {}", N, parts.len())); }
    let mut arr = [0.0; N];
    for (i, p) in parts.iter().enumerate() {
        let val: f32 = p.trim().parse().map_err(|_| format!("Invalid number: {}", p))?;
        if !val.is_finite() { return Err(format!("Value must be finite: {}", p)); }
        arr[i] = val;
    }
    Ok(arr)
}

/// Defines the source of the cost matrix used for scoring.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, ToSchema)]
#[serde(tag = "type", content = "data")]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub enum CostMatrixSource {
    /// Use a built-in cost matrix file.
    Predefined(String),
    /// Use a custom CSV string.
    Custom(String),
}

impl Default for CostMatrixSource {
    fn default() -> Self { CostMatrixSource::Predefined(ASSET_COST_MATRIX.to_string()) }
}

impl fmt::Display for CostMatrixSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CostMatrixSource::Predefined(s) => write!(f, "{}", s),
            CostMatrixSource::Custom(_) => write!(f, "<custom_content>"),
        }
    }
}

/// A constraint pinning a specific key to a specific physical index.
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct KeyConstraint {
    /// The physical index to pin.
    pub index: KeyIndex,
    /// The key label to pin (e.g., "A", "Shift").
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
