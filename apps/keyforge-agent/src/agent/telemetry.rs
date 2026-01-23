// apps/keyforge-agent/src/agent/telemetry.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::models::SharedTelemetry;
use keyforge_compute::{OptimizationControl, ProgressCallback};
use keyforge_model::KeyCode;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::info;

/// A logger that tracks optimization progress and reports it back to the system.
#[derive(Debug)]
pub struct WorkerLogger {
    /// Atomic flag used to signals the engine to stop gracefully.
    pub stop_flag: Arc<AtomicBool>,
    /// The unique identifier of the job being logged.
    pub job_id: String,
    /// Shared telemetry handle for live metric updates.
    pub telemetry: SharedTelemetry,
    /// Rate at which to log progress (e.g. every N steps).
    pub sample_rate: usize,
}

impl ProgressCallback for WorkerLogger {
    fn on_progress(
        &self,
        step: usize,
        score: f32,
        _layout: &[KeyCode],
        ips: f32,
    ) -> OptimizationControl {
        let stopped = self.stop_flag.load(Ordering::SeqCst);

        // Update shared telemetry (Lock-free)
        // Agent UI expects MOPS, so we scale back for now until UI is updated.
        self.telemetry.update(ips / 1_000_000.0, 0.0, score);

        if stopped {
            return OptimizationControl::Abort;
        }

        if step.is_multiple_of(self.sample_rate.max(1)) {
            info!(
                job_id = %self.job_id,
                step = step,
                score = score,
                ips = ips,
                "optimization progress"
            );
        }
        OptimizationControl::Continue
    }
}
