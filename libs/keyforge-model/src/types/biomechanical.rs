// libs/keyforge-model/src/types/biomechanical.rs

use crate::types::RowIndex;

/// The physical hand used for typing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Hand {
    /// The left hand.
    #[default]
    Left,
    /// The right hand.
    Right,
}

impl Hand {
    /// Returns the index of the hand (Left=0, Right=1).
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Left => 0,
            Self::Right => 1,
        }
    }
}

impl From<HandIndex> for Hand {
    fn from(idx: HandIndex) -> Self {
        if idx.is_left() {
            Self::Left
        } else {
            Self::Right
        }
    }
}

impl From<Hand> for HandIndex {
    fn from(hand: Hand) -> Self {
        Self(u8::try_from(hand.index()).unwrap_or(0))
    }
}

/// The finger used for typing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Finger {
    /// The thumb.
    Thumb,
    /// The index finger.
    #[default]
    Index,
    /// The middle finger.
    Middle,
    /// The ring finger.
    Ring,
    /// The pinky finger.
    Pinky,
}

impl Finger {
    /// Returns the index of the finger (Thumb=0 to Pinky=4).
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Thumb => 0,
            Self::Index => 1,
            Self::Middle => 2,
            Self::Ring => 3,
            Self::Pinky => 4,
        }
    }

    /// Returns true if this is considered a "weak" finger (Ring or Pinky).
    #[must_use]
    pub const fn is_weak(self) -> bool {
        matches!(self, Self::Ring | Self::Pinky)
    }

    /// Calculates the absolute distance between two fingers.
    #[must_use]
    pub fn distance(self, other: Self) -> u8 {
        u8::try_from(self.index())
            .unwrap_or(0)
            .abs_diff(u8::try_from(other.index()).unwrap_or(0))
    }
}

impl From<FingerIndex> for Finger {
    fn from(idx: FingerIndex) -> Self {
        match idx.raw() {
            0 => Self::Thumb,
            2 => Self::Middle,
            3 => Self::Ring,
            4 => Self::Pinky,
            _ => Self::Index,
        }
    }
}

impl From<Finger> for FingerIndex {
    fn from(finger: Finger) -> Self {
        Self(u8::try_from(finger.index()).unwrap_or(0))
    }
}

/// A physical zone on the keyboard relative to a finger's home position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Zone {
    /// The home column for the finger.
    Base,
    /// A column closer to the center of the keyboard.
    InnerReach,
    /// A column closer to the edge of the keyboard.
    OuterReach,
}

/// The direction of movement across the keyboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    /// Movement towards the center (Pinky -> Thumb).
    Inward,
    /// Movement towards the edge (Thumb -> Pinky).
    Outward,
    /// No horizontal movement (same finger or column).
    Neutral,
}

/// Represents a rich biomechanical movement between two keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Movement {
    /// Horizontal offset in spatial units.
    pub dx: i32,
    /// Vertical offset in spatial units.
    pub dy: i32,
    /// Hand for the first key.
    pub h1: Hand,
    /// Hand for the second key.
    pub h2: Hand,
    /// Finger for the first key.
    pub f1: Finger,
    /// Finger for the second key.
    pub f2: Finger,
    /// Row for the first key.
    pub r1: RowIndex,
    /// Row for the second key.
    pub r2: RowIndex,
}

impl Movement {
    /// Creates a `Movement` from two points and biomechanical context.
    #[must_use]
    pub fn new(
        dx: i32,
        dy: i32,
        h1: Hand,
        h2: Hand,
        f1: Finger,
        f2: Finger,
        r1: RowIndex,
        r2: RowIndex,
    ) -> Self {
        Self {
            dx,
            dy,
            h1,
            h2,
            f1,
            f2,
            r1,
            r2,
        }
    }

    /// Creates a `Movement` from two points (purely geometric, uses defaults for biomechanical context).
    #[must_use]
    pub fn from_points(start: crate::types::Point, end: crate::types::Point) -> Self {
        Self {
            dx: end.x.raw() - start.x.raw(),
            dy: end.y.raw() - start.y.raw(),
            ..Default::default()
        }
    }

    /// Creates a `Movement` from two `KeyNode`s.
    #[must_use]
    pub fn from_keys(k1: &crate::geometry::KeyNode, k2: &crate::geometry::KeyNode) -> Self {
        Self {
            dx: k2.x().raw() - k1.x().raw(),
            dy: k2.y().raw() - k1.y().raw(),
            h1: Hand::from(k1.hand()),
            h2: Hand::from(k2.hand()),
            f1: Finger::from(k1.finger()),
            f2: Finger::from(k2.finger()),
            r1: k1.row(),
            r2: k2.row(),
        }
    }

    /// Calculates the squared Euclidean distance.
    #[must_use]
    pub fn dist_sq(&self) -> i64 {
        let dx = i64::from(self.dx);
        let dy = i64::from(self.dy);
        dx * dx + dy * dy
    }

    /// Calculates the Manhattan distance.
    #[must_use]
    pub fn manhattan(&self) -> i32 {
        self.dx.abs() + self.dy.abs()
    }

    /// Returns true if the movement is purely vertical.
    #[must_use]
    pub fn is_vertical(&self) -> bool {
        self.dx == 0 && self.dy != 0
    }

    /// Returns true if the movement is purely horizontal.
    #[must_use]
    pub fn is_horizontal(&self) -> bool {
        self.dy == 0 && self.dx != 0
    }

    /// Returns true if the movement constitutes a "scissor" (uncomfortable row jump between adjacent fingers).
    #[must_use]
    pub fn is_scissor(&self, threshold: i8) -> bool {
        if self.h1 != self.h2 || self.f1 == self.f2 {
            return false;
        }

        let f_dist = self.f1.distance(self.f2);
        if f_dist != 1 || self.f1 == Finger::Thumb || self.f2 == Finger::Thumb {
            return false;
        }

        let row_diff = (i32::from(self.r1.raw()) - i32::from(self.r2.raw())).abs();
        row_diff >= i32::from(threshold)
    }

    /// Returns true if the movement is a Same-Finger Bigram (SFB).
    #[must_use]
    pub fn is_sfb(&self) -> bool {
        self.h1 == self.h2 && self.f1 == self.f2
    }

    /// Returns true if both keys are on the same hand.
    #[must_use]
    pub fn is_same_hand(&self) -> bool {
        self.h1 == self.h2
    }
}

impl crate::validator::Validator for Movement {
    fn validate(&self) -> Result<(), String> {
        // Enums ensure hand/finger validity.
        // dx/dy are raw spatial units, no specific domain invariants beyond being i32.
        Ok(())
    }
}

/// Analyzes a sequence of three keys for biomechanical flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TrigramFlow {
    /// Movement from the first to second key.
    pub m1: Movement,
    /// Movement from the second to third key.
    pub m2: Movement,
}

impl TrigramFlow {
    /// Creates a `TrigramFlow` from three `KeyNode`s.
    #[must_use]
    pub fn from_keys(
        k1: &crate::geometry::KeyNode,
        k2: &crate::geometry::KeyNode,
        k3: &crate::geometry::KeyNode,
    ) -> Self {
        Self {
            m1: Movement::from_keys(k1, k2),
            m2: Movement::from_keys(k2, k3),
        }
    }

    /// Returns true if the sequence represents a redirect (direction change).
    #[must_use]
    #[allow(clippy::similar_names)]
    pub fn is_redirect(&self) -> bool {
        if !self.m1.is_same_hand() || self.m1.h2 != self.m2.h1 || !self.m2.is_same_hand() {
            return false;
        }

        // One-finger redirect (f1 == f3 && f1 != f2)
        if self.m1.f1 == self.m2.f2 && self.m1.f1 != self.m1.f2 {
            return true;
        }

        // Directional redirect (direction change on either axis)
        let dx_redirect =
            self.m1.dx.signum() != self.m2.dx.signum() && self.m1.dx != 0 && self.m2.dx != 0;
        let dy_redirect =
            self.m1.dy.signum() != self.m2.dy.signum() && self.m1.dy != 0 && self.m2.dy != 0;

        dx_redirect || dy_redirect
    }

    /// Returns true if the sequence is an inward roll (Outer -> Inner).
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    pub fn is_roll_in(&self) -> bool {
        if !self.m1.is_same_hand() || self.m1.h2 != self.m2.h1 || !self.m2.is_same_hand() {
            return false;
        }

        let dir1 = self.m1.f2.index() as i16 - self.m1.f1.index() as i16;
        let dir2 = self.m2.f2.index() as i16 - self.m2.f1.index() as i16;

        if dir1 == 0 || dir2 == 0 {
            return false;
        }

        // All moving in same direction and inward (Outer -> Inner)
        // In our indexing (0=Thumb, 4=Pinky), inward is decreasing index.
        dir1 < 0 && dir2 < 0
    }

    /// Returns true if the sequence is an outward roll (Inner -> Outer).
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    pub fn is_roll_out(&self) -> bool {
        if !self.m1.is_same_hand() || self.m1.h2 != self.m2.h1 || !self.m2.is_same_hand() {
            return false;
        }

        let dir1 = self.m1.f2.index() as i16 - self.m1.f1.index() as i16;
        let dir2 = self.m2.f2.index() as i16 - self.m2.f1.index() as i16;

        if dir1 == 0 || dir2 == 0 {
            return false;
        }

        // All moving in same direction and outward (Inner -> Outer)
        // In our indexing (0=Thumb, 4=Pinky), outward is increasing index.
        dir1 > 0 && dir2 > 0
    }
}

impl crate::validator::Validator for TrigramFlow {
    fn validate(&self) -> Result<(), String> {
        self.m1.validate()?;
        self.m2.validate()?;
        if self.m1.h2 != self.m2.h1 {
            return Err(
                "TrigramFlow movements must be contiguous (h1.end != h2.start)".to_string(),
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::KeyNode;
    use crate::types::{ColIndex, KeyIndex, RowIndex, SpatialUnit};

    fn create_test_key(index: u16, hand: u8, finger: u8, row: i8, col: i8) -> KeyNode {
        KeyNode::builder()
            .index(KeyIndex::new(index))
            .hand(HandIndex::new(hand))
            .finger(FingerIndex::new_unchecked(finger))
            .row(RowIndex::new(row))
            .col(ColIndex::new(col))
            .x(SpatialUnit::from_f32(col as f32))
            .y(SpatialUnit::from_f32(row as f32))
            .build()
    }

    #[test]
    fn test_movement_scissor() {
        // Scissor: adjacent fingers, row diff >= threshold
        let k1 = create_test_key(0, 0, 1, 0, 0); // Left, Index, Row 0
        let k2 = create_test_key(1, 0, 2, 2, 1); // Left, Middle, Row 2
        let m = Movement::from_keys(&k1, &k2);

        assert!(m.is_scissor(2));
        assert!(!m.is_scissor(3));

        // Same finger: not a scissor
        let k3 = create_test_key(2, 0, 1, 2, 0);
        let m2 = Movement::from_keys(&k1, &k3);
        assert!(!m2.is_scissor(1));
    }

    #[test]
    fn test_movement_sfb() {
        let k1 = create_test_key(0, 0, 1, 0, 0);
        let k2 = create_test_key(1, 0, 1, 1, 0);
        let m = Movement::from_keys(&k1, &k2);
        assert!(m.is_sfb());

        let k3 = create_test_key(2, 0, 2, 1, 1);
        let m2 = Movement::from_keys(&k1, &k3);
        assert!(!m2.is_sfb());
    }

    #[test]
    fn test_trigram_flow_redirect() {
        // Redirect: f1 == f3 && f1 != f2
        let k1 = create_test_key(0, 0, 1, 0, 0); // Index
        let k2 = create_test_key(1, 0, 2, 0, 1); // Middle
        let k3 = create_test_key(2, 0, 1, 0, 0); // Index again

        let flow = TrigramFlow::from_keys(&k1, &k2, &k3);
        assert!(flow.is_redirect());

        // Not a redirect: index -> middle -> ring
        let k4 = create_test_key(3, 0, 3, 0, 2); // Ring
        let flow2 = TrigramFlow::from_keys(&k1, &k2, &k4);
        assert!(!flow2.is_redirect());
    }

    #[test]
    fn test_trigram_flow_rolls() {
        // Left hand: Pinky (4) -> Ring (3) -> Middle (2) is INWARD
        let k1 = create_test_key(0, 0, 4, 0, 0);
        let k2 = create_test_key(1, 0, 3, 0, 1);
        let k3 = create_test_key(2, 0, 2, 0, 2);

        let flow = TrigramFlow::from_keys(&k1, &k2, &k3);
        assert!(flow.is_roll_in());
        assert!(!flow.is_roll_out());

        // Right hand: Pinky (4) -> Ring (3) -> Middle (2) is INWARD
        let k4 = create_test_key(3, 1, 4, 0, 5);
        let k5 = create_test_key(4, 1, 3, 0, 4);
        let k6 = create_test_key(5, 1, 2, 0, 3);

        let flow2 = TrigramFlow::from_keys(&k4, &k5, &k6);
        assert!(flow2.is_roll_in());

        // Outward: Thumb (0) -> Index (1) -> Middle (2)
        let k7 = create_test_key(6, 0, 0, 0, 0);
        let k8 = create_test_key(7, 0, 1, 0, 1);
        let k9 = create_test_key(8, 0, 2, 0, 2);

        let flow3 = TrigramFlow::from_keys(&k7, &k8, &k9);
        assert!(flow3.is_roll_out());
    }
}

/// Identifier for a hand (Left=0, Right=1).
///
/// [DEPRECATED] Use `Hand` enum instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(transparent)]
pub struct HandIndex(u8);

impl HandIndex {
    /// Left hand index (0).
    pub const LEFT: Self = Self(0);
    /// Right hand index (1).
    pub const RIGHT: Self = Self(1);

    /// Creates a new `HandIndex`.
    #[must_use]
    pub const fn new(val: u8) -> Self {
        Self(val)
    }

    /// Returns the raw `u8` value.
    #[must_use]
    pub fn raw(&self) -> u8 {
        self.0
    }
    /// Returns the raw `u8` value.
    #[must_use]
    pub fn as_u8(&self) -> u8 {
        self.0
    }
    /// Returns the value as `usize`.
    #[must_use]
    pub fn as_usize(&self) -> usize {
        usize::from(self.0)
    }
    /// Returns true if this is the left hand.
    #[must_use]
    pub fn is_left(&self) -> bool {
        self.0 == 0
    }
    /// Returns true if this is the right hand.
    #[must_use]
    pub fn is_right(&self) -> bool {
        self.0 == 1
    }
}
impl TryFrom<u8> for HandIndex {
    type Error = String;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > 1 {
            Err(format!("Invalid HandIndex: {value}"))
        } else {
            Ok(Self(value))
        }
    }
}

/// Identifier for a finger (Thumb=0 to Pinky=4).
///
/// [DEPRECATED] Use `Finger` enum instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(transparent)]
pub struct FingerIndex(u8);

impl FingerIndex {
    /// Thumb index (0).
    pub const THUMB: Self = Self(0);
    /// Index finger index (1).
    pub const INDEX: Self = Self(1);
    /// Middle finger index (2).
    pub const MIDDLE: Self = Self(2);
    /// Ring finger index (3).
    pub const RING: Self = Self(3);
    /// Pinky finger index (4).
    pub const PINKY: Self = Self(4);

    /// Creates a new `FingerIndex`.
    #[must_use]
    pub const fn new(val: u8) -> Self {
        Self(val)
    }

    /// Creates a new `FingerIndex`.
    ///
    /// # Safety
    /// Calling this with a value > 4 violates domain invariants.
    #[must_use]
    pub const fn new_unchecked(val: u8) -> Self {
        Self(val)
    }

    /// Returns the raw `u8` value.
    #[must_use]
    pub fn raw(&self) -> u8 {
        self.0
    }
    /// Returns the raw `u8` value.
    #[must_use]
    pub fn as_u8(&self) -> u8 {
        self.0
    }
    /// Returns the value as `usize`.
    #[must_use]
    pub fn as_usize(&self) -> usize {
        usize::from(self.0)
    }

    /// Calculates the absolute distance between two fingers.
    #[must_use]
    pub fn distance(&self, other: Self) -> u8 {
        self.0.abs_diff(other.0)
    }

    /// Calculates the signed difference between two fingers.
    #[must_use]
    pub fn diff(&self, other: Self) -> i16 {
        i16::from(self.0) - i16::from(other.0)
    }

    /// Returns true if this is considered a "weak" finger (Ring or Pinky).
    #[must_use]
    pub fn is_weak(&self) -> bool {
        self.0 == 3 || self.0 == 4
    }
}
impl TryFrom<u8> for FingerIndex {
    type Error = String;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > 4 {
            Err(format!("Invalid FingerIndex: {value}"))
        } else {
            Ok(Self(value))
        }
    }
}

/// Preference for which hand should handle Space keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SpaceHandPreference {
    /// Only use left hand for space.
    Left,
    /// Only use right hand for space.
    Right,
    /// Use both hands (load balanced).
    #[default]
    Bilateral,
}
