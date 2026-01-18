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

/// Metadata describing a configuration parameter.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct ParameterMetadata {
    /// Internal key name.
    pub key: String,
    /// User-friendly label.
    pub label: String,
    /// Helpful description.
    pub description: String,
    /// Data type.
    pub param_type: ParamType,
    /// Minimum value (if numeric).
    pub min: Option<f32>,
    /// Maximum value (if numeric).
    pub max: Option<f32>,
    /// Default value.
    pub default: f32,
}

/// Supported data types for parameters.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
#[serde(rename_all = "lowercase")]
pub enum ParamType {
    /// Floating point number.
    Float,
    /// Integer number.
    Integer,
    /// Boolean toggle (mapped to 0.0/1.0 in map).
    Boolean,
}

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
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct SearchParams {
    /// Dynamic parameters map.
    #[serde(flatten)]
    pub params: std::collections::HashMap<String, f32>,
    /// Random seed for deterministic replay (Optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
    /// Whether to include thumb keys in swap suggestions.
    #[serde(default = "default_false")]
    pub include_thumbs: bool,
}

fn default_false() -> bool { false }

impl Default for SearchParams {
    fn default() -> Self {
        let mut params = std::collections::HashMap::new();
        params.insert("search_epochs".to_string(), DEFAULT_SEARCH_EPOCHS as f32);
        params.insert("search_steps".to_string(), DEFAULT_SEARCH_STEPS as f32);
        params.insert("search_patience".to_string(), DEFAULT_SEARCH_PATIENCE as f32);
        params.insert("search_patience_threshold".to_string(), DEFAULT_SEARCH_PATIENCE_THRESHOLD);
        params.insert("temp_min".to_string(), DEFAULT_TEMP_MIN);
        params.insert("temp_max".to_string(), DEFAULT_TEMP_MAX);
        params.insert("opt_limit_fast".to_string(), DEFAULT_OPT_LIMIT_FAST as f32);
        params.insert("opt_limit_slow".to_string(), DEFAULT_OPT_LIMIT_SLOW as f32);
        params.insert("reheats".to_string(), DEFAULT_REHEATS as f32);
        params.insert("reheat_factor".to_string(), DEFAULT_REHEAT_FACTOR);

        Self {
            params,
            seed: None,
            include_thumbs: false,
        }
    }
}

impl Validator for SearchParams {
    fn validate(&self) -> Result<(), String> {
        if self.get_search_epochs() == 0 { return Err("search_epochs must be > 0".into()); }
        if self.get_search_epochs() > MAX_SEARCH_EPOCHS {
            return Err(format!("search_epochs exceeds limit ({})", MAX_SEARCH_EPOCHS));
        }
        if self.get_search_steps() == 0 { return Err("search_steps must be > 0".into()); }
        if self.get_search_steps() > MAX_SEARCH_STEPS {
            return Err(format!("search_steps exceeds limit ({})", MAX_SEARCH_STEPS));
        }
        if self.get_opt_limit_fast() == 0 { return Err("opt_limit_fast must be > 0".into()); }
        if self.get_opt_limit_fast() > MAX_OPT_LIMIT_FAST {
            return Err(format!("opt_limit_fast exceeds limit ({})", MAX_OPT_LIMIT_FAST));
        }
        if self.get_opt_limit_slow() < self.get_opt_limit_fast() {
            return Err("opt_limit_slow must be >= opt_limit_fast".into());
        }
        if self.get_temp_min() < 0.0 || self.get_temp_max() < 0.0 {
            return Err("Temperature cannot be negative".into());
        }
        if self.get_temp_max() > MAX_TEMP {
            return Err(format!("temp_max exceeds limit ({})", MAX_TEMP));
        }
        if self.get_temp_min() < 0.0001 {
            return Err("temp_min too low (underflow risk)".into());
        }
        if self.get_temp_min() >= self.get_temp_max() {
            return Err("temp_min must be < temp_max".into());
        }
        if self.get_search_patience_threshold() < 0.0 || self.get_search_patience_threshold() > 1.0 {
            return Err("search_patience_threshold must be between 0.0 and 1.0".into());
        }
        Ok(())
    }
}

impl SearchParams {
    /// Returns the schema for search parameters.
    pub fn schema() -> Vec<ParameterMetadata> {
        vec![
            ParameterMetadata {
                key: "search_epochs".to_string(),
                label: "Search Epochs".to_string(),
                description: "Number of independent search runs to perform.".to_string(),
                param_type: ParamType::Integer,
                min: Some(1.0),
                max: Some(MAX_SEARCH_EPOCHS as f32),
                default: DEFAULT_SEARCH_EPOCHS as f32,
            },
            ParameterMetadata {
                key: "search_steps".to_string(),
                label: "Steps per Epoch".to_string(),
                description: "Maximum mutations to attempt per epoch.".to_string(),
                param_type: ParamType::Integer,
                min: Some(1000.0),
                max: Some(MAX_SEARCH_STEPS as f32),
                default: DEFAULT_SEARCH_STEPS as f32,
            },
            ParameterMetadata {
                key: "temp_max".to_string(),
                label: "Start Temperature".to_string(),
                description: "Initial chaos level (higher = more exploration).".to_string(),
                param_type: ParamType::Float,
                min: Some(0.1),
                max: Some(MAX_TEMP),
                default: DEFAULT_TEMP_MAX,
            },
            ParameterMetadata {
                key: "temp_min".to_string(),
                label: "End Temperature".to_string(),
                description: "Final greediness level (lower = more exploitation).".to_string(),
                param_type: ParamType::Float,
                min: Some(0.0001),
                max: Some(1.0),
                default: DEFAULT_TEMP_MIN,
            },
            ParameterMetadata {
                key: "search_patience".to_string(),
                label: "Patience".to_string(),
                description: "Steps without improvement before reheating.".to_string(),
                param_type: ParamType::Integer,
                min: Some(10.0),
                max: Some(10000.0),
                default: DEFAULT_SEARCH_PATIENCE as f32,
            },
            ParameterMetadata {
                key: "reheats".to_string(),
                label: "Reheats".to_string(),
                description: "Number of times to spike temperature when stuck.".to_string(),
                param_type: ParamType::Integer,
                min: Some(0.0),
                max: Some(10.0),
                default: DEFAULT_REHEATS as f32,
            },
        ]
    }

    /// Retrieves a parameter by key, falling back to a default value if not found.
    pub fn get_param(&self, key: &str, default: f32) -> f32 {
        self.params.get(key).copied().unwrap_or(default)
    }

    /// Number of independent search epochs to run.
    pub fn get_search_epochs(&self) -> usize { self.get_param("search_epochs", DEFAULT_SEARCH_EPOCHS as f32) as usize }
    /// Maximum number of mutation steps per epoch.
    pub fn get_search_steps(&self) -> usize { self.get_param("search_steps", DEFAULT_SEARCH_STEPS as f32) as usize }
    /// Steps without improvement before triggering a reheat.
    pub fn get_search_patience(&self) -> usize { self.get_param("search_patience", DEFAULT_SEARCH_PATIENCE as f32) as usize }
    /// Threshold for patience reset.
    pub fn get_search_patience_threshold(&self) -> f32 { self.get_param("search_patience_threshold", DEFAULT_SEARCH_PATIENCE_THRESHOLD) }
    /// Minimum temperature.
    pub fn get_temp_min(&self) -> f32 { self.get_param("temp_min", DEFAULT_TEMP_MIN) }
    /// Maximum temperature.
    pub fn get_temp_max(&self) -> f32 { self.get_param("temp_max", DEFAULT_TEMP_MAX) }
    /// Optimization limit for fast path.
    pub fn get_opt_limit_fast(&self) -> usize { self.get_param("opt_limit_fast", DEFAULT_OPT_LIMIT_FAST as f32) as usize }
    /// Optimization limit for slow path.
    pub fn get_opt_limit_slow(&self) -> usize { self.get_param("opt_limit_slow", DEFAULT_OPT_LIMIT_SLOW as f32) as usize }
    /// Number of times to reheat.
    pub fn get_reheats(&self) -> usize { self.get_param("reheats", DEFAULT_REHEATS as f32) as usize }
    /// Factor to multiply temperature by when reheating.
    pub fn get_reheat_factor(&self) -> f32 { self.get_param("reheat_factor", DEFAULT_REHEAT_FACTOR) }
}

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
        
        weights.insert("penalty_sfr_weak_finger".to_string(), DEFAULT_PENALTY_SFR_WEAK_FINGER);
        weights.insert("penalty_sfr_bad_row".to_string(), DEFAULT_PENALTY_SFR_BAD_ROW);
        weights.insert("penalty_sfr_lat".to_string(), DEFAULT_PENALTY_SFR_LAT);
        weights.insert("penalty_sfb_lateral".to_string(), DEFAULT_PENALTY_SFB_LATERAL);
        weights.insert("penalty_sfb_lateral_weak".to_string(), DEFAULT_PENALTY_SFB_LATERAL_WEAK);
        weights.insert("penalty_sfb_base".to_string(), DEFAULT_PENALTY_SFB_BASE);
        weights.insert("penalty_sfb_outward_adder".to_string(), DEFAULT_PENALTY_SFB_OUTWARD_ADDER);
        weights.insert("penalty_sfb_diagonal".to_string(), DEFAULT_PENALTY_SFB_DIAGONAL);
        weights.insert("penalty_sfb_long".to_string(), DEFAULT_PENALTY_SFB_LONG);
        weights.insert("penalty_sfb_bottom".to_string(), DEFAULT_PENALTY_SFB_BOTTOM);
        weights.insert("weight_weak_finger_sfb".to_string(), DEFAULT_WEIGHT_WEAK_FINGER_SFB);
        weights.insert("threshold_sfb_long_row_diff".to_string(), DEFAULT_THRESHOLD_SFB_LONG_ROW_DIFF as f32);
        weights.insert("threshold_scissor_row_diff".to_string(), DEFAULT_THRESHOLD_SCISSOR_ROW_DIFF as f32);
        weights.insert("threshold_reach_stretch".to_string(), DEFAULT_THRESHOLD_REACH_STRETCH);
        weights.insert("penalty_scissor".to_string(), DEFAULT_PENALTY_SCISSOR);
        weights.insert("penalty_ring_pinky".to_string(), DEFAULT_PENALTY_RING_PINKY);
        weights.insert("penalty_lateral".to_string(), DEFAULT_PENALTY_LATERAL);
        weights.insert("penalty_monogram_stretch".to_string(), DEFAULT_PENALTY_MONOGRAM_STRETCH);
        weights.insert("penalty_skip".to_string(), DEFAULT_PENALTY_SKIP);
        weights.insert("penalty_redirect".to_string(), DEFAULT_PENALTY_REDIRECT);
        weights.insert("penalty_hand_run".to_string(), DEFAULT_PENALTY_HAND_RUN);
        weights.insert("bonus_inward_roll".to_string(), DEFAULT_BONUS_INWARD_ROLL);
        weights.insert("bonus_bigram_roll_in".to_string(), DEFAULT_BONUS_BIGRAM_ROLL_IN);
        weights.insert("bonus_bigram_roll_out".to_string(), DEFAULT_BONUS_BIGRAM_ROLL_OUT);
        weights.insert("penalty_high_in_med".to_string(), DEFAULT_PENALTY_HIGH_IN_MED);
        weights.insert("penalty_high_in_low".to_string(), DEFAULT_PENALTY_HIGH_IN_LOW);
        weights.insert("penalty_med_in_prime".to_string(), DEFAULT_PENALTY_MED_IN_PRIME);
        weights.insert("penalty_med_in_low".to_string(), DEFAULT_PENALTY_MED_IN_LOW);
        weights.insert("penalty_low_in_prime".to_string(), DEFAULT_PENALTY_LOW_IN_PRIME);
        weights.insert("penalty_low_in_med".to_string(), DEFAULT_PENALTY_LOW_IN_MED);
        weights.insert("penalty_imbalance".to_string(), DEFAULT_PENALTY_IMBALANCE);
        weights.insert("max_hand_imbalance".to_string(), DEFAULT_MAX_HAND_IMBALANCE);
        weights.insert("weight_vertical_travel".to_string(), DEFAULT_WEIGHT_VERTICAL_TRAVEL);
        weights.insert("weight_lateral_travel".to_string(), DEFAULT_WEIGHT_LATERAL_TRAVEL);
        weights.insert("weight_finger_effort".to_string(), DEFAULT_WEIGHT_FINGER_EFFORT);
        weights.insert("default_cost_ms".to_string(), DEFAULT_COST_MS);
        weights.insert("loader_trigram_limit".to_string(), DEFAULT_LOADER_TRIGRAM_LIMIT as f32);
        weights.insert("trigram_coverage".to_string(), DEFAULT_TRIGRAM_COVERAGE);

        Self {
            weights,
            finger_penalty_scale: DEFAULT_FINGER_PENALTY_SCALE_ARRAY,
            comfortable_scissors: DEFAULT_COMFORTABLE_SCISSORS.to_string(),
        }
    }
}

impl Validator for ScoringWeights {
    fn validate(&self) -> Result<(), String> {
        if let Some(&limit) = self.weights.get("loader_trigram_limit") {
             if limit as usize > MAX_LOADER_TRIGRAM_LIMIT {
                 return Err(format!("loader_trigram_limit exceeds safety maximum ({})", MAX_LOADER_TRIGRAM_LIMIT));
             }
        }
        
        if self.get_penalty_sfb_base() < 0.0 || self.get_penalty_scissor() < 0.0 {
            return Err("Penalties cannot be negative".to_string());
        }
        if self.get_penalty_sfb_base() > MAX_SAFE_WEIGHT || self.get_penalty_scissor() > MAX_SAFE_WEIGHT {
            return Err(format!("Weights cannot exceed {:.0}", MAX_SAFE_WEIGHT));
        }
        for (i, &w) in self.finger_penalty_scale.iter().enumerate() {
            if w < 0.0 { return Err(format!("finger_penalty_scale[{}] cannot be negative", i)); }
        }
        Ok(())
    }
}

impl ScoringWeights {
    /// Returns the schema for scoring weights.
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
    pub fn get_weight(&self, key: &str, default: f32) -> f32 {
        self.weights.get(key).copied().unwrap_or(default)
    }

    /// Gets the penalty for Same Finger Repeat on a weak finger.
    pub fn get_penalty_sfr_weak_finger(&self) -> f32 { self.get_weight("penalty_sfr_weak_finger", DEFAULT_PENALTY_SFR_WEAK_FINGER) }
    /// Gets the penalty for Same Finger Repeat involving a bad row jump.
    pub fn get_penalty_sfr_bad_row(&self) -> f32 { self.get_weight("penalty_sfr_bad_row", DEFAULT_PENALTY_SFR_BAD_ROW) }
    /// Gets the penalty for lateral Same Finger Repeat.
    pub fn get_penalty_sfr_lat(&self) -> f32 { self.get_weight("penalty_sfr_lat", DEFAULT_PENALTY_SFR_LAT) }
    /// Gets the penalty for lateral Same Finger Bigram.
    pub fn get_penalty_sfb_lateral(&self) -> f32 { self.get_weight("penalty_sfb_lateral", DEFAULT_PENALTY_SFB_LATERAL) }
    /// Gets the penalty for lateral SFB on a weak finger.
    pub fn get_penalty_sfb_lateral_weak(&self) -> f32 { self.get_weight("penalty_sfb_lateral_weak", DEFAULT_PENALTY_SFB_LATERAL_WEAK) }
    /// Gets the base penalty for any Same Finger Bigram.
    pub fn get_penalty_sfb_base(&self) -> f32 { self.get_weight("penalty_sfb_base", DEFAULT_PENALTY_SFB_BASE) }
    /// Gets the additional penalty for outward rolling SFBs.
    pub fn get_penalty_sfb_outward_adder(&self) -> f32 { self.get_weight("penalty_sfb_outward_adder", DEFAULT_PENALTY_SFB_OUTWARD_ADDER) }
    /// Gets the penalty for diagonal SFBs.
    pub fn get_penalty_sfb_diagonal(&self) -> f32 { self.get_weight("penalty_sfb_diagonal", DEFAULT_PENALTY_SFB_DIAGONAL) }
    /// Gets the penalty for long-distance SFBs.
    pub fn get_penalty_sfb_long(&self) -> f32 { self.get_weight("penalty_sfb_long", DEFAULT_PENALTY_SFB_LONG) }
    /// Gets the penalty for bottom-row SFBs.
    pub fn get_penalty_sfb_bottom(&self) -> f32 { self.get_weight("penalty_sfb_bottom", DEFAULT_PENALTY_SFB_BOTTOM) }
    /// Gets the multiplier for SFBs on weak fingers.
    pub fn get_weight_weak_finger_sfb(&self) -> f32 { self.get_weight("weight_weak_finger_sfb", DEFAULT_WEIGHT_WEAK_FINGER_SFB) }
    
    /// Gets the row difference threshold for "long" SFBs.
    pub fn get_threshold_sfb_long_row_diff(&self) -> i8 { self.get_weight("threshold_sfb_long_row_diff", DEFAULT_THRESHOLD_SFB_LONG_ROW_DIFF as f32) as i8 }
    /// Gets the row difference threshold for scissors.
    pub fn get_threshold_scissor_row_diff(&self) -> i8 { self.get_weight("threshold_scissor_row_diff", DEFAULT_THRESHOLD_SCISSOR_ROW_DIFF as f32) as i8 }
    /// Gets the distance threshold for reach stretches.
    pub fn get_threshold_reach_stretch(&self) -> f32 { self.get_weight("threshold_reach_stretch", DEFAULT_THRESHOLD_REACH_STRETCH) }
    
    /// Gets the penalty for scissor (adjacent finger stretch) movements.
    pub fn get_penalty_scissor(&self) -> f32 { self.get_weight("penalty_scissor", DEFAULT_PENALTY_SCISSOR) }
    /// Gets the penalty for ring-pinky interactions.
    pub fn get_penalty_ring_pinky(&self) -> f32 { self.get_weight("penalty_ring_pinky", DEFAULT_PENALTY_RING_PINKY) }
    /// Gets the penalty for lateral movement.
    pub fn get_penalty_lateral(&self) -> f32 { self.get_weight("penalty_lateral", DEFAULT_PENALTY_LATERAL) }
    /// Gets the penalty for single-key stretches.
    pub fn get_penalty_monogram_stretch(&self) -> f32 { self.get_weight("penalty_monogram_stretch", DEFAULT_PENALTY_MONOGRAM_STRETCH) }
    /// Gets the penalty for skipping a key (hurdle).
    pub fn get_penalty_skip(&self) -> f32 { self.get_weight("penalty_skip", DEFAULT_PENALTY_SKIP) }
    /// Gets the penalty for redirecting flow (e.g., Left -> Right -> Left).
    pub fn get_penalty_redirect(&self) -> f32 { self.get_weight("penalty_redirect", DEFAULT_PENALTY_REDIRECT) }
    /// Gets the penalty for excessive hand alternation runs.
    pub fn get_penalty_hand_run(&self) -> f32 { self.get_weight("penalty_hand_run", DEFAULT_PENALTY_HAND_RUN) }
    /// Gets the bonus (negative cost) for inward rolls.
    pub fn get_bonus_inward_roll(&self) -> f32 { self.get_weight("bonus_inward_roll", DEFAULT_BONUS_INWARD_ROLL) }
    /// Gets the bonus for specific bigram inward rolls.
    pub fn get_bonus_bigram_roll_in(&self) -> f32 { self.get_weight("bonus_bigram_roll_in", DEFAULT_BONUS_BIGRAM_ROLL_IN) }
    /// Gets the bonus for specific bigram outward rolls.
    pub fn get_bonus_bigram_roll_out(&self) -> f32 { self.get_weight("bonus_bigram_roll_out", DEFAULT_BONUS_BIGRAM_ROLL_OUT) }
    
    /// Gets the penalty for high-frequency keys in medium slots.
    pub fn get_penalty_high_in_med(&self) -> f32 { self.get_weight("penalty_high_in_med", DEFAULT_PENALTY_HIGH_IN_MED) }
    /// Gets the penalty for high-frequency keys in low slots.
    pub fn get_penalty_high_in_low(&self) -> f32 { self.get_weight("penalty_high_in_low", DEFAULT_PENALTY_HIGH_IN_LOW) }
    /// Gets the penalty for medium-frequency keys in prime slots.
    pub fn get_penalty_med_in_prime(&self) -> f32 { self.get_weight("penalty_med_in_prime", DEFAULT_PENALTY_MED_IN_PRIME) }
    /// Gets the penalty for medium-frequency keys in low slots.
    pub fn get_penalty_med_in_low(&self) -> f32 { self.get_weight("penalty_med_in_low", DEFAULT_PENALTY_MED_IN_LOW) }
    /// Gets the penalty for low-frequency keys in prime slots.
    pub fn get_penalty_low_in_prime(&self) -> f32 { self.get_weight("penalty_low_in_prime", DEFAULT_PENALTY_LOW_IN_PRIME) }
    /// Gets the penalty for low-frequency keys in medium slots.
    pub fn get_penalty_low_in_med(&self) -> f32 { self.get_weight("penalty_low_in_med", DEFAULT_PENALTY_LOW_IN_MED) }
    
    /// Gets the penalty for hand imbalance.
    pub fn get_penalty_imbalance(&self) -> f32 { self.get_weight("penalty_imbalance", DEFAULT_PENALTY_IMBALANCE) }
    /// Gets the maximum allowed hand imbalance ratio.
    pub fn get_max_hand_imbalance(&self) -> f32 { self.get_weight("max_hand_imbalance", DEFAULT_MAX_HAND_IMBALANCE) }
    
    /// Gets the weight multiplier for vertical travel distance.
    pub fn get_weight_vertical_travel(&self) -> f32 { self.get_weight("weight_vertical_travel", DEFAULT_WEIGHT_VERTICAL_TRAVEL) }
    /// Gets the weight multiplier for lateral travel distance.
    pub fn get_weight_lateral_travel(&self) -> f32 { self.get_weight("weight_lateral_travel", DEFAULT_WEIGHT_LATERAL_TRAVEL) }
    /// Gets the weight multiplier for finger effort.
    pub fn get_weight_finger_effort(&self) -> f32 { self.get_weight("weight_finger_effort", DEFAULT_WEIGHT_FINGER_EFFORT) }
    
    /// Gets the default cost in milliseconds (if using time-based scoring).
    pub fn get_default_cost_ms(&self) -> f32 { self.get_weight("default_cost_ms", DEFAULT_COST_MS) }
    /// Gets the limit on the number of trigrams to load.
    pub fn get_loader_trigram_limit(&self) -> usize { self.get_weight("loader_trigram_limit", DEFAULT_LOADER_TRIGRAM_LIMIT as f32) as usize }
    /// Gets the required trigram coverage.
    pub fn get_trigram_coverage(&self) -> f32 { self.get_weight("trigram_coverage", DEFAULT_TRIGRAM_COVERAGE) }

    /// Returns the finger penalty scale array.
    pub fn get_finger_penalty_scale(&self) -> [f32; 5] {
        self.finger_penalty_scale
    }
    /// Calculates the allowed deviation from perfect hand balance.
    pub fn allowed_hand_balance_deviation(&self) -> f32 {
        (self.get_max_hand_imbalance() - 0.5).max(0.0)
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
