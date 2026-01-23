// libs/keyforge-compute/src/biometrics.rs

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

//! Biometric profiling logic for personalizing cost models.

use keyforge_model::CostModel;
use keyforge_protocol::BiometricSample;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Statistics for a specific bigram.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct BigramStats {
    /// Number of samples collected.
    pub count: usize,
    /// Arithmetic mean of latency (ms).
    pub mean: f64,
    /// Sum of squared differences from the mean (for variance calculation).
    pub m2: f64,
}

impl BigramStats {
    /// Updates the stats with a new sample using Welford's online algorithm.
    pub fn add_sample(&mut self, ms: f64) {
        self.count += 1;
        let delta = ms - self.mean;
        #[allow(clippy::cast_precision_loss)]
        {
            self.mean += delta / self.count as f64;
        }
        let delta2 = ms - self.mean;
        self.m2 += delta * delta2;
    }

    /// Returns the variance of the samples.
    #[must_use]
    pub fn variance(&self) -> f64 {
        if self.count < 2 {
            0.0
        } else {
            #[allow(clippy::cast_precision_loss)]
            {
                self.m2 / self.count as f64
            }
        }
    }

    /// Returns the standard deviation of the samples.
    #[must_use]
    pub fn std_dev(&self) -> f64 {
        self.variance().sqrt()
    }
}

/// A streaming builder for generating cost profiles from biometric samples.
/// This allows processing large datasets without loading everything into memory.
#[derive(Default, Debug)]
pub struct StreamingProfileBuilder {
    /// Map of bigram strings to their running statistics.
    pub stats: HashMap<String, BigramStats>,
    /// Total number of samples processed.
    pub sample_count: usize,
}

impl StreamingProfileBuilder {
    /// Minimum samples required for a bigram to be included in the profile.
    pub const MIN_SAMPLES: usize = 5;
    /// Outlier threshold: ignore latencies above 5 seconds.
    pub const MAX_LATENCY_MS: f64 = 5000.0;

    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a single biometric sample to the running statistics.
    pub fn add_sample(&mut self, sample: &BiometricSample) {
        // Outlier detection: ignore extreme latencies
        if sample.ms > 0.0 && sample.ms < Self::MAX_LATENCY_MS {
            self.stats
                .entry(sample.bigram.clone())
                .or_default()
                .add_sample(sample.ms);
            self.sample_count += 1;
        }
    }

    /// Generates a `CostModel` from the accumulated statistics.
    #[must_use]
    pub fn build_model(&self) -> CostModel {
        let mut modifiers = HashMap::new();

        // Task-phys-rev-042: Use Median for Global Baseline to resist outliers.
        // We only consider bigrams with sufficient samples for the baseline.
        let reliable_means: Vec<f64> = self
            .stats
            .values()
            .filter(|s| s.count >= Self::MIN_SAMPLES)
            .map(|s| s.mean)
            .collect();

        let global_avg = if reliable_means.is_empty() {
            // Fallback to all samples if none meet MIN_SAMPLES
            let all_means: Vec<f64> = self.stats.values().map(|s| s.mean).collect();
            if all_means.is_empty() {
                1.0
            } else {
                Self::calculate_median(all_means)
            }
        } else {
            Self::calculate_median(reliable_means)
        };

        for (bigram, s) in &self.stats {
            // Only use bigrams with sufficient sample size
            if s.count >= Self::MIN_SAMPLES {
                // Ratio relative to average: > 1.0 means slower than average
                let ratio = s.mean / global_avg;
                #[allow(clippy::cast_possible_truncation)]
                {
                    modifiers.insert(bigram.clone(), ratio as f32);
                }
            }
        }

        CostModel {
            meta: keyforge_model::cost_model::CostModelMeta {
                version: "2.0".into(),
                description: format!("Generated from {} biometric samples", self.sample_count),
                unit: "pts".into(),
            },
            models: HashMap::new(),
            dynamic_rules: keyforge_model::cost_model::DynamicRules {
                sequence_modifiers: modifiers,
                penalties: HashMap::new(),
                constraints: HashMap::new(),
            },
        }
    }

    fn calculate_median(mut values: Vec<f64>) -> f64 {
        if values.is_empty() {
            return 1.0;
        }
        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mid = values.len() / 2;
        #[allow(clippy::manual_is_multiple_of, clippy::manual_midpoint)]
        if values.len() % 2 == 0 {
            (values[mid - 1] + values[mid]) / 2.0
        } else {
            values[mid]
        }
    }
}

/// Aggregates raw biometric timing data into a personalized Physics Cost Model.
#[derive(Debug)]
pub struct BiometricProfiler;

impl BiometricProfiler {
    /// Transforms a set of biometric samples into a `CostModel` based on a template.
    #[must_use]
    pub fn profile(samples: &[BiometricSample], base_model: &CostModel) -> CostModel {
        let mut builder = StreamingProfileBuilder::new();
        for s in samples {
            builder.add_sample(s);
        }

        let mut result = base_model.clone();
        let generated = builder.build_model();

        // Merge sequence modifiers
        for (bigram, modifier) in generated.dynamic_rules.sequence_modifiers {
            result
                .dynamic_rules
                .sequence_modifiers
                .insert(bigram, modifier);
        }

        // Update description to reflect personalization
        result.meta.description = format!(
            "{} (Personalized with {} samples)",
            result.meta.description, builder.sample_count
        );

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_biometric_profiler_logic() {
        let base = CostModel::default();

        // 1. Empty samples returns default model
        let model = BiometricProfiler::profile(&[], &base);
        assert_eq!(model.dynamic_rules.sequence_modifiers.len(), 0);

        // 2. Below threshold
        let samples = vec![
            BiometricSample {
                bigram: "th".into(),
                ms: 100.0,
                timestamp: 0
            };
            StreamingProfileBuilder::MIN_SAMPLES - 1
        ];
        let model = BiometricProfiler::profile(&samples, &base);
        assert_eq!(model.dynamic_rules.sequence_modifiers.len(), 0);

        // 3. Above threshold
        let mut samples = Vec::new();
        for _ in 0..10 {
            samples.push(BiometricSample {
                bigram: "th".into(),
                ms: 100.0,
                timestamp: 0,
            });
            samples.push(BiometricSample {
                bigram: "he".into(),
                ms: 200.0,
                timestamp: 0,
            });
        }

        let model = BiometricProfiler::profile(&samples, &base);
        assert_eq!(model.dynamic_rules.sequence_modifiers.len(), 2);

        // Avg = (100 + 200) / 2 = 150
        // "th" ratio = 100 / 150 = 0.666...
        // "he" ratio = 200 / 150 = 1.333...
        let th_val = *model.dynamic_rules.sequence_modifiers.get("th").unwrap();
        let he_val = *model.dynamic_rules.sequence_modifiers.get("he").unwrap();

        assert!(th_val < 1.0);
        assert!(he_val > 1.0);
    }

    #[test]
    fn test_median_robustness() {
        let mut builder = StreamingProfileBuilder::new();

        // Add several normal bigrams
        for i in 1..=5 {
            for _ in 0..10 {
                builder.add_sample(&BiometricSample {
                    bigram: format!("b{}", i),
                    ms: 100.0,
                    timestamp: 0,
                });
            }
        }

        // Add one extreme outlier bigram
        for _ in 0..10 {
            builder.add_sample(&BiometricSample {
                bigram: "outlier".into(),
                ms: 4000.0,
                timestamp: 0,
            });
        }

        let model = builder.build_model();

        // Global baseline should be around 100.0 (the median of [100, 100, 100, 100, 100, 4000])
        // If it was arithmetic mean, it would be (500 + 4000) / 6 = 750

        let b1_val = *model.dynamic_rules.sequence_modifiers.get("b1").unwrap();
        // If baseline is 100, b1_val should be 1.0
        // If baseline was 750, b1_val would be 100/750 = 0.13

        assert!((b1_val - 1.0).abs() < 0.1);
    }
}
