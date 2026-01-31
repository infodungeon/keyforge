// libs/keyforge-model/src/types/biomechanical.rs

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// The physical hand used for typing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum Hand {
    /// The left hand.
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

impl From<Hand> for HandIndex {
    fn from(hand: Hand) -> Self {
        #[allow(clippy::cast_possible_truncation)]
        Self(hand.index() as u8)
    }
}

/// The finger used for typing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum Finger {
    /// The thumb.
    Thumb,
    /// The index finger.
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
        #[allow(clippy::cast_possible_truncation)]
        (self.index() as u8).abs_diff(other.index() as u8)
    }
}

impl From<Finger> for FingerIndex {
    fn from(finger: Finger) -> Self {
        #[allow(clippy::cast_possible_truncation)]
        Self(finger.index() as u8)
    }
}

/// A physical zone on the keyboard relative to a finger's home position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum Zone {
    /// The home column for the finger.
    Base,
    /// A column closer to the center of the keyboard.
    InnerReach,
    /// A column closer to the edge of the keyboard.
    OuterReach,
}

/// The direction of movement across the keyboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    /// Movement towards the center (Pinky -> Thumb).
    Inward,
    /// Movement towards the edge (Thumb -> Pinky).
    Outward,
    /// No horizontal movement (same finger or column).
    Neutral,
}

/// Identifier for a hand (Left=0, Right=1).
///
/// [DEPRECATED] Use `Hand` enum instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(transparent)]
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
        self.0 as usize
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
impl Default for HandIndex {
    fn default() -> Self {
        Self::LEFT
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(transparent)]
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
        self.0 as usize
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
impl Default for FingerIndex {
    fn default() -> Self {
        Self::INDEX
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema, Default)]
#[serde(rename_all = "lowercase")]
pub enum SpaceHandPreference {
    /// Only use left hand for space.
    Left,
    /// Only use right hand for space.
    Right,
    /// Use both hands (load balanced).
    #[default]
    Bilateral,
}
