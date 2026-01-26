// libs/keyforge-model/src/types/newtypes.rs

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// --- Domain-Specific Newtypes (Anti-Primitive Obsession) ---

/// A security-bounded duration in milliseconds.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ToSchema, Default,
)]
#[serde(transparent)]
#[repr(transparent)]
pub struct DurationMs(pub u64);

/// High-precision latency measurement in milliseconds.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize, ToSchema, Default)]
#[serde(transparent)]
#[repr(transparent)]
pub struct LatencyMs(pub f64);

/// Number of iterations for an optimization step.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ToSchema, Default,
)]
#[serde(transparent)]
#[repr(transparent)]
pub struct IterationCount(pub usize);

impl From<usize> for IterationCount {
    fn from(val: usize) -> Self {
        Self(val)
    }
}

/// Simulated annealing temperature.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize, ToSchema, Default)]
#[serde(transparent)]
#[repr(transparent)]
pub struct Temperature(pub f32);

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

impl From<f32> for Temperature {
    fn from(val: f32) -> Self {
        Self(val)
    }
}

/// Patience limit for stagnant optimization runs.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ToSchema, Default,
)]
#[serde(transparent)]
#[repr(transparent)]
pub struct PatienceCount(pub usize);

impl From<usize> for PatienceCount {
    fn from(val: usize) -> Self {
        Self(val)
    }
}

/// Number of reheating cycles.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ToSchema, Default,
)]
#[serde(transparent)]
#[repr(transparent)]
pub struct ReheatCount(pub usize);

impl From<usize> for ReheatCount {
    fn from(val: usize) -> Self {
        Self(val)
    }
}

/// Generic scaling or adjustment factor.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize, ToSchema, Default)]
#[serde(transparent)]
#[repr(transparent)]
pub struct ScalingFactor(pub f32);

impl From<f32> for ScalingFactor {
    fn from(val: f32) -> Self {
        Self(val)
    }
}

/// A deterministic seed for PRNGs.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ToSchema, Default,
)]
#[serde(transparent)]
#[repr(transparent)]
pub struct Seed(pub u64);

impl From<u64> for Seed {
    fn from(val: u64) -> Self {
        Self(val)
    }
}