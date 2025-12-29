use keyforge_core::ProgressCallback;
use keyforge_model::KeyCode;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::info;

/// A logger that tracks optimization progress and reports it back to the system.
///
/// It also monitors a `stop_flag` to allow for early termination of the optimization process.
pub struct WorkerLogger {
    pub stop_flag: Arc<AtomicBool>,
    pub job_id: String,
}

impl ProgressCallback for WorkerLogger {
    /// Called by the optimizer to report current progress.
    ///
    /// # Parameters
    /// - `step`: The current iteration step.
    /// - `score`: The current best score.
    /// - `_layout`: The current layout (unused by this logger).
    /// - `ips`: Iterations per second.
    ///
    /// Returns `false` if the optimization should stop immediately.
    fn on_progress(&self, step: usize, score: f32, _layout: &[KeyCode], ips: f32) -> bool {
        // Task 12: Use SeqCst for AtomicBool memory ordering
        if self.stop_flag.load(Ordering::SeqCst) {
            return false;
        }

        // Deterministic Sampling:
        // We want to log ~1% of steps, but consistently for the same job/step combo.
        let mut hasher = DefaultHasher::new();
        self.job_id.hash(&mut hasher);
        step.hash(&mut hasher);
        let hash = hasher.finish();

        // 1% sample rate (approx)
        if hash.is_multiple_of(100) {
            // Task 27: Structured logging
            info!(
                job_id = %self.job_id,
                step = step,
                score = score,
                ips = ips,
                "optimization progress"
            );
        }
        true
    }
}
