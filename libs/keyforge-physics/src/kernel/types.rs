use keyforge_protocol::constants::SCORE_SCALE;
use std::convert::TryFrom;

/// Represents a physical index into the keyboard arrays.
/// Prevents confusion with character codes (u16) or other indices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyIndex(pub usize);

impl KeyIndex {
    pub fn as_usize(&self) -> usize {
        self.0
    }
}

/// Represents which hand a key belongs to (0=Left, 1=Right).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandIndex(u8);

impl HandIndex {
    pub fn as_u8(&self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for HandIndex {
    type Error = &'static str;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > 1 {
            Err("Hand index must be 0 or 1")
        } else {
            Ok(Self(value))
        }
    }
}

/// Represents which finger a key belongs to (0=Thumb..4=Pinky).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FingerIndex(u8);

impl FingerIndex {
    pub fn as_u8(&self) -> u8 {
        self.0
    }
    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

impl TryFrom<u8> for FingerIndex {
    type Error = &'static str;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > 4 {
            Err("Finger index must be 0-4")
        } else {
            Ok(Self(value))
        }
    }
}

/// Represents a squared distance value.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct DistanceSquared(f32);

impl DistanceSquared {
    pub fn new(val: f32) -> Self {
        Self(val.max(0.0))
    }
    pub fn as_f32(&self) -> f32 {
        self.0
    }
}

/// Represents a fixed-point score value.
/// Wraps i64 to prevent accidental arithmetic with raw integers or floats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Score(pub i64);

impl Score {
    pub const MAX: Score = Score(i64::MAX);
    pub const MIN: Score = Score(i64::MIN);
    pub const ZERO: Score = Score(0);

    /// Safely converts a float to a fixed-point Score.
    /// Handles NaN (0), Infinity (MAX/MIN), and saturation.
    pub fn from_f32(val: f32) -> Self {
        if val.is_nan() {
            return Self::ZERO;
        }
        if val.is_infinite() {
            return if val.is_sign_positive() {
                Self::MAX
            } else {
                Self::MIN
            };
        }

        let scaled = val * SCORE_SCALE;

        if scaled >= i64::MAX as f32 {
            return Self::MAX;
        }
        if scaled <= i64::MIN as f32 {
            return Self::MIN;
        }

        Self(scaled as i64)
    }

    pub fn to_f32(self) -> f32 {
        self.0 as f32 / SCORE_SCALE
    }

    pub fn saturating_add(self, other: Score) -> Score {
        Score(self.0.saturating_add(other.0))
    }

    pub fn saturating_sub(self, other: Score) -> Score {
        Score(self.0.saturating_sub(other.0))
    }

    pub fn saturating_mul(self, factor: i64) -> Score {
        Score(self.0.saturating_mul(factor))
    }
}

impl std::ops::Add for Score {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        self.saturating_add(rhs)
    }
}

impl std::ops::Sub for Score {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        self.saturating_sub(rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hand_index_bounds() {
        assert!(HandIndex::try_from(0).is_ok());
        assert!(HandIndex::try_from(1).is_ok());
        assert!(HandIndex::try_from(2).is_err()); // Branch coverage
    }

    #[test]
    fn test_finger_index_bounds() {
        assert!(FingerIndex::try_from(0).is_ok());
        assert!(FingerIndex::try_from(4).is_ok());
        assert!(FingerIndex::try_from(5).is_err()); // Branch coverage
    }

    #[test]
    fn test_distance_squared_clamping() {
        assert_eq!(DistanceSquared::new(10.0).as_f32(), 10.0);
        assert_eq!(DistanceSquared::new(-5.0).as_f32(), 0.0); // Branch coverage
    }

    #[test]
    fn test_score_saturation() {
        let max = Score::MAX;
        let min = Score::MIN;

        // Add
        assert_eq!(max + Score(1), max);
        assert_eq!(min + Score(-1), min);

        // Sub
        assert_eq!(min - Score(1), min);
        assert_eq!(max - Score(-1), max);

        // Mul
        assert_eq!(max.saturating_mul(2), max);
        assert_eq!(min.saturating_mul(2), min);
    }

    #[test]
    fn test_score_float_conversion() {
        // NaN
        assert_eq!(Score::from_f32(f32::NAN), Score::ZERO);

        // Infinity
        assert_eq!(Score::from_f32(f32::INFINITY), Score::MAX);
        assert_eq!(Score::from_f32(f32::NEG_INFINITY), Score::MIN);

        // Overflow
        assert_eq!(Score::from_f32(f32::MAX), Score::MAX);
        assert_eq!(Score::from_f32(f32::MIN), Score::MIN);

        // Normal
        let val = 123.456;
        let score = Score::from_f32(val);
        // Expect close round-trip within epsilon due to fixed-point
        assert!((score.to_f32() - val).abs() < 0.0001);
    }
}
