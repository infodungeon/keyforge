// libs/keyforge-model/src/config/weights/cli.rs
#![cfg(feature = "cli")]

use super::config::ScoringWeights;
use crate::validator::Validator;

/// CLI arguments mirroring `ScoringWeights`.
#[derive(clap::Args, Debug, Clone)]
pub struct ScoringWeightsConfig {
    /// Generic scoring weights in KEY=VALUE format.
    /// Example: --weight `penalty_sfb_base=500.0` --weight `penalty_lateral=25.0`
    #[arg(long = "weight", value_parser = crate::config::utils::parse_key_val)]
    pub overrides: Vec<(String, f32)>,

    /// Finger penalty multipliers (Thumb, Index, Middle, Ring, Pinky).
    #[arg(long, value_delimiter = ',', num_args = 5, default_values_t = super::constants::DEFAULT_FINGER_PENALTY_SCALE_ARRAY)]
    pub finger_penalty_scale: Vec<f32>,

    /// Comma-separated list of comfortable scissor pairs.
    #[arg(long, default_value = super::constants::DEFAULT_COMFORTABLE_SCISSORS)]
    pub comfortable_scissors: String,
}

use crate::types::FixedWeight;

impl TryFrom<ScoringWeightsConfig> for ScoringWeights {
    type Error = String;
    fn try_from(args: ScoringWeightsConfig) -> Result<Self, Self::Error> {
        let mut w = ScoringWeights::default();

        // Overlay dynamic CLI overrides
        for (key, value) in args.overrides {
            w.weights.insert(key, FixedWeight::from_f32(value)?);
        }

        w.finger_penalty_scale = vec_to_array_5(&args.finger_penalty_scale)?;
        w.comfortable_scissors = args.comfortable_scissors;

        w.validate()?;
        Ok(w)
    }
}

fn vec_to_array_5(v: &[f32]) -> Result<[FixedWeight; 5], String> {
    if v.len() != 5 {
        return Err(format!("Expected 5 values, got {}", v.len()));
    }
    let mut arr = [FixedWeight::default(); 5];
    for (i, &val) in v.iter().enumerate() {
        arr[i] = FixedWeight::from_f32(val)?;
    }
    Ok(arr)
}
