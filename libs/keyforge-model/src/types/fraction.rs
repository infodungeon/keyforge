// libs/keyforge-model/src/types/fraction.rs

/// Represents a rational fraction for deterministic physics calculations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fraction {
    /// The numerator of the fraction.
    pub numerator: i64,
    /// The denominator of the fraction.
    pub denominator: i64,
}

impl Fraction {
    /// Creates a new `Fraction`.
    #[must_use]
    pub const fn new(numerator: i64, denominator: i64) -> Self {
        Self {
            numerator,
            denominator,
        }
    }
}

impl Default for Fraction {
    fn default() -> Self {
        Self {
            numerator: 1,
            denominator: 1,
        }
    }
}
