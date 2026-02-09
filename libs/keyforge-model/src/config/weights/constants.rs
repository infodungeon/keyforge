// libs/keyforge-model/src/config/weights/constants.rs
use crate::types::Score;

// --- Default Values (Scoring) ---

/// Maximum safe weight value to prevent overflow.
pub const MAX_SAFE_WEIGHT: Score = Score::from_scaled_i64(100_000_000_000_000);
/// Maximum number of trigrams to load from corpus.
pub const MAX_LOADER_TRIGRAM_LIMIT: usize = 50_000;

/// Default penalty for Same Finger Repeat on a weak finger.
pub const DEFAULT_PENALTY_SFR_WEAK_FINGER: Score = Score::from_scaled_i64(20_000_000);
/// Default penalty for Same Finger Repeat involving a bad row jump.
pub const DEFAULT_PENALTY_SFR_BAD_ROW: Score = Score::from_scaled_i64(25_000_000);
/// Default penalty for lateral Same Finger Repeat.
pub const DEFAULT_PENALTY_SFR_LAT: Score = Score::from_scaled_i64(40_000_000);
/// Default penalty for lateral Same Finger Bigram.
pub const DEFAULT_PENALTY_SFB_LATERAL: Score = Score::from_scaled_i64(65_000_000);
/// Default penalty for lateral SFB on a weak finger.
pub const DEFAULT_PENALTY_SFB_LATERAL_WEAK: Score = Score::from_scaled_i64(160_000_000);
/// Default base penalty for any Same Finger Bigram.
pub const DEFAULT_PENALTY_SFB_BASE: Score = Score::from_scaled_i64(400_000_000);
/// Default additional penalty for outward rolling SFBs.
pub const DEFAULT_PENALTY_SFB_OUTWARD_ADDER: Score = Score::from_scaled_i64(10_000_000);
/// Default penalty for diagonal SFBs.
pub const DEFAULT_PENALTY_SFB_DIAGONAL: Score = Score::from_scaled_i64(240_000_000);
/// Default penalty for long-distance SFBs.
pub const DEFAULT_PENALTY_SFB_LONG: Score = Score::from_scaled_i64(280_000_000);
/// Default penalty for bottom-row SFBs.
pub const DEFAULT_PENALTY_SFB_BOTTOM: Score = Score::from_scaled_i64(45_000_000);
/// Default multiplier for SFBs on weak fingers.
pub const DEFAULT_WEIGHT_WEAK_FINGER_SFB: Score = Score::from_scaled_i64(2_700_000);

/// Default row difference threshold for "long" SFBs.
pub const DEFAULT_THRESHOLD_SFB_LONG_ROW_DIFF: i8 = 2;
/// Default row difference threshold for scissors.
pub const DEFAULT_THRESHOLD_SCISSOR_ROW_DIFF: i8 = 2;
/// Default distance threshold for reach stretches.
pub const DEFAULT_THRESHOLD_REACH_STRETCH: Score = Score::from_scaled_i64(1_200_000);

/// Default penalty for scissor (adjacent finger stretch) movements.
pub const DEFAULT_PENALTY_SCISSOR: Score = Score::from_scaled_i64(25_000_000);
/// Default penalty for ring-pinky interactions.
pub const DEFAULT_PENALTY_RING_PINKY: Score = Score::from_scaled_i64(1_300_000);
/// Default penalty for lateral movement.
pub const DEFAULT_PENALTY_LATERAL: Score = Score::from_scaled_i64(50_000_000);
/// Default penalty for single-key stretches.
pub const DEFAULT_PENALTY_MONOGRAM_STRETCH: Score = Score::from_scaled_i64(20_000_000);
/// Default penalty for skipping a key (hurdle).
pub const DEFAULT_PENALTY_SKIP: Score = Score::from_scaled_i64(20_000_000);
/// Default penalty for redirecting flow (e.g., Left -> Right -> Left).
pub const DEFAULT_PENALTY_REDIRECT: Score = Score::from_scaled_i64(65_000_000);
/// Default penalty for excessive hand alternation runs.
pub const DEFAULT_PENALTY_HAND_RUN: Score = Score::from_scaled_i64(5_000_000);
/// Default bonus (negative cost) for inward rolls.
pub const DEFAULT_BONUS_INWARD_ROLL: Score = Score::from_scaled_i64(40_000_000);
/// Default bonus for specific bigram inward rolls.
pub const DEFAULT_BONUS_BIGRAM_ROLL_IN: Score = Score::from_scaled_i64(35_000_000);
/// Default bonus for specific bigram outward rolls.
pub const DEFAULT_BONUS_BIGRAM_ROLL_OUT: Score = Score::from_scaled_i64(25_000_000);
/// Default penalty for high-frequency keys in medium slots.
pub const DEFAULT_PENALTY_HIGH_IN_MED: Score = Score::from_scaled_i64(12_000_000);
/// Default penalty for high-frequency keys in low slots.
pub const DEFAULT_PENALTY_HIGH_IN_LOW: Score = Score::from_scaled_i64(20_000_000);
/// Default penalty for medium-frequency keys in prime slots.
pub const DEFAULT_PENALTY_MED_IN_PRIME: Score = Score::from_scaled_i64(2_000_000);
/// Default penalty for medium-frequency keys in low slots.
pub const DEFAULT_PENALTY_MED_IN_LOW: Score = Score::from_scaled_i64(2_000_000);
/// Default penalty for low-frequency keys in prime slots.
pub const DEFAULT_PENALTY_LOW_IN_PRIME: Score = Score::from_scaled_i64(15_000_000);
/// Default penalty for low-frequency keys in medium slots.
pub const DEFAULT_PENALTY_LOW_IN_MED: Score = Score::from_scaled_i64(2_000_000);

/// Default penalty for hand imbalance.
pub const DEFAULT_PENALTY_IMBALANCE: Score = Score::from_scaled_i64(200_000_000);
/// Default maximum allowed hand imbalance ratio.
pub const DEFAULT_MAX_HAND_IMBALANCE: Score = Score::from_scaled_i64(550_000);
/// Default weight multiplier for vertical travel distance.
pub const DEFAULT_WEIGHT_VERTICAL_TRAVEL: Score = Score::from_scaled_i64(1_000_000);
/// Default weight multiplier for lateral travel distance.
pub const DEFAULT_WEIGHT_LATERAL_TRAVEL: Score = Score::from_scaled_i64(3_500_000);
/// Default weight multiplier for finger effort.
pub const DEFAULT_WEIGHT_FINGER_EFFORT: Score = Score::from_scaled_i64(2_200_000);
/// Default penalty for keys missing from the cost model.
pub const DEFAULT_PENALTY_MISSING_KEY: Score = Score::from_scaled_i64(100_000_000);

/// Default cost in milliseconds (if using time-based scoring).
pub const DEFAULT_COST_MS: Score = Score::from_scaled_i64(120_000_000);
/// Default limit on the number of trigrams to load.
pub const DEFAULT_LOADER_TRIGRAM_LIMIT: usize = 3000;
/// Default required trigram coverage.
pub const DEFAULT_TRIGRAM_COVERAGE: Score = Score::from_scaled_i64(990_000);

/// Default scale factors for finger penalties as an array (f32 for CLI compatibility).
pub const DEFAULT_FINGER_PENALTY_SCALE_ARRAY_F32: [f32; 5] = [0.0, 1.0, 1.1, 1.3, 1.6];
/// Default scale factors for finger penalties as an array (Score).
pub const DEFAULT_FINGER_PENALTY_SCALE_ARRAY: [Score; 5] = [
    Score::from_scaled_i64(0),
    Score::from_scaled_i64(1_000_000),
    Score::from_scaled_i64(1_100_000),
    Score::from_scaled_i64(1_300_000),
    Score::from_scaled_i64(1_600_000),
];
/// Default comfortable scissor pairs (Indices).
pub const DEFAULT_COMFORTABLE_SCISSORS: &str = "21,23,34";

/// Original f32 constants for Metadata/CLI compatibility.
pub mod f32_compat {
    /// Default SFB Base (f32).
    pub const DEFAULT_PENALTY_SFB_BASE: f32 = 400.0;
    /// Default Scissor Penalty (f32).
    pub const DEFAULT_PENALTY_SCISSOR: f32 = 25.0;
    /// Default Vertical Travel Weight (f32).
    pub const DEFAULT_WEIGHT_VERTICAL_TRAVEL: f32 = 1.0;
}
