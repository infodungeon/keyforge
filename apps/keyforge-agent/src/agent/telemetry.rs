// Copyright (c) 2025 KeyForge Contributors
//
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
use keyforge_core::ProgressCallback;
use keyforge_model::KeyCode;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::info;
use crate::models::SharedTelemetry;

/// A logger that tracks optimization progress and reports it back to the system.
pub struct WorkerLogger {
    pub stop_flag: Arc<AtomicBool>,
    pub job_id: String,
    pub telemetry: SharedTelemetry,
}

impl ProgressCallback for WorkerLogger {
    fn on_progress(&self, step: usize, score: f32, _layout: &[KeyCode], ips: f32) -> bool {
        let stopped = self.stop_flag.load(Ordering::SeqCst);
        
        // Update shared telemetry (Lock-free)
        // Note: Core doesn't pass temp yet, so we pass 0.0 or infer it later.
        self.telemetry.update(ips, 0.0, score);

        if stopped {
            return false;
        }

        let mut hasher = DefaultHasher::new();
        self.job_id.hash(&mut hasher);
        step.hash(&mut hasher);
        let hash = hasher.finish();

        if hash % 100 == 0 {
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