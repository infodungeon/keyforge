// libs/keyforge-model/src/constants/physics.rs

/// Scaling factor for fixed-point score arithmetic.
pub const SCORE_SCALE: i64 = 1_000_000;
/// Scaling factor for fixed-point weight arithmetic.
pub const WEIGHT_SCALE: i32 = 1_000;
/// Tolerance ratio for score verification.
pub const VERIFICATION_TOLERANCE_RATIO: f32 = 0.0001;
/// Minimum absolute tolerance for score verification.
pub const VERIFICATION_TOLERANCE_ABS_MIN: f32 = 1.0;

/// Maximum score considered plausible for a standard layout (sanity check).
pub const MAX_PLAUSIBLE_SCORE: f32 = 10_000_000.0;
/// Maximum SFB ratio considered plausible (sanity check, 20%).
pub const MAX_PLAUSIBLE_SFB_RATIO: f32 = 0.20;

/// Number of top frequent words to consider for Arena typing practice.
pub const ARENA_TOP_WORDS_LIMIT: usize = 2000;

/// Assumed error rate for standard prose (3%).
pub const STD_CORPUS_ERROR_RATE: f32 = 0.03;
/// Correction factor for backspace usage (1.25x error rate).
pub const STD_CORPUS_BACKSPACE_FACTOR: f32 = 1.25;
/// Average sentences per paragraph for standard prose.
pub const STD_CORPUS_SENTENCE_RATIO: f32 = 3.0;

/// Minimum temperature threshold before clipping to zero.
pub const TEMP_UNDERFLOW_THRESHOLD: f32 = 1e-10;
/// Default divisor for determining reporting interval (steps / divisor).
pub const DEFAULT_REPORT_DIVISOR: usize = 100;
/// Minimum number of steps between progress reports.
pub const MIN_REPORT_INTERVAL: usize = 1000;
/// Minimum temperature threshold for accepting worsening moves.
pub const ANNEALING_MIN_TEMP: f32 = 1e-6;

/// Default weight for a corpus source.
pub const DEFAULT_CORPUS_WEIGHT: f32 = 1.0;
