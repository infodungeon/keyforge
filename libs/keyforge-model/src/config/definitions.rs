// libs/keyforge-model/src/config/definitions.rs

// use crate::config::metadata::{ParamType, ParameterMetadata};
use crate::validator::Validator;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// --- Default Values (Strings) ---

/// Default characters considered high priority (Home row candidates).
pub const DEFAULT_TIER_HIGH: &str = "etaoinshr";
/// Default characters considered medium priority.
pub const DEFAULT_TIER_MED: &str = "ldcumwfgypb.,";
/// Default characters considered low priority.
pub const DEFAULT_TIER_LOW: &str = "vkjxqz/;";

/// Default bigrams that must be optimized for.
pub const DEFAULT_CRITICAL_BIGRAMS: &str = "th,he,in,er,an,re,nd,ou";
/// Default scale factors for finger repeat penalties as an array.
pub const DEFAULT_FINGER_REPEAT_SCALE_ARRAY: [f32; 5] = [1.0, 1.0, 1.0, 1.2, 1.5];

/// Definitions for character tiers and critical bigrams.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(default)]
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
        if self.tier_high_chars.is_empty() {
            return Err("tier_high_chars cannot be empty".to_string());
        }

        for (i, &v) in self.finger_repeat_scale.iter().enumerate() {
            if v < 0.0 {
                return Err(format!("finger_repeat_scale[{i}] cannot be negative"));
            }
        }

        Ok(())
    }
}

impl LayoutDefinitions {
    /// Returns a list of critical bigrams (monogram * monogram) for the layout.
    ///
    /// # Errors
    /// Returns `ForgeError::InvalidData` if the layout is incomplete.
    pub fn get_critical_bigrams(&self) -> Result<Vec<(String, f32)>, crate::error::ForgeError> {
        Ok(self
            .critical_bigrams
            .split(',')
            .map(str::trim)
            .filter(|s| s.len() == 2)
            .map(|s| (s.to_string(), 1.0))
            .collect())
    }
}

/// CLI arguments mirroring `LayoutDefinitions`.
#[cfg(feature = "cli")]
#[derive(clap::Args, Debug, Clone)]
pub struct LayoutDefinitionsConfig {
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

#[cfg(feature = "cli")]
impl TryFrom<LayoutDefinitionsConfig> for LayoutDefinitions {
    type Error = String;
    fn try_from(args: LayoutDefinitionsConfig) -> Result<Self, Self::Error> {
        Ok(Self {
            tier_high_chars: args.tier_high_chars,
            tier_med_chars: args.tier_med_chars,
            tier_low_chars: args.tier_low_chars,
            critical_bigrams: args.critical_bigrams,
            finger_repeat_scale: vec_to_array_5(&args.finger_repeat_scale)?,
        })
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
    fn test_layout_definitions_validation() {
        let mut def = LayoutDefinitions::default();
        assert!(def.validate().is_ok());

        // 1. Empty high tier
        def.tier_high_chars = String::new();
        assert!(def.validate().is_err());

        // 2. Negative finger repeat scale
        def.tier_high_chars = "abc".into();
        def.finger_repeat_scale[0] = -1.0;
        assert!(def.validate().is_err());
    }

    #[test]
    fn test_layout_definitions_get_critical_bigrams() {
        let def = LayoutDefinitions {
            critical_bigrams: "th,he,in, invalid, x".into(),
            ..Default::default()
        };
        let bigrams = def.get_critical_bigrams().unwrap();
        assert_eq!(bigrams.len(), 3);
        assert_eq!(bigrams[0], ("th".to_string(), 1.0));
        assert_eq!(bigrams[1], ("he".to_string(), 1.0));
        assert_eq!(bigrams[2], ("in".to_string(), 1.0));
    }

    #[cfg(feature = "cli")]
    #[test]
    fn test_layout_definitions_config_conversion() {
        let config = LayoutDefinitionsConfig {
            tier_high_chars: "abc".into(),
            tier_med_chars: "def".into(),
            tier_low_chars: "ghi".into(),
            critical_bigrams: "ab,cd".into(),
            finger_repeat_scale: vec![1.0, 1.0, 1.0, 1.0, 1.0],
        };
        let def = LayoutDefinitions::try_from(config).unwrap();
        assert_eq!(def.tier_high_chars, "abc");
        assert_eq!(def.finger_repeat_scale, [1.0, 1.0, 1.0, 1.0, 1.0]);
    }
}
