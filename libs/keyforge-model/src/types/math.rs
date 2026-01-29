// libs/keyforge-model/src/types/math.rs

/// Trait for bit-perfect deterministic arithmetic across the domain model.
pub trait FixedPointMath {
    /// The raw storage type.
    type Raw;

    /// Returns the raw value.
    fn raw(self) -> Self::Raw;

    /// Creates a value from a raw scaled value.
    fn from_raw(val: Self::Raw) -> Self;

    /// Scaling factor used for this type.
    fn scale() -> f64;
}

/// Units for physical distance on the keyboard (1000 units = 1.0 Key Unit).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct SpatialUnit(i32);

impl SpatialUnit {
    /// Scaling factor for spatial units (1000).
    pub const SCALE: f64 = 1000.0;

    /// Creates a new `SpatialUnit`.
    #[must_use]
    pub const fn new(val: i32) -> Self {
        Self(val)
    }

    /// Returns the raw `i32` value.
    #[must_use]
    pub const fn raw(self) -> i32 {
        self.0
    }

    /// Converts a float KU value to `SpatialUnit`.
    #[must_use]
    pub fn from_f32(val: f32) -> Self {
        Self((val as f64 * Self::SCALE).round() as i32)
    }

    /// Converts `SpatialUnit` to a float KU value.
    #[must_use]
    pub fn to_f32(self) -> f32 {
        (self.0 as f64 / Self::SCALE) as f32
    }
}

impl FixedPointMath for SpatialUnit {
    type Raw = i32;
    fn raw(self) -> Self::Raw {
        self.0
    }
    fn from_raw(val: Self::Raw) -> Self {
        Self(val)
    }
    fn scale() -> f64 {
        Self::SCALE
    }
}

/// A 2D point in deterministic spatial units.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Point {
    /// X coordinate.
    pub x: SpatialUnit,
    /// Y coordinate.
    pub y: SpatialUnit,
}

impl Point {
    /// Creates a new `Point`.
    #[must_use]
    pub const fn new(x: SpatialUnit, y: SpatialUnit) -> Self {
        Self { x, y }
    }

    /// Calculates the squared distance between two points.
    #[must_use]
    pub fn dist_sq(self, other: Self) -> i64 {
        let dx = i64::from(self.x.raw()) - i64::from(other.x.raw());
        let dy = i64::from(self.y.raw()) - i64::from(other.y.raw());
        dx * dx + dy * dy
    }
}
