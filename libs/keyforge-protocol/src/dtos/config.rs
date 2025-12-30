use super::constants::{
    MAX_LOADER_TRIGRAM_LIMIT, MAX_OPT_LIMIT_FAST, MAX_SAFE_WEIGHT, MAX_SEARCH_EPOCHS,
    MAX_SEARCH_STEPS, MAX_TEMP,
};
use super::types::KeyIndex;
use super::validator::Validator;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use utoipa::ToSchema;
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema, TS)]
#[ts(export)]
pub struct Config {
    pub search: SearchParams,
    pub weights: ScoringWeights,
    pub defs: LayoutDefinitions,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema, TS)]
#[ts(export)]
pub struct CorpusSource {
    pub id: String,
    pub weight: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct SearchParams {
    pub search_epochs: usize,
    pub search_steps: usize,
    pub search_patience: usize,
    pub search_patience_threshold: f32,
    pub temp_min: f32,
    pub temp_max: f32,
    pub opt_limit_fast: usize,
    pub opt_limit_slow: usize,
    #[serde(default = "default_reheats")]
    pub reheats: usize,
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

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[serde(default)]
#[ts(export)]
pub struct ScoringWeights {
    pub penalty_sfr_weak_finger: f32,
    pub penalty_sfr_bad_row: f32,
    pub penalty_sfr_lat: f32,
    pub penalty_sfb_lateral: f32,
    pub penalty_sfb_lateral_weak: f32,
    pub penalty_sfb_base: f32,
    pub penalty_sfb_outward_adder: f32,
    pub penalty_sfb_diagonal: f32,
    pub penalty_sfb_long: f32,
    pub penalty_sfb_bottom: f32,
    pub weight_weak_finger_sfb: f32,
    pub threshold_sfb_long_row_diff: i8,
    pub threshold_scissor_row_diff: i8,
    pub threshold_reach_stretch: f32,
    pub penalty_scissor: f32,
    pub penalty_ring_pinky: f32,
    pub penalty_lateral: f32,
    pub penalty_monogram_stretch: f32,
    pub penalty_skip: f32,
    pub penalty_redirect: f32,
    pub penalty_hand_run: f32,
    pub bonus_inward_roll: f32,
    pub bonus_bigram_roll_in: f32,
    pub bonus_bigram_roll_out: f32,
    pub penalty_high_in_med: f32,
    pub penalty_high_in_low: f32,
    pub penalty_med_in_prime: f32,
    pub penalty_med_in_low: f32,
    pub penalty_low_in_prime: f32,
    pub penalty_low_in_med: f32,
    pub penalty_imbalance: f32,
    pub max_hand_imbalance: f32,
    pub weight_vertical_travel: f32,
    pub weight_lateral_travel: f32,
    pub weight_finger_effort: f32,
    pub default_cost_ms: f32,
    pub loader_trigram_limit: usize,
    pub trigram_coverage: f32,
    pub finger_penalty_scale: String,
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
            finger_penalty_scale: "".to_string(),
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
    pub fn get_finger_penalty_scale(&self) -> [f32; 5] {
        if self.finger_penalty_scale.is_empty() { return [0.0; 5]; }
        parse_f32_array::<5>(&self.finger_penalty_scale).unwrap_or([0.0; 5])
    }
    pub fn allowed_hand_balance_deviation(&self) -> f32 {
        (self.max_hand_imbalance - 0.5).max(0.0)
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
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[serde(default)]
#[ts(export)]
pub struct LayoutDefinitions {
    pub tier_high_chars: String,
    pub tier_med_chars: String,
    pub tier_low_chars: String,
    pub critical_bigrams: String,
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

impl LayoutDefinitions {
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, ToSchema, TS)]
#[serde(tag = "type", content = "data")]
#[ts(export)]
pub enum CostMatrixSource {
    Predefined(String),
    Custom(String),
}

impl Default for CostMatrixSource {
    fn default() -> Self { CostMatrixSource::Predefined("default_costmatrix.json".to_string()) }
}

impl fmt::Display for CostMatrixSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CostMatrixSource::Predefined(s) => write!(f, "{}", s),
            CostMatrixSource::Custom(_) => write!(f, "<custom_content>"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema, TS)]
#[ts(export)]
pub struct KeyConstraint {
    pub index: KeyIndex,
    pub key: String,
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
