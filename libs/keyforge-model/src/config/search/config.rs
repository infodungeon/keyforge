// libs/keyforge-model/src/config/search/config.rs
use crate::error::ForgeError;

/// Configuration for the optimization search strategy.
#[derive(Debug, Clone)]
pub enum SearchConfig {
    /// Simulated Annealing strategy.
    Annealing {
        /// Total number of mutation steps.
        steps: usize,
        /// Initial temperature (higher = more chaotic).
        start_temp: f32,
        /// Final temperature (lower = more greedy).
        end_temp: f32,
        /// PRNG seed for deterministic replay.
        seed: u64,
        /// Steps without improvement before reheating.
        patience: usize,
        /// Number of times to reheat.
        reheats: usize,
        /// Multiplier for `start_temp` when reheating.
        reheat_factor: f32,
        /// Whether to include thumb keys in swap suggestions.
        include_thumbs: bool,
    },
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self::Annealing {
            steps: 100_000,
            start_temp: 100.0,
            end_temp: 0.01,
            seed: 42,
            patience: 500,
            reheats: 3,
            reheat_factor: 0.5,
            include_thumbs: false,
        }
    }
}

impl SearchConfig {
    /// Validates that configuration parameters are within safe bounds.
    ///
    /// # Errors
    /// Returns a `ForgeError` if the parameters are out of reasonable bounds.
    pub fn validate(&self) -> Result<(), ForgeError> {
        match self {
            SearchConfig::Annealing {
                steps,
                start_temp,
                end_temp,
                reheat_factor,
                ..
            } => {
                if *steps == 0 {
                    return Err(ForgeError::InvalidData("Steps must be > 0".into()));
                }
                if *start_temp < 0.0 {
                    return Err(ForgeError::InvalidData("Start temp must be >= 0".into()));
                }
                if *end_temp < 0.0 {
                    return Err(ForgeError::InvalidData("End temp must be >= 0".into()));
                }
                if *reheat_factor <= 0.0 {
                    return Err(ForgeError::InvalidData("Reheat factor must be > 0".into()));
                }
            }
        }
        Ok(())
    }

    /// Returns whether thumb keys should be included in swap suggestions.
    #[must_use]
    pub fn include_thumbs(&self) -> bool {
        match self {
            SearchConfig::Annealing { include_thumbs, .. } => *include_thumbs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_config_validation() {
        let c = SearchConfig::default();
        assert!(c.validate().is_ok());

        let invalid_steps = SearchConfig::Annealing {
            steps: 0,
            start_temp: 100.0,
            end_temp: 0.01,
            seed: 42,
            patience: 500,
            reheats: 3,
            reheat_factor: 0.5,
            include_thumbs: false,
        };
        assert!(invalid_steps.validate().is_err());

        let invalid_temp = SearchConfig::Annealing {
            steps: 100,
            start_temp: -1.0,
            end_temp: 0.01,
            seed: 42,
            patience: 500,
            reheats: 3,
            reheat_factor: 0.5,
            include_thumbs: false,
        };
        assert!(invalid_temp.validate().is_err());
    }

    #[test]
    fn test_search_config_validation_extended() {
        // end_temp < 0
        let c = SearchConfig::Annealing {
            steps: 100,
            start_temp: 10.0,
            end_temp: -1.0,
            seed: 0,
            patience: 10,
            reheats: 0,
            reheat_factor: 0.5,
            include_thumbs: false,
        };
        assert!(c.validate().is_err());

        // reheat_factor <= 0
        let c = SearchConfig::Annealing {
            steps: 100,
            start_temp: 10.0,
            end_temp: 0.1,
            seed: 0,
            patience: 10,
            reheats: 1,
            reheat_factor: 0.0,
            include_thumbs: false,
        };
        assert!(c.validate().is_err());

        assert!(!SearchConfig::default().include_thumbs());
        assert!(SearchConfig::Annealing {
            steps: 100,
            start_temp: 10.0,
            end_temp: 0.1,
            seed: 0,
            patience: 10,
            reheats: 1,
            reheat_factor: 0.5,
            include_thumbs: true,
        }
        .include_thumbs());
    }
}
