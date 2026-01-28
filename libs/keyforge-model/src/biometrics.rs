// libs/keyforge-model/src/biometrics.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You    may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Biometric profiling and timing models.

use crate::types::UserId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

/// A refined profile of a user's typing performance.
/// Generated from raw biometric samples to provide statistical insights.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Default)]
pub struct BiometricProfile {
    /// The user this profile belongs to.
    pub user_id: UserId,
    /// Mapping of bigram (char1, char2) to statistical timing data.
    pub bigram_latencies: HashMap<(u16, u16), LatencyStats>,
    /// Summary of overall typing efficiency.
    pub performance_index: f32,
}

/// Statistical summary of timing for a specific action.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema, Default)]
pub struct LatencyStats {
    /// Median latency in milliseconds.
    pub median_ms: f32,
    /// Standard deviation of latency.
    pub std_dev: f32,
    /// Number of samples used to calculate these stats.
    pub sample_count: usize,
}

/// A raw biometric measurement.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
pub struct BiometricSample {
    /// First keycode in the sequence.
    pub key_a: u16,
    /// Second keycode in the sequence.
    pub key_b: u16,
    /// Measured duration between events in milliseconds.
    pub duration_ms: u32,
}

impl BiometricProfile {
    /// Creates a new empty profile for a user.
    #[must_use]
    pub fn new(user_id: UserId) -> Self {
        Self {
            user_id,
            bigram_latencies: HashMap::new(),
            performance_index: 1.0,
        }
    }
}
