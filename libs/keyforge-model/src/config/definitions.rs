// libs/keyforge-model/src/config/definitions.rs

// use crate::config::metadata::{ParamType, ParameterMetadata};
use crate::validator::Validator;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

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
