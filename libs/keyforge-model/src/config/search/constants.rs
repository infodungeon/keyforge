// libs/keyforge-model/src/config/search/constants.rs

/// Maximum allowed temperature for annealing.
pub const MAX_TEMP: f32 = 1_000.0;
/// Maximum number of search epochs.
pub const MAX_SEARCH_EPOCHS: usize = 1_000_000;
/// Maximum number of search steps per epoch.
pub const MAX_SEARCH_STEPS: usize = 5_000_000;
/// Maximum optimization limit for fast path.
pub const MAX_OPT_LIMIT_FAST: usize = 10_000;

/// Default number of search epochs.
pub const DEFAULT_SEARCH_EPOCHS: usize = 10_000;
/// Default number of search steps per epoch.
pub const DEFAULT_SEARCH_STEPS: usize = 100_000;
/// Default search patience.
pub const DEFAULT_SEARCH_PATIENCE: usize = 500;
/// Default search patience threshold.
pub const DEFAULT_SEARCH_PATIENCE_THRESHOLD: f32 = 0.1;
/// Default minimum temperature.
pub const DEFAULT_TEMP_MIN: f32 = 0.005;
/// Default maximum temperature.
pub const DEFAULT_TEMP_MAX: f32 = 20.0;
/// Default fast-path optimization limit.
pub const DEFAULT_OPT_LIMIT_FAST: usize = 100;
/// Default slow-path optimization limit.
pub const DEFAULT_OPT_LIMIT_SLOW: usize = 1500;
/// Default number of reheats.
pub const DEFAULT_REHEATS: usize = 3;
/// Default reheat factor.
pub const DEFAULT_REHEAT_FACTOR: f32 = 0.5;
