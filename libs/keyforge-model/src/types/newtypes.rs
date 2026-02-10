// libs/keyforge-model/src/types/newtypes.rs

// --- Domain-Specific Newtypes (Anti-Primitive Obsession) ---

/// A security-bounded duration in milliseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
#[repr(transparent)]
pub struct DurationMs(u64);

impl DurationMs {
    /// Creates a new `DurationMs`.
    #[must_use]
    pub const fn new(val: u64) -> Self {
        Self(val)
    }
    /// Returns the raw `u64` value.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// High-precision latency measurement in milliseconds.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
#[repr(transparent)]
pub struct LatencyMs(f64);

impl LatencyMs {
    /// Creates a new `LatencyMs`.
    #[must_use]
    pub const fn new(val: f64) -> Self {
        Self(val)
    }
    /// Returns the raw `f64` value.
    #[must_use]
    pub const fn raw(self) -> f64 {
        self.0
    }
}

/// Number of iterations for an optimization step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
#[repr(transparent)]
pub struct IterationCount(usize);

impl IterationCount {
    /// Creates a new `IterationCount`.
    #[must_use]
    pub const fn new(val: usize) -> Self {
        Self(val)
    }
    /// Returns the raw `usize` value.
    #[must_use]
    pub const fn raw(self) -> usize {
        self.0
    }
}

impl From<usize> for IterationCount {
    fn from(val: usize) -> Self {
        Self(val)
    }
}

/// Simulated annealing temperature.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
#[repr(transparent)]
pub struct Temperature(f32);

impl Temperature {
    /// Creates a new `Temperature`.
    #[must_use]
    pub const fn new(val: f32) -> Self {
        Self(val)
    }
    /// Returns the raw `f32` value.
    #[must_use]
    pub const fn raw(self) -> f32 {
        self.0
    }
}

impl std::ops::MulAssign<f32> for Temperature {
    fn mul_assign(&mut self, rhs: f32) {
        self.0 *= rhs;
    }
}

impl std::ops::Mul<f32> for Temperature {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Self(self.0 * rhs)
    }
}

impl std::ops::Mul<ScalingFactor> for Temperature {
    type Output = Self;
    #[allow(clippy::cast_precision_loss)]
    fn mul(self, rhs: ScalingFactor) -> Self {
        Self(self.0 * (rhs.raw() as f32))
    }
}

impl From<f32> for Temperature {
    fn from(val: f32) -> Self {
        Self(val)
    }
}

/// Patience limit for stagnant optimization runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
#[repr(transparent)]
pub struct PatienceCount(usize);

impl PatienceCount {
    /// Creates a new `PatienceCount`.
    #[must_use]
    pub const fn new(val: usize) -> Self {
        Self(val)
    }
    /// Returns the raw `usize` value.
    #[must_use]
    pub const fn raw(self) -> usize {
        self.0
    }
}

impl From<usize> for PatienceCount {
    fn from(val: usize) -> Self {
        Self(val)
    }
}

/// Number of reheating cycles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
#[repr(transparent)]
pub struct ReheatCount(usize);

impl ReheatCount {
    /// Creates a new `ReheatCount`.
    #[must_use]
    pub const fn new(val: usize) -> Self {
        Self(val)
    }
    /// Returns the raw `usize` value.
    #[must_use]
    pub const fn raw(self) -> usize {
        self.0
    }
}

impl From<usize> for ReheatCount {
    fn from(val: usize) -> Self {
        Self(val)
    }
}

/// Generic scaling or adjustment factor for bit-perfect deterministic math.
///
/// Use `raw()`/`from_raw()` for integer-only arithmetic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
#[repr(transparent)]
pub struct ScalingFactor(i64);

impl ScalingFactor {
    /// Creates a new `ScalingFactor`.
    #[must_use]
    pub const fn new(val: i64) -> Self {
        Self(val)
    }

    /// Deterministically converts an f32 multiplier to a `ScalingFactor`.
    /// This is used solely at the boundary between float-based evolution and integer physics.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn from_f32(val: f32) -> Self {
        // Use rounding to ensure deterministic behavior across platforms
        Self(f64::from(val).round() as i64) // sg-ignore: mandated boundary conversion
    }
    /// Returns the raw `i64` value.
    #[must_use]
    pub const fn raw(self) -> i64 {
        self.0
    }
    /// Converts to f32 for compatibility with legacy float-based logic.
    #[allow(clippy::cast_precision_loss)]
    #[must_use]
    pub fn to_f32(self) -> f32 {
        self.0 as f32
    }
}

impl From<i64> for ScalingFactor {
    fn from(val: i64) -> Self {
        Self(val)
    }
}

/// A deterministic seed for PRNGs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
#[repr(transparent)]
pub struct Seed(u64);

impl Seed {
    /// Creates a new `Seed`.
    #[must_use]
    pub const fn new(val: u64) -> Self {
        Self(val)
    }
    /// Returns the raw `u64` value.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

impl From<u64> for Seed {
    fn from(val: u64) -> Self {
        Self(val)
    }
}
