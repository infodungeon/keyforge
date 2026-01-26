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

/// Number of iterations for an optimization step.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ToSchema, Default,
)]
#[serde(transparent)]
#[repr(transparent)]
pub struct IterationCount(pub usize);

/// Simulated annealing temperature.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize, ToSchema, Default)]
#[serde(transparent)]
#[repr(transparent)]
pub struct Temperature(pub f32);

/// Patience limit for stagnant optimization runs.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ToSchema, Default,
)]
#[serde(transparent)]
#[repr(transparent)]
pub struct PatienceCount(pub usize);

/// Number of reheating cycles.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ToSchema, Default,
)]
#[serde(transparent)]
#[repr(transparent)]
pub struct ReheatCount(pub usize);

/// Generic scaling or adjustment factor.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize, ToSchema, Default)]
#[serde(transparent)]
#[repr(transparent)]
pub struct ScalingFactor(pub f32);
