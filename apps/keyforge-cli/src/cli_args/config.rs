// apps/keyforge-cli/src/cli_args/config.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.


use clap::Args;
use keyforge_model::config::{Config, LayoutDefinitions, ScoringWeights, SearchParams};

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
    #[arg(long, default_value_t = 10_000)]
    pub search_epochs: usize,
    /// Maximum number of mutation steps per epoch.
    #[arg(long, default_value_t = 100_000)]
    pub search_steps: usize,
    /// Steps without improvement before triggering a reheat.
    #[arg(long, default_value_t = 500)]
    pub search_patience: usize,
    /// Threshold for patience reset (improvement must be > this).
    #[arg(long, default_value_t = 0.1)]
    pub search_patience_threshold: f32,

    /// Minimum temperature (stop condition).
    #[arg(long, default_value_t = 0.005)]
    pub temp_min: f32,
    /// Maximum temperature (start condition).
    #[arg(long, default_value_t = 20.0)]
    pub temp_max: f32,

    /// Optimization limit for fast path.
    #[arg(long, default_value_t = 100)]
    pub opt_limit_fast: usize,
    /// Optimization limit for slow path.
    #[arg(long, default_value_t = 1500)]
    pub opt_limit_slow: usize,

    /// Number of times to reheat the system if stuck.
    #[arg(long, default_value_t = 3)]
    pub reheats: usize,
    /// Factor to multiply temperature by when reheating.
    #[arg(long, default_value_t = 0.5)]
    pub reheat_factor: f32,
}

/// Arguments defining the scoring weights and penalties.
#[derive(Args, Debug, Clone)]
pub struct ScoringWeightsArgs {
    /// Penalty for Same Finger Repeat on a weak finger.
    #[arg(long, default_value_t = 20.0)]
    pub penalty_sfr_weak_finger: f32,
    /// Penalty for Same Finger Repeat involving a bad row jump.
    #[arg(long, default_value_t = 25.0)]
    pub penalty_sfr_bad_row: f32,
    /// Penalty for lateral Same Finger Repeat.
    #[arg(long, default_value_t = 40.0)]
    pub penalty_sfr_lat: f32,
    /// Penalty for lateral Same Finger Bigram.
    #[arg(long, default_value_t = 65.0)]
    pub penalty_sfb_lateral: f32,
    /// Penalty for lateral SFB on a weak finger.
    #[arg(long, default_value_t = 160.0)]
    pub penalty_sfb_lateral_weak: f32,
    /// Base penalty for any Same Finger Bigram.
    #[arg(long, default_value_t = 400.0)]
    pub penalty_sfb_base: f32,
    /// Additional penalty for outward rolling SFBs.
    #[arg(long, default_value_t = 10.0)]
    pub penalty_sfb_outward_adder: f32,
    /// Penalty for diagonal SFBs.
    #[arg(long, default_value_t = 240.0)]
    pub penalty_sfb_diagonal: f32,
    /// Penalty for long-distance SFBs.
    #[arg(long, default_value_t = 280.0)]
    pub penalty_sfb_long: f32,
    /// Penalty for bottom-row SFBs.
    #[arg(long, default_value_t = 45.0)]
    pub penalty_sfb_bottom: f32,
    /// Multiplier for SFBs on weak fingers.
    #[arg(long, default_value_t = 2.7)]
    pub weight_weak_finger_sfb: f32,

    /// Row difference threshold for "long" SFBs.
    #[arg(long, default_value_t = 2)]
    pub threshold_sfb_long_row_diff: i8,
    /// Row difference threshold for scissors.
    #[arg(long, default_value_t = 2)]
    pub threshold_scissor_row_diff: i8,

    /// Distance threshold for reach stretches.
    #[arg(long, default_value_t = 1.2)]
    pub threshold_reach_stretch: f32,

    /// Penalty for scissor (adjacent finger stretch) movements.
    #[arg(long, default_value_t = 25.0)]
    pub penalty_scissor: f32,
    /// Penalty for ring-pinky interactions.
    #[arg(long, default_value_t = 1.3)]
    pub penalty_ring_pinky: f32,
    /// Penalty for lateral movement.
    #[arg(long, default_value_t = 50.0)]
    pub penalty_lateral: f32,
    /// Penalty for single-key stretches.
    #[arg(long, default_value_t = 20.0)]
    pub penalty_monogram_stretch: f32,
    /// Penalty for skipping a key (hurdle).
    #[arg(long, default_value_t = 20.0)]
    pub penalty_skip: f32,
    /// Penalty for redirecting flow (e.g., Left -> Right -> Left).
    #[arg(long, default_value_t = 65.0)]
    pub penalty_redirect: f32,
    /// Penalty for excessive hand alternation runs.
    #[arg(long, default_value_t = 5.0)]
    pub penalty_hand_run: f32,
    /// Bonus (negative cost) for inward rolls.
    #[arg(long, default_value_t = 40.0)]
    pub bonus_inward_roll: f32,
    /// Bonus for specific bigram inward rolls.
    #[arg(long, default_value_t = 35.0)]
    pub bonus_bigram_roll_in: f32,
    /// Bonus for specific bigram outward rolls.
    #[arg(long, default_value_t = 25.0)]
    pub bonus_bigram_roll_out: f32,
    /// Penalty for high-frequency keys in medium slots.
    #[arg(long, default_value_t = 12.0)]
    pub penalty_high_in_med: f32,
    /// Penalty for high-frequency keys in low slots.
    #[arg(long, default_value_t = 20.0)]
    pub penalty_high_in_low: f32,
    /// Penalty for medium-frequency keys in prime slots.
    #[arg(long, default_value_t = 2.0)]
    pub penalty_med_in_prime: f32,
    /// Penalty for medium-frequency keys in low slots.
    #[arg(long, default_value_t = 2.0)]
    pub penalty_med_in_low: f32,
    /// Penalty for low-frequency keys in prime slots.
    #[arg(long, default_value_t = 15.0)]
    pub penalty_low_in_prime: f32,
    /// Penalty for low-frequency keys in medium slots.
    #[arg(long, default_value_t = 2.0)]
    pub penalty_low_in_med: f32,
    /// Penalty for hand imbalance.
    #[arg(long, default_value_t = 200.0)]
    pub penalty_imbalance: f32,
    /// Maximum allowed hand imbalance ratio.
    #[arg(long, default_value_t = 0.55)]
    pub max_hand_imbalance: f32,
    /// Weight multiplier for vertical travel distance.
    #[arg(long, default_value_t = 1.0)]
    pub weight_vertical_travel: f32,
    /// Weight multiplier for lateral travel distance.
    #[arg(long, default_value_t = 3.5)]
    pub weight_lateral_travel: f32,
    /// Weight multiplier for finger effort.
    #[arg(long, default_value_t = 2.2)]
    pub weight_finger_effort: f32,

    /// Default cost in milliseconds (if using time-based scoring).
    #[arg(long, default_value_t = 120.0)]
    pub default_cost_ms: f32,
    /// Limit on the number of trigrams to load.
    #[arg(long, default_value_t = 3000)]
    pub loader_trigram_limit: usize,
    /// Required trigram coverage (0.0 - 1.0).
    #[arg(long, default_value_t = 0.99)]
    pub trigram_coverage: f32,

    /// Finger penalty multipliers (Thumb, Index, Middle, Ring, Pinky).
    #[arg(long, default_value = "0.0,1.0,1.1,1.3,1.6")]
    pub finger_penalty_scale: String,

    /// Comma-separated list of comfortable scissor pairs.
    #[arg(long, default_value = "21,23,34")]
    pub comfortable_scissors: String,
}

/// Arguments defining character tiers and critical bigrams.
#[derive(Args, Debug, Clone)]
pub struct LayoutDefinitionsArgs {
    /// Characters considered high priority.
    #[arg(long, default_value = "etaoinshr")]
    pub tier_high_chars: String,
    /// Characters considered medium priority.
    #[arg(long, default_value = "ldcumwfgypb.,")]
    pub tier_med_chars: String,
    /// Characters considered low priority.
    #[arg(long, default_value = "vkjxqz/;")]
    pub tier_low_chars: String,
    /// Bigrams that must be optimized for.
    #[arg(long, default_value = "th,he,in,er,an,re,nd,ou")]
    pub critical_bigrams: String,
    /// Scale factors for finger repeat penalties.
    #[arg(long, default_value = "1.0,1.0,1.0,1.2,1.5")]
    pub finger_repeat_scale: String,
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
        let p = Self {
            search_epochs: args.search_epochs,
            search_steps: args.search_steps,
            search_patience: args.search_patience,
            search_patience_threshold: args.search_patience_threshold,
            temp_min: args.temp_min,
            temp_max: args.temp_max,
            opt_limit_fast: args.opt_limit_fast,
            opt_limit_slow: args.opt_limit_slow,
            reheats: args.reheats,
            reheat_factor: args.reheat_factor,
        };
        p.validate()?;
        Ok(p)
    }
}

impl TryFrom<ScoringWeightsArgs> for ScoringWeights {
    type Error = String;
    fn try_from(args: ScoringWeightsArgs) -> Result<Self, Self::Error> {
        let w = Self {
            penalty_sfr_weak_finger: args.penalty_sfr_weak_finger,
            penalty_sfr_bad_row: args.penalty_sfr_bad_row,
            penalty_sfr_lat: args.penalty_sfr_lat,
            penalty_sfb_lateral: args.penalty_sfb_lateral,
            penalty_sfb_lateral_weak: args.penalty_sfb_lateral_weak,
            penalty_sfb_base: args.penalty_sfb_base,
            penalty_sfb_outward_adder: args.penalty_sfb_outward_adder,
            penalty_sfb_diagonal: args.penalty_sfb_diagonal,
            penalty_sfb_long: args.penalty_sfb_long,
            penalty_sfb_bottom: args.penalty_sfb_bottom,
            weight_weak_finger_sfb: args.weight_weak_finger_sfb,
            threshold_sfb_long_row_diff: args.threshold_sfb_long_row_diff,
            threshold_scissor_row_diff: args.threshold_scissor_row_diff,
            threshold_reach_stretch: args.threshold_reach_stretch,
            penalty_scissor: args.penalty_scissor,
            penalty_ring_pinky: args.penalty_ring_pinky,
            penalty_lateral: args.penalty_lateral,
            penalty_monogram_stretch: args.penalty_monogram_stretch,
            penalty_skip: args.penalty_skip,
            penalty_redirect: args.penalty_redirect,
            penalty_hand_run: args.penalty_hand_run,
            bonus_inward_roll: args.bonus_inward_roll,
            bonus_bigram_roll_in: args.bonus_bigram_roll_in,
            bonus_bigram_roll_out: args.bonus_bigram_roll_out,
            penalty_high_in_med: args.penalty_high_in_med,
            penalty_high_in_low: args.penalty_high_in_low,
            penalty_med_in_prime: args.penalty_med_in_prime,
            penalty_med_in_low: args.penalty_med_in_low,
            penalty_low_in_prime: args.penalty_low_in_prime,
            penalty_low_in_med: args.penalty_low_in_med,
            penalty_imbalance: args.penalty_imbalance,
            max_hand_imbalance: args.max_hand_imbalance,
            weight_vertical_travel: args.weight_vertical_travel,
            weight_lateral_travel: args.weight_lateral_travel,
            weight_finger_effort: args.weight_finger_effort,
            default_cost_ms: args.default_cost_ms,
            loader_trigram_limit: args.loader_trigram_limit,
            trigram_coverage: args.trigram_coverage,
            finger_penalty_scale: parse_array_5(&args.finger_penalty_scale)?,
            comfortable_scissors: args.comfortable_scissors,
        };
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
            finger_repeat_scale: parse_array_5(&args.finger_repeat_scale)?,
        })
    }
}

fn parse_array_5(s: &str) -> Result<[f32; 5], String> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 5 {
        return Err(format!("Expected 5 values, got {}", parts.len()));
    }
    let mut arr = [0.0; 5];
    for (i, p) in parts.iter().enumerate() {
        arr[i] = p.trim().parse::<f32>().map_err(|e| e.to_string())?;
    }
    Ok(arr)
}
