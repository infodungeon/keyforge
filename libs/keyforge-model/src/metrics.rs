// libs/keyforge-model/src/metrics.rs

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

//! Central registry for biomechanical metrics.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

/// Unique identifier for a biomechanical metric (e.g. "sfb", "`travel_dist`").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum MetricId {
    /// Total finger travel distance.
    TravelDistance,
    /// Same Finger Bigram cost.
    Sfb,
    /// Same Finger Bigram penalty contribution.
    SfbPenalty,
    /// Disjoint Same Finger Bigram (1-u skip).
    Dsfb,
    /// Lateral Stretch Bigram.
    Lsb,
    /// Home Finger Substitution (skipping home key).
    Hfs,
    /// Scissor cost.
    Scissor,
    /// Scissor penalty contribution.
    ScissorPenalty,
    /// Redirect cost.
    Redirect,
    /// Redirect penalty contribution.
    RedirectPenalty,
    /// Inward roll bonus/cost.
    RollIn,
    /// Outward roll bonus/cost.
    RollOut,
    /// Total roll penalty contribution.
    RollPenalty,
    /// Hand balance metric.
    HandBalance,
    /// Finger usage weighting.
    FingerUsage,
    /// Row distribution balance.
    RowBalance,
}

/// A collection of metric values.
#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
pub struct MetricSet {
    /// Map of metric ID to its calculated value.
    #[serde(default)]
    values: HashMap<MetricId, f32>,
}

impl MetricSet {
    /// Creates a new empty metric set.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the value for a specific metric, or 0.0 if not found.
    #[must_use]
    pub fn get(&self, id: MetricId) -> f32 {
        self.values.get(&id).copied().unwrap_or(0.0)
    }

    /// Sets the value for a specific metric.
    pub fn set(&mut self, id: MetricId, value: f32) {
        self.values.insert(id, value);
    }

    /// Adds the values from another metric set to this one.
    pub fn accumulate(&mut self, other: &MetricSet) {
        for (id, val) in &other.values {
            let current = self.values.entry(*id).or_insert(0.0);
            *current += val;
        }
    }

    /// Returns an iterator over the metric values.
    pub fn iter(&self) -> impl Iterator<Item = (&MetricId, &f32)> {
        self.values.iter()
    }

    /// Clears all metric values.
    pub fn clear(&mut self) {
        self.values.clear();
    }
}
