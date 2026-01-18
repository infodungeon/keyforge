// apps/keyforge-cli/src/cli_args/config.rs

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


use clap::Args;
use keyforge_model::config::{Config, LayoutDefinitions, ScoringWeights, SearchParams};
use keyforge_model::constants::*;

/// Top-level configuration arguments combining search, weights, and definitions.
#[derive(Args, Debug, Clone)]
pub struct ConfigArgs {
    #[command(flatten)]
    pub search: SearchParamsArgs,
    #[command(flatten)]
    pub weights: ScoringWeightsArgs,
    #[command(flatten)]
    pub defs: LayoutDefinitionsArgs,
}

/// Arguments controlling the Simulated Annealing search process.
#[derive(Args, Debug, Clone, Copy)]
pub struct SearchParamsArgs {
    /// Number of independent search epochs to run.
    #[arg(long, default_value_t = DEFAULT_SEARCH_EPOCHS)]
    pub search_epochs: usize,
    /// Maximum number of mutation steps per epoch.
    #[arg(long, default_value_t = DEFAULT_SEARCH_STEPS)]
    pub search_steps: usize,
    /// Steps without improvement before triggering a reheat.
    #[arg(long, default_value_t = DEFAULT_SEARCH_PATIENCE)]
    pub search_patience: usize,
    /// Threshold for patience reset (improvement must be > this).
    #[arg(long, default_value_t = DEFAULT_SEARCH_PATIENCE_THRESHOLD)]
    pub search_patience_threshold: f32,

    /// Minimum temperature (stop condition).
    #[arg(long, default_value_t = DEFAULT_TEMP_MIN)]
    pub temp_min: f32,
    /// Maximum temperature (start condition).
    #[arg(long, default_value_t = DEFAULT_TEMP_MAX)]
    pub temp_max: f32,

    /// Optimization limit for fast path.
    #[arg(long, default_value_t = DEFAULT_OPT_LIMIT_FAST)]
    pub opt_limit_fast: usize,
    /// Optimization limit for slow path.
    #[arg(long, default_value_t = DEFAULT_OPT_LIMIT_SLOW)]
    pub opt_limit_slow: usize,

    /// Number of times to reheat the system if stuck.
    #[arg(long, default_value_t = DEFAULT_REHEATS)]
    pub reheats: usize,
    /// Factor to multiply temperature by when reheating.
    #[arg(long, default_value_t = DEFAULT_REHEAT_FACTOR)]
    pub reheat_factor: f32,
}

/// Arguments defining the scoring weights and penalties.
#[derive(Args, Debug, Clone)]
pub struct ScoringWeightsArgs {
    /// Penalty for Same Finger Repeat on a weak finger.
    #[arg(long, default_value_t = DEFAULT_PENALTY_SFR_WEAK_FINGER)]
    pub penalty_sfr_weak_finger: f32,
    /// Penalty for Same Finger Repeat involving a bad row jump.
    #[arg(long, default_value_t = DEFAULT_PENALTY_SFR_BAD_ROW)]
    pub penalty_sfr_bad_row: f32,
    /// Penalty for lateral Same Finger Repeat.
    #[arg(long, default_value_t = DEFAULT_PENALTY_SFR_LAT)]
    pub penalty_sfr_lat: f32,
    /// Penalty for lateral Same Finger Bigram.
    #[arg(long, default_value_t = DEFAULT_PENALTY_SFB_LATERAL)]
    pub penalty_sfb_lateral: f32,
    /// Penalty for lateral SFB on a weak finger.
    #[arg(long, default_value_t = DEFAULT_PENALTY_SFB_LATERAL_WEAK)]
    pub penalty_sfb_lateral_weak: f32,
    /// Base penalty for any Same Finger Bigram.
    #[arg(long, default_value_t = DEFAULT_PENALTY_SFB_BASE)]
    pub penalty_sfb_base: f32,
    /// Additional penalty for outward rolling SFBs.
    #[arg(long, default_value_t = DEFAULT_PENALTY_SFB_OUTWARD_ADDER)]
    pub penalty_sfb_outward_adder: f32,
    /// Penalty for diagonal SFBs.
    #[arg(long, default_value_t = DEFAULT_PENALTY_SFB_DIAGONAL)]
    pub penalty_sfb_diagonal: f32,
    /// Penalty for long-distance SFBs.
    #[arg(long, default_value_t = DEFAULT_PENALTY_SFB_LONG)]
    pub penalty_sfb_long: f32,
    /// Penalty for bottom-row SFBs.
    #[arg(long, default_value_t = DEFAULT_PENALTY_SFB_BOTTOM)]
    pub penalty_sfb_bottom: f32,
    /// Multiplier for SFBs on weak fingers.
    #[arg(long, default_value_t = DEFAULT_WEIGHT_WEAK_FINGER_SFB)]
    pub weight_weak_finger_sfb: f32,

    /// Row difference threshold for "long" SFBs.
    #[arg(long, default_value_t = DEFAULT_THRESHOLD_SFB_LONG_ROW_DIFF)]
    pub threshold_sfb_long_row_diff: i8,
    /// Row difference threshold for scissors.
    #[arg(long, default_value_t = DEFAULT_THRESHOLD_SCISSOR_ROW_DIFF)]
    pub threshold_scissor_row_diff: i8,

    /// Distance threshold for reach stretches.
    #[arg(long, default_value_t = DEFAULT_THRESHOLD_REACH_STRETCH)]
    pub threshold_reach_stretch: f32,

    /// Penalty for scissor (adjacent finger stretch) movements.
    #[arg(long, default_value_t = DEFAULT_PENALTY_SCISSOR)]
    pub penalty_scissor: f32,
    /// Penalty for ring-pinky interactions.
    #[arg(long, default_value_t = DEFAULT_PENALTY_RING_PINKY)]
    pub penalty_ring_pinky: f32,
    /// Penalty for lateral movement.
    #[arg(long, default_value_t = DEFAULT_PENALTY_LATERAL)]
    pub penalty_lateral: f32,
    /// Penalty for single-key stretches.
    #[arg(long, default_value_t = DEFAULT_PENALTY_MONOGRAM_STRETCH)]
    pub penalty_monogram_stretch: f32,
    /// Penalty for skipping a key (hurdle).
    #[arg(long, default_value_t = DEFAULT_PENALTY_SKIP)]
    pub penalty_skip: f32,
    /// Penalty for redirecting flow (e.g., Left -> Right -> Left).
    #[arg(long, default_value_t = DEFAULT_PENALTY_REDIRECT)]
    pub penalty_redirect: f32,
    /// Penalty for excessive hand alternation runs.
    #[arg(long, default_value_t = DEFAULT_PENALTY_HAND_RUN)]
    pub penalty_hand_run: f32,
    /// Bonus (negative cost) for inward rolls.
    #[arg(long, default_value_t = DEFAULT_BONUS_INWARD_ROLL)]
    pub bonus_inward_roll: f32,
    /// Bonus for specific bigram inward rolls.
    #[arg(long, default_value_t = DEFAULT_BONUS_BIGRAM_ROLL_IN)]
    pub bonus_bigram_roll_in: f32,
    /// Bonus for specific bigram outward rolls.
    #[arg(long, default_value_t = DEFAULT_BONUS_BIGRAM_ROLL_OUT)]
    pub bonus_bigram_roll_out: f32,
    /// Penalty for high-frequency keys in medium slots.
    #[arg(long, default_value_t = DEFAULT_PENALTY_HIGH_IN_MED)]
    pub penalty_high_in_med: f32,
    /// Penalty for high-frequency keys in low slots.
    #[arg(long, default_value_t = DEFAULT_PENALTY_HIGH_IN_LOW)]
    pub penalty_high_in_low: f32,
    /// Penalty for medium-frequency keys in prime slots.
    #[arg(long, default_value_t = DEFAULT_PENALTY_MED_IN_PRIME)]
    pub penalty_med_in_prime: f32,
    /// Penalty for medium-frequency keys in low slots.
    #[arg(long, default_value_t = DEFAULT_PENALTY_MED_IN_LOW)]
    pub penalty_med_in_low: f32,
    /// Penalty for low-frequency keys in prime slots.
    #[arg(long, default_value_t = DEFAULT_PENALTY_LOW_IN_PRIME)]
    pub penalty_low_in_prime: f32,
    /// Penalty for low-frequency keys in medium slots.
    #[arg(long, default_value_t = DEFAULT_PENALTY_LOW_IN_MED)]
    pub penalty_low_in_med: f32,
    /// Penalty for hand imbalance.
    #[arg(long, default_value_t = DEFAULT_PENALTY_IMBALANCE)]
    pub penalty_imbalance: f32,
    /// Maximum allowed hand imbalance ratio.
    #[arg(long, default_value_t = DEFAULT_MAX_HAND_IMBALANCE)]
    pub max_hand_imbalance: f32,
    /// Weight multiplier for vertical travel distance.
    #[arg(long, default_value_t = DEFAULT_WEIGHT_VERTICAL_TRAVEL)]
    pub weight_vertical_travel: f32,
    /// Weight multiplier for lateral travel distance.
    #[arg(long, default_value_t = DEFAULT_WEIGHT_LATERAL_TRAVEL)]
    pub weight_lateral_travel: f32,
    /// Weight multiplier for finger effort.
    #[arg(long, default_value_t = DEFAULT_WEIGHT_FINGER_EFFORT)]
    pub weight_finger_effort: f32,

    /// Default cost in milliseconds (if using time-based scoring).
    #[arg(long, default_value_t = DEFAULT_COST_MS)]
    pub default_cost_ms: f32,
    /// Limit on the number of trigrams to load.
    #[arg(long, default_value_t = DEFAULT_LOADER_TRIGRAM_LIMIT)]
    pub loader_trigram_limit: usize,
    /// Required trigram coverage (0.0 - 1.0).
    #[arg(long, default_value_t = DEFAULT_TRIGRAM_COVERAGE)]
    pub trigram_coverage: f32,

    /// Finger penalty multipliers (Thumb, Index, Middle, Ring, Pinky).
    #[arg(long, value_delimiter = ',', num_args = 5, default_values_t = DEFAULT_FINGER_PENALTY_SCALE_ARRAY)]
    pub finger_penalty_scale: Vec<f32>,

    /// Comma-separated list of comfortable scissor pairs.
    #[arg(long, default_value = DEFAULT_COMFORTABLE_SCISSORS)]
    pub comfortable_scissors: String,
}

/// Arguments defining character tiers and critical bigrams.
#[derive(Args, Debug, Clone)]
pub struct LayoutDefinitionsArgs {
    /// Characters considered high priority.
    #[arg(long, default_value = DEFAULT_TIER_HIGH)]
    pub tier_high_chars: String,
    /// Characters considered medium priority.
    #[arg(long, default_value = DEFAULT_TIER_MED)]
    pub tier_med_chars: String,
    /// Characters considered low priority.
    #[arg(long, default_value = DEFAULT_TIER_LOW)]
    pub tier_low_chars: String,
    /// Bigrams that must be optimized for.
    #[arg(long, default_value = DEFAULT_CRITICAL_BIGRAMS)]
    pub critical_bigrams: String,
    /// Scale factors for finger repeat penalties.
    #[arg(long, value_delimiter = ',', num_args = 5, default_values_t = DEFAULT_FINGER_REPEAT_SCALE_ARRAY)]
    pub finger_repeat_scale: Vec<f32>,
}

use keyforge_model::Validator;
use std::convert::TryFrom;

impl TryFrom<ConfigArgs> for Config {
    type Error = String;
    fn try_from(args: ConfigArgs) -> Result<Self, Self::Error> {
        let config = Self {
            search: SearchParams::try_from(args.search)?,
            weights: ScoringWeights::try_from(args.weights)?,
            defs: LayoutDefinitions::try_from(args.defs)?,
        };
        config.search.validate()?;
        config.weights.validate()?;
        Ok(config)
    }
}

impl TryFrom<SearchParamsArgs> for SearchParams {
    type Error = String;
    fn try_from(args: SearchParamsArgs) -> Result<Self, Self::Error> {
        let mut p = SearchParams::default();
        
        p.params.insert("search_epochs".to_string(), args.search_epochs as f32);
        p.params.insert("search_steps".to_string(), args.search_steps as f32);
        p.params.insert("search_patience".to_string(), args.search_patience as f32);
        p.params.insert("search_patience_threshold".to_string(), args.search_patience_threshold);
        p.params.insert("temp_min".to_string(), args.temp_min);
        p.params.insert("temp_max".to_string(), args.temp_max);
        p.params.insert("opt_limit_fast".to_string(), args.opt_limit_fast as f32);
        p.params.insert("opt_limit_slow".to_string(), args.opt_limit_slow as f32);
        p.params.insert("reheats".to_string(), args.reheats as f32);
        p.params.insert("reheat_factor".to_string(), args.reheat_factor);
        
        p.seed = None;
        p.include_thumbs = false;
        
        p.validate()?;
        Ok(p)
    }
}

impl TryFrom<ScoringWeightsArgs> for ScoringWeights {
    type Error = String;
    fn try_from(args: ScoringWeightsArgs) -> Result<Self, Self::Error> {
        let mut w = ScoringWeights::default();
        
        w.weights.insert("penalty_sfr_weak_finger".to_string(), args.penalty_sfr_weak_finger);
        w.weights.insert("penalty_sfr_bad_row".to_string(), args.penalty_sfr_bad_row);
        w.weights.insert("penalty_sfr_lat".to_string(), args.penalty_sfr_lat);
        w.weights.insert("penalty_sfb_lateral".to_string(), args.penalty_sfb_lateral);
        w.weights.insert("penalty_sfb_lateral_weak".to_string(), args.penalty_sfb_lateral_weak);
        w.weights.insert("penalty_sfb_base".to_string(), args.penalty_sfb_base);
        w.weights.insert("penalty_sfb_outward_adder".to_string(), args.penalty_sfb_outward_adder);
        w.weights.insert("penalty_sfb_diagonal".to_string(), args.penalty_sfb_diagonal);
        w.weights.insert("penalty_sfb_long".to_string(), args.penalty_sfb_long);
        w.weights.insert("penalty_sfb_bottom".to_string(), args.penalty_sfb_bottom);
        w.weights.insert("weight_weak_finger_sfb".to_string(), args.weight_weak_finger_sfb);
        w.weights.insert("threshold_sfb_long_row_diff".to_string(), args.threshold_sfb_long_row_diff as f32);
        w.weights.insert("threshold_scissor_row_diff".to_string(), args.threshold_scissor_row_diff as f32);
        w.weights.insert("threshold_reach_stretch".to_string(), args.threshold_reach_stretch);
        w.weights.insert("penalty_scissor".to_string(), args.penalty_scissor);
        w.weights.insert("penalty_ring_pinky".to_string(), args.penalty_ring_pinky);
        w.weights.insert("penalty_lateral".to_string(), args.penalty_lateral);
        w.weights.insert("penalty_monogram_stretch".to_string(), args.penalty_monogram_stretch);
        w.weights.insert("penalty_skip".to_string(), args.penalty_skip);
        w.weights.insert("penalty_redirect".to_string(), args.penalty_redirect);
        w.weights.insert("penalty_hand_run".to_string(), args.penalty_hand_run);
        w.weights.insert("bonus_inward_roll".to_string(), args.bonus_inward_roll);
        w.weights.insert("bonus_bigram_roll_in".to_string(), args.bonus_bigram_roll_in);
        w.weights.insert("bonus_bigram_roll_out".to_string(), args.bonus_bigram_roll_out);
        w.weights.insert("penalty_high_in_med".to_string(), args.penalty_high_in_med);
        w.weights.insert("penalty_high_in_low".to_string(), args.penalty_high_in_low);
        w.weights.insert("penalty_med_in_prime".to_string(), args.penalty_med_in_prime);
        w.weights.insert("penalty_med_in_low".to_string(), args.penalty_med_in_low);
        w.weights.insert("penalty_low_in_prime".to_string(), args.penalty_low_in_prime);
        w.weights.insert("penalty_low_in_med".to_string(), args.penalty_low_in_med);
        w.weights.insert("penalty_imbalance".to_string(), args.penalty_imbalance);
        w.weights.insert("max_hand_imbalance".to_string(), args.max_hand_imbalance);
        w.weights.insert("weight_vertical_travel".to_string(), args.weight_vertical_travel);
        w.weights.insert("weight_lateral_travel".to_string(), args.weight_lateral_travel);
        w.weights.insert("weight_finger_effort".to_string(), args.weight_finger_effort);
        w.weights.insert("default_cost_ms".to_string(), args.default_cost_ms);
        w.weights.insert("loader_trigram_limit".to_string(), args.loader_trigram_limit as f32);
        w.weights.insert("trigram_coverage".to_string(), args.trigram_coverage);
        
        w.finger_penalty_scale = vec_to_array_5(args.finger_penalty_scale)?;
        w.comfortable_scissors = args.comfortable_scissors;
        
        w.validate()?;
        Ok(w)
    }
}

impl TryFrom<LayoutDefinitionsArgs> for LayoutDefinitions {
    type Error = String;
    fn try_from(args: LayoutDefinitionsArgs) -> Result<Self, Self::Error> {
        Ok(Self {
            tier_high_chars: args.tier_high_chars,
            tier_med_chars: args.tier_med_chars,
            tier_low_chars: args.tier_low_chars,
            critical_bigrams: args.critical_bigrams,
            finger_repeat_scale: vec_to_array_5(args.finger_repeat_scale)?,
        })
    }
}

fn vec_to_array_5(v: Vec<f32>) -> Result<[f32; 5], String> {
    if v.len() != 5 {
        return Err(format!("Expected 5 values, got {}", v.len()));
    }
    let mut arr = [0.0; 5];
    for (i, &val) in v.iter().enumerate() {
        arr[i] = val;
    }
    Ok(arr)
}
