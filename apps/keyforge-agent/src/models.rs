use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;

// Re-export specific types if needed
pub use keyforge_protocol::{NodeRequest, NodeResponse, TuningProfile};

/// Shared state for real-time telemetry.
/// Uses atomics for lock-free updates from the hot loop.
#[derive(Debug, Default)]
pub struct AgentTelemetry {
    // f32 bits stored as u32
    pub ips: AtomicU32,
    pub temp: AtomicU32,
    pub best_score: AtomicU32,
    pub job_id_hash: AtomicU64, // Partial hash for identification
}

impl AgentTelemetry {
    pub fn update(&self, ips: f32, temp: f32, best_score: f32) {
        self.ips.store(ips.to_bits(), Ordering::Relaxed);
        self.temp.store(temp.to_bits(), Ordering::Relaxed);
        self.best_score.store(best_score.to_bits(), Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> (f32, f32, f32) {
        (
            f32::from_bits(self.ips.load(Ordering::Relaxed)),
            f32::from_bits(self.temp.load(Ordering::Relaxed)),
            f32::from_bits(self.best_score.load(Ordering::Relaxed)),
        )
    }
}

pub type SharedTelemetry = Arc<AgentTelemetry>;