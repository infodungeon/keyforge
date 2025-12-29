use keyforge_protocol::constants::SCORE_SCALE;
use std::convert::TryFrom;
use crate::errors::PhysicsError;

/// Represents a physical index into the keyboard arrays.
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
    type Error = PhysicsError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > 1 {
            Err(PhysicsError::InvalidHandIndex(value))
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
    type Error = PhysicsError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > 4 {
            Err(PhysicsError::InvalidFingerIndex(value))
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Score(pub i64);

impl Score {
    pub const MAX: Score = Score(i64::MAX);
    pub const MIN: Score = Score(i64::MIN);
    pub const ZERO: Score = Score(0);

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

/// A layout slice that has been validated against a specific key count.
/// This guarantees that the layout has enough keys to cover the keyboard geometry.
#[derive(Debug, Clone, Copy)]
pub struct ValidatedLayout<'a> {
    slice: &'a [u16],
}

impl<'a> ValidatedLayout<'a> {
    pub fn new(slice: &'a [u16], required_count: usize) -> Result<Self, PhysicsError> {
        if slice.len() < required_count {
            Err(PhysicsError::LayoutUnderflow(slice.len(), required_count))
        } else {
            Ok(Self { slice })
        }
    }

    pub fn as_slice(&self) -> &'a [u16] {
        self.slice
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_validated_layout() {
        let data = vec![0, 1, 2];
        assert!(ValidatedLayout::new(&data, 3).is_ok());
        assert!(ValidatedLayout::new(&data, 2).is_ok());
        assert!(ValidatedLayout::new(&data, 4).is_err());
    }

    #[test]
    fn test_hand_index_bounds() {
        assert!(HandIndex::try_from(0).is_ok());
        assert!(HandIndex::try_from(1).is_ok());
        assert!(HandIndex::try_from(2).is_err());
    }

    #[test]
    fn test_finger_index_bounds() {
        assert!(FingerIndex::try_from(0).is_ok());
        assert!(FingerIndex::try_from(4).is_ok());
        assert!(FingerIndex::try_from(5).is_err());
    }

    #[test]
    fn test_distance_squared_clamping() {
        assert_eq!(DistanceSquared::new(10.0).as_f32(), 10.0);
        assert_eq!(DistanceSquared::new(-5.0).as_f32(), 0.0);
    }

    #[test]
    fn test_score_saturation() {
        let max = Score::MAX;
        let min = Score::MIN;
        assert_eq!(max + Score(1), max);
        assert_eq!(min + Score(-1), min);
        assert_eq!(min - Score(1), min);
        assert_eq!(max - Score(-1), max);
        assert_eq!(max.saturating_mul(2), max);
        assert_eq!(min.saturating_mul(2), min);
    }

    #[test]
    fn test_score_float_conversion() {
        assert_eq!(Score::from_f32(f32::NAN), Score::ZERO);
        assert_eq!(Score::from_f32(f32::INFINITY), Score::MAX);
        assert_eq!(Score::from_f32(f32::NEG_INFINITY), Score::MIN);
        assert_eq!(Score::from_f32(f32::MAX), Score::MAX);
        assert_eq!(Score::from_f32(f32::MIN), Score::MIN);
        let val = 123.456;
        let score = Score::from_f32(val);
        assert!((score.to_f32() - val).abs() < 0.0001);
    }

    proptest! {
        #[test]
        fn test_score_commutativity(a in any::<i64>(), b in any::<i64>()) {
            let s1 = Score(a);
            let s2 = Score(b);
            prop_assert_eq!(s1 + s2, s2 + s1);
        }

        #[test]
        fn test_score_associativity_positive(a in 0..i64::MAX, b in 0..i64::MAX, c in 0..i64::MAX) {
             let s1 = Score(a);
             let s2 = Score(b);
             let s3 = Score(c);
             prop_assert_eq!((s1 + s2) + s3, s1 + (s2 + s3));
        }

        #[test]
        fn test_score_identity(a in any::<i64>()) {
            let s = Score(a);
            prop_assert_eq!(s + Score::ZERO, s);
            prop_assert_eq!(Score::ZERO + s, s);
            prop_assert_eq!(s - Score::ZERO, s);
        }
    }
}
