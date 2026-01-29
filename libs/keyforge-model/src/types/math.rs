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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, utoipa::ToSchema)]
#[schema(as = f32)]
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

impl std::fmt::Display for SpatialUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.3}", self.to_f32())
    }
}

impl serde::Serialize for SpatialUnit {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_f32(self.to_f32())
    }
}

impl<'de> serde::Deserialize<'de> for SpatialUnit {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let val = f32::deserialize(deserializer)?;
        Ok(Self::from_f32(val))
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
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
        Movement::from_points(self, other).dist_sq()
    }
}

/// Represents the transition between two points on the keyboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Movement {
    /// Horizontal offset in spatial units.
    pub dx: i32,
    /// Vertical offset in spatial units.
    pub dy: i32,
}

impl Movement {
    /// Creates a `Movement` from two points.
    #[must_use]
    pub fn from_points(start: Point, end: Point) -> Self {
        Self {
            dx: end.x.raw() - start.x.raw(),
            dy: end.y.raw() - start.y.raw(),
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
}

/// Analyzes a sequence of three points for biomechanical flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrigramFlow {
    /// Movement from the first to second point.
    pub m1: Movement,
    /// Movement from the second to third point.
    pub m2: Movement,
}

impl TrigramFlow {
    /// Creates a `TrigramFlow` from three points.
    #[must_use]
    pub fn from_points(p1: Point, p2: Point, p3: Point) -> Self {
        Self {
            m1: Movement::from_points(p1, p2),
            m2: Movement::from_points(p2, p3),
        }
    }

    /// Returns true if the sequence represents a redirect (direction change on either axis).
    #[must_use]
    pub fn is_redirect(&self) -> bool {
        (self.m1.dx.signum() != self.m2.dx.signum() && self.m1.dx != 0 && self.m2.dx != 0)
            || (self.m1.dy.signum() != self.m2.dy.signum() && self.m1.dy != 0 && self.m2.dy != 0)
    }
}
