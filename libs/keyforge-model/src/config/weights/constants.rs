// libs/keyforge-model/src/config/weights/constants.rs

// --- Default Values (Scoring) ---

/// Maximum safe weight value to prevent overflow.
pub const MAX_SAFE_WEIGHT: f32 = 100_000_000.0;
/// Maximum number of trigrams to load from corpus.
pub const MAX_LOADER_TRIGRAM_LIMIT: usize = 50_000;

/// Default penalty for Same Finger Repeat on a weak finger.
pub const DEFAULT_PENALTY_SFR_WEAK_FINGER: f32 = 20.0;
/// Default penalty for Same Finger Repeat involving a bad row jump.
pub const DEFAULT_PENALTY_SFR_BAD_ROW: f32 = 25.0;
/// Default penalty for lateral Same Finger Repeat.
pub const DEFAULT_PENALTY_SFR_LAT: f32 = 40.0;
/// Default penalty for lateral Same Finger Bigram.
pub const DEFAULT_PENALTY_SFB_LATERAL: f32 = 65.0;
/// Default penalty for lateral SFB on a weak finger.
pub const DEFAULT_PENALTY_SFB_LATERAL_WEAK: f32 = 160.0;
/// Default base penalty for any Same Finger Bigram.
pub const DEFAULT_PENALTY_SFB_BASE: f32 = 400.0;
/// Default additional penalty for outward rolling SFBs.
pub const DEFAULT_PENALTY_SFB_OUTWARD_ADDER: f32 = 10.0;
/// Default penalty for diagonal SFBs.
pub const DEFAULT_PENALTY_SFB_DIAGONAL: f32 = 240.0;
/// Default penalty for long-distance SFBs.
pub const DEFAULT_PENALTY_SFB_LONG: f32 = 280.0;
/// Default penalty for bottom-row SFBs.
pub const DEFAULT_PENALTY_SFB_BOTTOM: f32 = 45.0;
/// Default multiplier for SFBs on weak fingers.
pub const DEFAULT_WEIGHT_WEAK_FINGER_SFB: f32 = 2.7;

/// Default row difference threshold for "long" SFBs.
pub const DEFAULT_THRESHOLD_SFB_LONG_ROW_DIFF: i8 = 2;
/// Default row difference threshold for scissors.
pub const DEFAULT_THRESHOLD_SCISSOR_ROW_DIFF: i8 = 2;
/// Default distance threshold for reach stretches.
pub const DEFAULT_THRESHOLD_REACH_STRETCH: f32 = 1.2;

/// Default penalty for scissor (adjacent finger stretch) movements.
pub const DEFAULT_PENALTY_SCISSOR: f32 = 25.0;
/// Default penalty for ring-pinky interactions.
pub const DEFAULT_PENALTY_RING_PINKY: f32 = 1.3;
/// Default penalty for lateral movement.
pub const DEFAULT_PENALTY_LATERAL: f32 = 50.0;
/// Default penalty for single-key stretches.
pub const DEFAULT_PENALTY_MONOGRAM_STRETCH: f32 = 20.0;
/// Default penalty for skipping a key (hurdle).
pub const DEFAULT_PENALTY_SKIP: f32 = 20.0;
/// Default penalty for redirecting flow (e.g., Left -> Right -> Left).
pub const DEFAULT_PENALTY_REDIRECT: f32 = 65.0;
/// Default penalty for excessive hand alternation runs.
pub const DEFAULT_PENALTY_HAND_RUN: f32 = 5.0;
/// Default bonus (negative cost) for inward rolls.
pub const DEFAULT_BONUS_INWARD_ROLL: f32 = 40.0;
/// Default bonus for specific bigram inward rolls.
pub const DEFAULT_BONUS_BIGRAM_ROLL_IN: f32 = 35.0;
/// Default bonus for specific bigram outward rolls.
pub const DEFAULT_BONUS_BIGRAM_ROLL_OUT: f32 = 25.0;
/// Default penalty for high-frequency keys in medium slots.
pub const DEFAULT_PENALTY_HIGH_IN_MED: f32 = 12.0;
/// Default penalty for high-frequency keys in low slots.
pub const DEFAULT_PENALTY_HIGH_IN_LOW: f32 = 20.0;
/// Default penalty for medium-frequency keys in prime slots.
pub const DEFAULT_PENALTY_MED_IN_PRIME: f32 = 2.0;
/// Default penalty for medium-frequency keys in low slots.
pub const DEFAULT_PENALTY_MED_IN_LOW: f32 = 2.0;
/// Default penalty for low-frequency keys in prime slots.
pub const DEFAULT_PENALTY_LOW_IN_PRIME: f32 = 15.0;
/// Default penalty for low-frequency keys in medium slots.
pub const DEFAULT_PENALTY_LOW_IN_MED: f32 = 2.0;

/// Default penalty for hand imbalance.
pub const DEFAULT_PENALTY_IMBALANCE: f32 = 200.0;
/// Default maximum allowed hand imbalance ratio.
pub const DEFAULT_MAX_HAND_IMBALANCE: f32 = 0.55;
/// Default weight multiplier for vertical travel distance.
pub const DEFAULT_WEIGHT_VERTICAL_TRAVEL: f32 = 1.0;
/// Default weight multiplier for lateral travel distance.
pub const DEFAULT_WEIGHT_LATERAL_TRAVEL: f32 = 3.5;
/// Default weight multiplier for finger effort.
pub const DEFAULT_WEIGHT_FINGER_EFFORT: f32 = 2.2;
/// Default penalty for keys missing from the cost model.
pub const DEFAULT_PENALTY_MISSING_KEY: f32 = 100.0;

/// Default cost in milliseconds (if using time-based scoring).
pub const DEFAULT_COST_MS: f32 = 120.0;
/// Default limit on the number of trigrams to load.
pub const DEFAULT_LOADER_TRIGRAM_LIMIT: usize = 3000;
/// Default required trigram coverage.
pub const DEFAULT_TRIGRAM_COVERAGE: f32 = 0.99;

/// Default scale factors for finger penalties as an array.
pub const DEFAULT_FINGER_PENALTY_SCALE_ARRAY: [f32; 5] = [0.0, 1.0, 1.1, 1.3, 1.6];
/// Default comfortable scissor pairs (Indices).
pub const DEFAULT_COMFORTABLE_SCISSORS: &str = "21,23,34";
