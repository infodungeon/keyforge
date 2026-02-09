// libs/keyforge-model/src/config/weights/accessors.rs
use super::config::ScoringWeights;
use super::constants::{
    DEFAULT_BONUS_BIGRAM_ROLL_IN, DEFAULT_BONUS_BIGRAM_ROLL_OUT, DEFAULT_BONUS_INWARD_ROLL,
    DEFAULT_COST_MS, DEFAULT_LOADER_TRIGRAM_LIMIT, DEFAULT_MAX_HAND_IMBALANCE,
    DEFAULT_PENALTY_HAND_RUN, DEFAULT_PENALTY_HIGH_IN_LOW, DEFAULT_PENALTY_HIGH_IN_MED,
    DEFAULT_PENALTY_IMBALANCE, DEFAULT_PENALTY_LATERAL, DEFAULT_PENALTY_LOW_IN_MED,
    DEFAULT_PENALTY_LOW_IN_PRIME, DEFAULT_PENALTY_MED_IN_LOW, DEFAULT_PENALTY_MED_IN_PRIME,
    DEFAULT_PENALTY_MONOGRAM_STRETCH, DEFAULT_PENALTY_REDIRECT, DEFAULT_PENALTY_RING_PINKY,
    DEFAULT_PENALTY_SCISSOR, DEFAULT_PENALTY_SFB_BASE, DEFAULT_PENALTY_SFB_BOTTOM,
    DEFAULT_PENALTY_SFB_DIAGONAL, DEFAULT_PENALTY_SFB_LATERAL, DEFAULT_PENALTY_SFB_LATERAL_WEAK,
    DEFAULT_PENALTY_SFB_LONG, DEFAULT_PENALTY_SFB_OUTWARD_ADDER, DEFAULT_PENALTY_SFR_BAD_ROW,
    DEFAULT_PENALTY_SFR_LAT, DEFAULT_PENALTY_SFR_WEAK_FINGER, DEFAULT_PENALTY_SKIP,
    DEFAULT_THRESHOLD_REACH_STRETCH, DEFAULT_THRESHOLD_SCISSOR_ROW_DIFF,
    DEFAULT_THRESHOLD_SFB_LONG_ROW_DIFF, DEFAULT_TRIGRAM_COVERAGE, DEFAULT_WEIGHT_FINGER_EFFORT,
    DEFAULT_WEIGHT_LATERAL_TRAVEL, DEFAULT_WEIGHT_VERTICAL_TRAVEL, DEFAULT_WEIGHT_WEAK_FINGER_SFB,
};
use crate::types::Score;

impl ScoringWeights {
    /// Retrieves a weight by key, falling back to a default value if not found.
    #[must_use]
    pub fn get_weight(&self, key: &str, default: Score) -> Score {
        self.weights.get(key).copied().unwrap_or(default)
    }

    /// Gets the penalty for Same Finger Repeat on a weak finger.
    #[must_use]
    pub fn get_penalty_sfr_weak_finger(&self) -> Score {
        self.get_weight("penalty_sfr_weak_finger", DEFAULT_PENALTY_SFR_WEAK_FINGER)
    }
    /// Gets the penalty for Same Finger Repeat involving a bad row jump.
    #[must_use]
    pub fn get_penalty_sfr_bad_row(&self) -> Score {
        self.get_weight("penalty_sfr_bad_row", DEFAULT_PENALTY_SFR_BAD_ROW)
    }
    /// Gets the penalty for lateral Same Finger Repeat.
    #[must_use]
    pub fn get_penalty_sfr_lat(&self) -> Score {
        self.get_weight("penalty_sfr_lat", DEFAULT_PENALTY_SFR_LAT)
    }
    /// Gets the penalty for lateral Same Finger Bigram.
    #[must_use]
    pub fn get_penalty_sfb_lateral(&self) -> Score {
        self.get_weight("penalty_sfb_lateral", DEFAULT_PENALTY_SFB_LATERAL)
    }
    /// Gets the penalty for lateral SFB on a weak finger.
    #[must_use]
    pub fn get_penalty_sfb_lateral_weak(&self) -> Score {
        self.get_weight("penalty_sfb_lateral_weak", DEFAULT_PENALTY_SFB_LATERAL_WEAK)
    }
    /// Gets the base penalty for any Same Finger Bigram.
    #[must_use]
    pub fn get_penalty_sfb_base(&self) -> Score {
        self.get_weight("penalty_sfb_base", DEFAULT_PENALTY_SFB_BASE)
    }
    /// Gets the additional penalty for outward rolling SFBs.
    #[must_use]
    pub fn get_penalty_sfb_outward_adder(&self) -> Score {
        self.get_weight(
            "penalty_sfb_outward_adder",
            DEFAULT_PENALTY_SFB_OUTWARD_ADDER,
        )
    }
    /// Gets the penalty for diagonal SFBs.
    #[must_use]
    pub fn get_penalty_sfb_diagonal(&self) -> Score {
        self.get_weight("penalty_sfb_diagonal", DEFAULT_PENALTY_SFB_DIAGONAL)
    }
    /// Gets the penalty for long-distance SFBs.
    #[must_use]
    pub fn get_penalty_sfb_long(&self) -> Score {
        self.get_weight("penalty_sfb_long", DEFAULT_PENALTY_SFB_LONG)
    }
    /// Gets the penalty for bottom-row SFBs.
    #[must_use]
    pub fn get_penalty_sfb_bottom(&self) -> Score {
        self.get_weight("penalty_sfb_bottom", DEFAULT_PENALTY_SFB_BOTTOM)
    }
    /// Gets the multiplier for SFBs on weak fingers.
    #[must_use]
    pub fn get_weight_weak_finger_sfb(&self) -> Score {
        self.get_weight("weight_weak_finger_sfb", DEFAULT_WEIGHT_WEAK_FINGER_SFB)
    }

    /// Gets the row difference threshold for "long" SFBs.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn get_threshold_sfb_long_row_diff(&self) -> i8 {
        let val = self.get_weight(
            "threshold_sfb_long_row_diff",
            Score::from_scaled_i64(i64::from(DEFAULT_THRESHOLD_SFB_LONG_ROW_DIFF) * 1_000_000),
        );
        i8::try_from(val.raw() / 1_000_000).unwrap_or(0)
    }
    /// Gets the row difference threshold for scissors.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn get_threshold_scissor_row_diff(&self) -> i8 {
        let val = self.get_weight(
            "threshold_scissor_row_diff",
            Score::from_scaled_i64(i64::from(DEFAULT_THRESHOLD_SCISSOR_ROW_DIFF) * 1_000_000),
        );
        i8::try_from(val.raw() / 1_000_000).unwrap_or(0)
    }
    /// Gets the distance threshold for reach stretches.
    #[must_use]
    pub fn get_threshold_reach_stretch(&self) -> Score {
        self.get_weight("threshold_reach_stretch", DEFAULT_THRESHOLD_REACH_STRETCH)
    }

    /// Gets the penalty for scissor (adjacent finger stretch) movements.
    #[must_use]
    pub fn get_penalty_scissor(&self) -> Score {
        self.get_weight("penalty_scissor", DEFAULT_PENALTY_SCISSOR)
    }
    /// Gets the penalty for ring-pinky interactions.
    #[must_use]
    pub fn get_penalty_ring_pinky(&self) -> Score {
        self.get_weight("penalty_ring_pinky", DEFAULT_PENALTY_RING_PINKY)
    }
    /// Gets the penalty for lateral movement.
    #[must_use]
    pub fn get_penalty_lateral(&self) -> Score {
        self.get_weight("penalty_lateral", DEFAULT_PENALTY_LATERAL)
    }
    /// Gets the penalty for single-key stretches.
    #[must_use]
    pub fn get_penalty_monogram_stretch(&self) -> Score {
        self.get_weight("penalty_monogram_stretch", DEFAULT_PENALTY_MONOGRAM_STRETCH)
    }
    /// Gets the penalty for skipping a key (hurdle).
    #[must_use]
    pub fn get_penalty_skip(&self) -> Score {
        self.get_weight("penalty_skip", DEFAULT_PENALTY_SKIP)
    }
    /// Gets the penalty for redirecting flow (e.g., Left -> Right -> Left).
    #[must_use]
    pub fn get_penalty_redirect(&self) -> Score {
        self.get_weight("penalty_redirect", DEFAULT_PENALTY_REDIRECT)
    }
    /// Gets the penalty for excessive hand alternation runs.
    #[must_use]
    pub fn get_penalty_hand_run(&self) -> Score {
        self.get_weight("penalty_hand_run", DEFAULT_PENALTY_HAND_RUN)
    }
    /// Gets the bonus (negative cost) for inward rolls.
    #[must_use]
    pub fn get_bonus_inward_roll(&self) -> Score {
        self.get_weight("bonus_inward_roll", DEFAULT_BONUS_INWARD_ROLL)
    }
    /// Gets the bonus for specific bigram inward rolls.
    #[must_use]
    pub fn get_bonus_bigram_roll_in(&self) -> Score {
        self.get_weight("bonus_bigram_roll_in", DEFAULT_BONUS_BIGRAM_ROLL_IN)
    }
    /// Gets the bonus for specific bigram outward rolls.
    #[must_use]
    pub fn get_bonus_bigram_roll_out(&self) -> Score {
        self.get_weight("bonus_bigram_roll_out", DEFAULT_BONUS_BIGRAM_ROLL_OUT)
    }

    /// Gets the penalty for high-frequency keys in medium slots.
    #[must_use]
    pub fn get_penalty_high_in_med(&self) -> Score {
        self.get_weight("penalty_high_in_med", DEFAULT_PENALTY_HIGH_IN_MED)
    }
    /// Gets the penalty for high-frequency keys in low slots.
    #[must_use]
    pub fn get_penalty_high_in_low(&self) -> Score {
        self.get_weight("penalty_high_in_low", DEFAULT_PENALTY_HIGH_IN_LOW)
    }
    /// Gets the penalty for medium-frequency keys in prime slots.
    #[must_use]
    pub fn get_penalty_med_in_prime(&self) -> Score {
        self.get_weight("penalty_med_in_prime", DEFAULT_PENALTY_MED_IN_PRIME)
    }
    /// Gets the penalty for medium-frequency keys in low slots.
    #[must_use]
    pub fn get_penalty_med_in_low(&self) -> Score {
        self.get_weight("penalty_med_in_low", DEFAULT_PENALTY_MED_IN_LOW)
    }
    /// Gets the penalty for low-frequency keys in prime slots.
    #[must_use]
    pub fn get_penalty_low_in_prime(&self) -> Score {
        self.get_weight("penalty_low_in_prime", DEFAULT_PENALTY_LOW_IN_PRIME)
    }
    /// Gets the penalty for low-frequency keys in medium slots.
    #[must_use]
    pub fn get_penalty_low_in_med(&self) -> Score {
        self.get_weight("penalty_low_in_med", DEFAULT_PENALTY_LOW_IN_MED)
    }

    /// Gets the penalty for hand imbalance.
    #[must_use]
    pub fn get_penalty_imbalance(&self) -> Score {
        self.get_weight("penalty_imbalance", DEFAULT_PENALTY_IMBALANCE)
    }
    /// Gets the maximum allowed hand imbalance ratio.
    #[must_use]
    pub fn get_max_hand_imbalance(&self) -> Score {
        self.get_weight("max_hand_imbalance", DEFAULT_MAX_HAND_IMBALANCE)
    }

    /// Gets the weight multiplier for vertical travel distance.
    #[must_use]
    pub fn get_weight_vertical_travel(&self) -> Score {
        self.get_weight("weight_vertical_travel", DEFAULT_WEIGHT_VERTICAL_TRAVEL)
    }
    /// Gets the weight multiplier for lateral travel distance.
    #[must_use]
    pub fn get_weight_lateral_travel(&self) -> Score {
        self.get_weight("weight_lateral_travel", DEFAULT_WEIGHT_LATERAL_TRAVEL)
    }
    /// Gets the weight multiplier for finger effort.
    #[must_use]
    pub fn get_weight_finger_effort(&self) -> Score {
        self.get_weight("weight_finger_effort", DEFAULT_WEIGHT_FINGER_EFFORT)
    }

    /// Gets the default cost in milliseconds (if using time-based scoring).
    #[must_use]
    pub fn get_default_cost_ms(&self) -> Score {
        self.get_weight("default_cost_ms", DEFAULT_COST_MS)
    }
    /// Gets the limit on the number of trigrams to load.
    #[must_use]
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        clippy::cast_sign_loss,
        clippy::cast_possible_wrap
    )]
    pub fn get_loader_trigram_limit(&self) -> usize {
        let val = self.get_weight(
            "loader_trigram_limit",
            Score::from_scaled_i64(
                i64::try_from(DEFAULT_LOADER_TRIGRAM_LIMIT).unwrap_or_default() * 1_000_000,
            ),
        );
        usize::try_from(val.raw() / 1_000_000).unwrap_or(0)
    }
    /// Gets the required trigram coverage.
    #[must_use]
    pub fn get_trigram_coverage(&self) -> Score {
        self.get_weight("trigram_coverage", DEFAULT_TRIGRAM_COVERAGE)
    }

    /// Returns the finger penalty scale array.
    #[must_use]
    pub fn get_finger_penalty_scale(&self) -> [Score; 5] {
        self.finger_penalty_scale
    }
    /// Calculates the allowed deviation from perfect hand balance.
    #[must_use]
    pub fn allowed_hand_balance_deviation(&self) -> f32 {
        (self.get_max_hand_imbalance().to_f32() - 0.5).max(0.0)
    }
    /// Parses the comfortable scissors string into a list of pairs.
    #[must_use]
    pub fn get_comfortable_scissors(&self) -> Vec<(u8, u8)> {
        let mut pairs = Vec::new();
        for s in self.comfortable_scissors.split(',') {
            let s = s.trim();
            if s.len() == 2 {
                let bytes = s.as_bytes();
                if bytes[0] >= b'0' && bytes[1] >= b'0' {
                    pairs.push((bytes[0] - b'0', bytes[1] - b'0'));
                }
            }
        }
        pairs
    }
}
