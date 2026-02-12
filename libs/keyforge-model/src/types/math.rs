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
    pub const SCALE: i64 = 1000;

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
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    pub fn from_f32(val: f32) -> Self {
        // SCALE is 1000, which fits exactly in f64.
        let result_f64 = (f64::from(val) * 1000.0).round();
        Self(i32::try_from(result_f64 as i64).unwrap_or(0)) // sg-ignore
    }

    /// Converts `SpatialUnit` to a float KU value.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    pub fn to_f32(self) -> f32 {
        (f64::from(self.0) / 1000.0) as f32 // sg-ignore
    }
}

impl std::fmt::Display for SpatialUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.3}", self.to_f32())
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
    #[allow(clippy::cast_precision_loss)]
    fn scale() -> f64 {
        1000.0
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

    /// Creates a new `Point` from f32 coordinates.
    #[must_use]
    pub fn from_f32(x: f32, y: f32) -> Self {
        Self {
            x: SpatialUnit::from_f32(x),
            y: SpatialUnit::from_f32(y),
        }
    }

    /// Calculates the squared distance between two points.
    #[must_use]
    pub fn dist_sq(self, other: Self) -> i64 {
        let dx = i64::from(other.x.raw() - self.x.raw());
        let dy = i64::from(other.y.raw() - self.y.raw());
        dx * dx + dy * dy
    }
}
