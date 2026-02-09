// libs/keyforge-model/src/config/search/params.rs
use super::constants::{
    DEFAULT_OPT_LIMIT_FAST, DEFAULT_OPT_LIMIT_SLOW, DEFAULT_REHEATS, DEFAULT_REHEAT_FACTOR,
    DEFAULT_SEARCH_EPOCHS, DEFAULT_SEARCH_PATIENCE, DEFAULT_SEARCH_PATIENCE_THRESHOLD,
    DEFAULT_SEARCH_STEPS, DEFAULT_TEMP_MAX, DEFAULT_TEMP_MIN, MAX_OPT_LIMIT_FAST,
    MAX_SEARCH_EPOCHS, MAX_SEARCH_STEPS, MAX_TEMP,
};
use crate::config::metadata::{ParamType, ParameterMetadata};
use crate::validator::Validator;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Parameters controlling the Simulated Annealing algorithm.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SearchParams {
    /// Dynamic parameters map.
    pub params: std::collections::HashMap<String, f32>,
    /// Random seed for deterministic replay (Optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
    /// Whether to include thumb keys in swap suggestions.
    #[serde(default = "default_false")]
    pub include_thumbs: bool,
}

fn default_false() -> bool {
    false
}

impl Default for SearchParams {
    fn default() -> Self {
        let mut params = std::collections::HashMap::new();

        // Use temporary variables to avoid 'as' casts in the map insertion
        let epochs_f32 = f32::from(u16::try_from(DEFAULT_SEARCH_EPOCHS).unwrap_or(0));
        let steps_f32 = f32::from(u16::try_from(DEFAULT_SEARCH_STEPS / 10).unwrap_or(0)) * 10.0;
        let patience_f32 = f32::from(u16::try_from(DEFAULT_SEARCH_PATIENCE).unwrap_or(0));
        let opt_fast_f32 = f32::from(u16::try_from(DEFAULT_OPT_LIMIT_FAST).unwrap_or(0));
        let opt_slow_f32 = f32::from(u16::try_from(DEFAULT_OPT_LIMIT_SLOW).unwrap_or(0));
        let reheats_f32 = f32::from(u16::try_from(DEFAULT_REHEATS).unwrap_or(0));

        params.insert("search_epochs".to_string(), epochs_f32);
        params.insert("search_steps".to_string(), steps_f32);
        params.insert("search_patience".to_string(), patience_f32);
        params.insert(
            "search_patience_threshold".to_string(),
            DEFAULT_SEARCH_PATIENCE_THRESHOLD,
        );
        params.insert("temp_min".to_string(), DEFAULT_TEMP_MIN);
        params.insert("temp_max".to_string(), DEFAULT_TEMP_MAX);
        params.insert("opt_limit_fast".to_string(), opt_fast_f32);
        params.insert("opt_limit_slow".to_string(), opt_slow_f32);
        params.insert("reheats".to_string(), reheats_f32);
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
        if self.get_search_epochs() == 0 {
            return Err("search_epochs must be > 0".into());
        }
        if self.get_search_epochs() > MAX_SEARCH_EPOCHS {
            return Err(format!("search_epochs exceeds limit ({MAX_SEARCH_EPOCHS})"));
        }
        if self.get_search_steps() == 0 {
            return Err("search_steps must be > 0".into());
        }
        if self.get_search_steps() > MAX_SEARCH_STEPS {
            return Err(format!("search_steps exceeds limit ({MAX_SEARCH_STEPS})"));
        }
        if self.get_opt_limit_fast() == 0 {
            return Err("opt_limit_fast must be > 0".into());
        }
        if self.get_opt_limit_fast() > MAX_OPT_LIMIT_FAST {
            return Err(format!(
                "opt_limit_fast exceeds limit ({MAX_OPT_LIMIT_FAST})"
            ));
        }
        if self.get_opt_limit_slow() < self.get_opt_limit_fast() {
            return Err("opt_limit_slow must be >= opt_limit_fast".into());
        }
        if self.get_temp_min() < 0.0 || self.get_temp_max() < 0.0 {
            return Err("Temperature cannot be negative".into());
        }
        if self.get_temp_max() > MAX_TEMP {
            return Err(format!("temp_max exceeds limit ({MAX_TEMP})"));
        }
        if self.get_temp_min() < 0.0001 {
            return Err("temp_min too low (underflow risk)".into());
        }
        if self.get_temp_min() >= self.get_temp_max() {
            return Err("temp_min must be < temp_max".into());
        }
        if self.get_search_patience_threshold() < 0.0 || self.get_search_patience_threshold() > 1.0
        {
            return Err("search_patience_threshold must be between 0.0 and 1.0".into());
        }
        Ok(())
    }
}

impl SearchParams {
    /// Returns the schema for search parameters.
    #[must_use]
    pub fn schema() -> Vec<ParameterMetadata> {
        let epochs_max = f32::from(u16::try_from(MAX_SEARCH_EPOCHS / 100).unwrap_or(0)) * 100.0;
        let epochs_def = f32::from(u16::try_from(DEFAULT_SEARCH_EPOCHS).unwrap_or(0));
        let steps_max = f32::from(u16::try_from(MAX_SEARCH_STEPS / 1000).unwrap_or(0)) * 1000.0;
        let steps_def = f32::from(u16::try_from(DEFAULT_SEARCH_STEPS / 10).unwrap_or(0)) * 10.0;
        let patience_def = f32::from(u16::try_from(DEFAULT_SEARCH_PATIENCE).unwrap_or(0));
        let reheats_def = f32::from(u16::try_from(DEFAULT_REHEATS).unwrap_or(0));

        vec![
            ParameterMetadata {
                key: "search_epochs".to_string(),
                label: "Search Epochs".to_string(),
                description: "Number of independent search runs to perform.".to_string(),
                param_type: ParamType::Integer,
                min: Some(1.0),
                max: Some(epochs_max),
                default: epochs_def,
            },
            ParameterMetadata {
                key: "search_steps".to_string(),
                label: "Steps per Epoch".to_string(),
                description: "Maximum mutations to attempt per epoch.".to_string(),
                param_type: ParamType::Integer,
                min: Some(1000.0),
                max: Some(steps_max),
                default: steps_def,
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
                default: patience_def,
            },
            ParameterMetadata {
                key: "reheats".to_string(),
                label: "Reheats".to_string(),
                description: "Number of times to spike temperature when stuck.".to_string(),
                param_type: ParamType::Integer,
                min: Some(0.0),
                max: Some(10.0),
                default: reheats_def,
            },
        ]
    }

    /// Retrieves a parameter by key, falling back to a default value if not found.
    #[must_use]
    pub fn get_param(&self, key: &str, default: f32) -> f32 {
        self.params.get(key).copied().unwrap_or(default)
    }

    /// Number of independent search epochs to run.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn get_search_epochs(&self) -> usize {
        let epochs_def = f32::from(u16::try_from(DEFAULT_SEARCH_EPOCHS).unwrap_or(0));
        let val = self.get_param("search_epochs", epochs_def);
        let score = crate::types::Score::from_f32(val).unwrap_or_default();
        usize::try_from(score.raw() / 1_000_000).unwrap_or(0)
    }
    /// Maximum number of mutation steps per epoch.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn get_search_steps(&self) -> usize {
        let steps_def = f32::from(u16::try_from(DEFAULT_SEARCH_STEPS / 10).unwrap_or(0)) * 10.0;
        let val = self.get_param("search_steps", steps_def);
        let score = crate::types::Score::from_f32(val).unwrap_or_default();
        usize::try_from(score.raw() / 1_000_000).unwrap_or(0)
    }
    /// Steps without improvement before triggering a reheat.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn get_search_patience(&self) -> usize {
        let patience_def = f32::from(u16::try_from(DEFAULT_SEARCH_PATIENCE).unwrap_or(0));
        let val = self.get_param("search_patience", patience_def);
        let score = crate::types::Score::from_f32(val).unwrap_or_default();
        usize::try_from(score.raw() / 1_000_000).unwrap_or(0)
    }
    /// Threshold for patience reset.
    #[must_use]
    pub fn get_search_patience_threshold(&self) -> f32 {
        self.get_param(
            "search_patience_threshold",
            DEFAULT_SEARCH_PATIENCE_THRESHOLD,
        )
    }
    /// Minimum temperature.
    #[must_use]
    pub fn get_temp_min(&self) -> f32 {
        self.get_param("temp_min", DEFAULT_TEMP_MIN)
    }
    /// Maximum temperature.
    #[must_use]
    pub fn get_temp_max(&self) -> f32 {
        self.get_param("temp_max", DEFAULT_TEMP_MAX)
    }
    /// Optimization limit for fast path.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn get_opt_limit_fast(&self) -> usize {
        let opt_def = f32::from(u16::try_from(DEFAULT_OPT_LIMIT_FAST).unwrap_or(0));
        let val = self.get_param("opt_limit_fast", opt_def);
        let score = crate::types::Score::from_f32(val).unwrap_or_default();
        usize::try_from(score.raw() / 1_000_000).unwrap_or(0)
    }
    /// Optimization limit for slow path.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn get_opt_limit_slow(&self) -> usize {
        let opt_def = f32::from(u16::try_from(DEFAULT_OPT_LIMIT_SLOW).unwrap_or(0));
        let val = self.get_param("opt_limit_slow", opt_def);
        let score = crate::types::Score::from_f32(val).unwrap_or_default();
        usize::try_from(score.raw() / 1_000_000).unwrap_or(0)
    }
    /// Number of times to reheat.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn get_reheats(&self) -> usize {
        let reheats_def = f32::from(u16::try_from(DEFAULT_REHEATS).unwrap_or(0));
        let val = self.get_param("reheats", reheats_def);
        let score = crate::types::Score::from_f32(val).unwrap_or_default();
        usize::try_from(score.raw() / 1_000_000).unwrap_or(0)
    }
    /// Factor to multiply temperature by when reheating.
    #[must_use]
    pub fn get_reheat_factor(&self) -> f32 {
        self.get_param("reheat_factor", DEFAULT_REHEAT_FACTOR)
    }
}

/// CLI arguments mirroring `SearchParams`.
#[cfg(feature = "cli")]
#[derive(clap::Args, Debug, Clone)]
pub struct SearchParamsConfig {
    /// Generic search parameters in KEY=VALUE format.
    /// Example: --search `temp_max=5.0` --search `search_epochs=50000`
    #[arg(long = "search", value_parser = crate::config::utils::parse_key_val)]
    pub params: Vec<(String, f32)>,

    /// Whether to include thumb keys in swap suggestions.
    #[arg(long)]
    pub include_thumbs: bool,
}

#[cfg(feature = "cli")]
impl TryFrom<SearchParamsConfig> for SearchParams {
    type Error = String;
    fn try_from(args: SearchParamsConfig) -> Result<Self, Self::Error> {
        let mut p = SearchParams::default();

        // Dynamic overrides from CLI
        for (key, value) in args.params {
            p.params.insert(key, value);
        }

        p.seed = None; // Seed handled by runner options
        p.include_thumbs = args.include_thumbs;

        p.validate()?;
        Ok(p)
    }
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;

    #[test]
    fn test_search_params_getters() -> anyhow::Result<()> {
        let p = SearchParams::default();
        assert_eq!(p.get_search_epochs(), DEFAULT_SEARCH_EPOCHS);
        assert_eq!(p.get_search_steps(), DEFAULT_SEARCH_STEPS);
        assert_eq!(p.get_search_patience(), DEFAULT_SEARCH_PATIENCE);
        assert_eq!(
            p.get_search_patience_threshold(),
            DEFAULT_SEARCH_PATIENCE_THRESHOLD
        );
        assert_eq!(p.get_temp_min(), DEFAULT_TEMP_MIN);
        assert_eq!(p.get_temp_max(), DEFAULT_TEMP_MAX);
        assert_eq!(p.get_opt_limit_fast(), DEFAULT_OPT_LIMIT_FAST);
        assert_eq!(p.get_opt_limit_slow(), DEFAULT_OPT_LIMIT_SLOW);
        assert_eq!(p.get_reheats(), DEFAULT_REHEATS);
        assert_eq!(p.get_reheat_factor(), DEFAULT_REHEAT_FACTOR);

        // Test empty params fallback
        let empty = SearchParams {
            params: std::collections::HashMap::new(),
            seed: None,
            include_thumbs: false,
        };
        assert_eq!(empty.get_search_epochs(), DEFAULT_SEARCH_EPOCHS);
        assert_eq!(empty.get_search_steps(), DEFAULT_SEARCH_STEPS);
        assert_eq!(empty.get_opt_limit_fast(), DEFAULT_OPT_LIMIT_FAST);
        assert_eq!(empty.get_reheats(), DEFAULT_REHEATS);
        Ok(())
    }

    #[test]
    fn test_search_params_validation_extended() -> anyhow::Result<()> {
        let mut p = SearchParams::default();

        // epochs exceeds limit
        let epochs_max_plus =
            f32::from(u16::try_from(MAX_SEARCH_EPOCHS / 1000).unwrap_or(0)) * 1000.0 + 1.0;
        p.params.insert("search_epochs".into(), epochs_max_plus);
        assert!(p.validate().is_err());

        // steps exceeds limit
        p = SearchParams::default();
        let steps_max_plus =
            f32::from(u16::try_from(MAX_SEARCH_STEPS / 1000).unwrap_or(0)) * 1000.0 + 1.0;
        p.params.insert("search_steps".into(), steps_max_plus);
        assert!(p.validate().is_err());

        // opt_limit_fast 0
        p = SearchParams::default();
        p.params.insert("opt_limit_fast".into(), 0.0);
        assert!(p.validate().is_err());

        // opt_limit_fast exceeds limit
        p = SearchParams::default();
        let opt_max_plus = f32::from(u16::try_from(MAX_OPT_LIMIT_FAST).unwrap_or(0)) + 1.0;
        p.params.insert("opt_limit_fast".into(), opt_max_plus);
        assert!(p.validate().is_err());

        // opt_limit_slow < fast
        p = SearchParams::default();
        p.params.insert("opt_limit_fast".into(), 1000.0);
        p.params.insert("opt_limit_slow".into(), 500.0);
        assert!(p.validate().is_err());

        // temp_max exceeds limit
        p = SearchParams::default();
        p.params.insert("temp_max".into(), MAX_TEMP + 1.0);
        assert!(p.validate().is_err());

        // temp_min too low
        p = SearchParams::default();
        p.params.insert("temp_min".into(), 0.00001);
        assert!(p.validate().is_err());

        // temp_min >= temp_max
        p = SearchParams::default();
        p.params.insert("temp_min".into(), 10.0);
        p.params.insert("temp_max".into(), 5.0);
        assert!(p.validate().is_err());
        Ok(())
    }

    #[test]
    fn test_search_params_schema() -> anyhow::Result<()> {
        let schema = SearchParams::schema();
        assert!(!schema.is_empty());
        assert!(schema.iter().any(|m| m.key == "search_epochs"));
        Ok(())
    }

    #[cfg(feature = "cli")]
    #[test]
    fn test_search_params_config_conversion() -> anyhow::Result<()> {
        let config = SearchParamsConfig {
            params: vec![("temp_max".to_string(), 50.0)],
            include_thumbs: true,
        };
        let p = SearchParams::try_from(config).map_err(|e: String| anyhow::anyhow!(e))?;
        assert_eq!(p.get_temp_max(), 50.0);
        assert!(p.include_thumbs);
        Ok(())
    }

    #[test]
    fn test_search_params_validation_errors() -> anyhow::Result<()> {
        let mut p = SearchParams::default();

        p.params.insert("search_epochs".into(), 0.0);
        assert!(p.validate().is_err());
        p.params.insert("search_epochs".into(), 10.0);

        p.params.insert("search_steps".into(), 0.0);
        assert!(p.validate().is_err());
        p.params.insert("search_steps".into(), 100.0);

        p.params.insert("temp_max".into(), -1.0);
        assert!(p.validate().is_err());
        p.params.insert("temp_max".into(), 100.0);

        p.params.insert("search_patience_threshold".into(), 2.0);
        assert!(p.validate().is_err());
        Ok(())
    }
}
