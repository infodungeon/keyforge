// libs/keyforge-compute/src/biometrics.rs

use keyforge_model::types::LatencyMs;
use keyforge_model::CostModel;
use keyforge_protocol::BiometricSample;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Statistics for a specific bigram.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct BigramStats {
    pub count: usize,
    pub mean: f64,
    pub m2: f64,
}

impl BigramStats {
    #[allow(clippy::cast_precision_loss)]
    pub fn add_sample(&mut self, ms: LatencyMs) {
        let ms_raw = ms.0;
        self.count += 1;
        let delta = ms_raw - self.mean;
        self.mean += delta / (self.count as f64);
        let delta2 = ms_raw - self.mean;
        self.m2 += delta * delta2;
    }

    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn variance(&self) -> f64 {
        if self.count < 2 {
            0.0
        } else {
            self.m2 / (self.count as f64)
        }
    }

    #[must_use]
    pub fn std_dev(&self) -> f64 {
        self.variance().sqrt()
    }
}

/// A streaming builder for generating cost profiles from biometric samples.
#[derive(Default, Debug)]
pub struct StreamingProfileBuilder {
    pub stats: HashMap<(u16, u16), BigramStats>,
    pub sample_count: usize,
}

impl StreamingProfileBuilder {
    pub const MIN_SAMPLES: usize = 5;
    pub const MAX_LATENCY_MS: f64 = 5000.0;

    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_sample(&mut self, sample: &BiometricSample) {
        let ms = LatencyMs(f64::from(sample.duration_ms));
        if ms.0 > 0.0 && ms.0 < Self::MAX_LATENCY_MS {
            self.stats
                .entry((sample.key_a, sample.key_b))
                .or_default()
                .add_sample(ms);
            self.sample_count += 1;
        }
    }

    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn build_model(&self) -> keyforge_model::CostModel {
        let mut modifiers = HashMap::new();
        let reliable_means: Vec<f64> = self
            .stats
            .values()
            .filter(|s| s.count >= Self::MIN_SAMPLES)
            .map(|s| s.mean)
            .collect();

        let global_avg = if reliable_means.is_empty() {
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
            if s.count >= Self::MIN_SAMPLES {
                let ratio = s.mean / global_avg;
                modifiers.insert(format!("{}_{}", bigram.0, bigram.1), ratio as f32);
            }
        }

        keyforge_model::CostModel {
            version: "2.0".into(),
            description: format!("Generated from {} biometric samples", self.sample_count),
            unit: "pts".into(),
            models: HashMap::new(),
            dynamic_rules: keyforge_model::cost_model::DynamicRules {
                sequence_modifiers: keyforge_model::cost_model::SequenceModifiers {
                    map: modifiers,
                },
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
    #[must_use]
    pub fn profile(samples: &[BiometricSample], base_model: &CostModel) -> CostModel {
        let mut builder = StreamingProfileBuilder::new();
        for s in samples {
            builder.add_sample(s);
        }

        let mut cost_model = base_model.clone();
        let generated = builder.build_model();

        for (bigram, modifier) in generated.dynamic_rules.sequence_modifiers.map {
            cost_model
                .dynamic_rules
                .sequence_modifiers
                .map
                .insert(bigram, modifier);
        }

        cost_model.description = format!(
            "{} (Personalized with {} samples)",
            cost_model.description, builder.sample_count
        );
        cost_model
    }
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;

    #[test]
    fn test_biometric_profiler_logic() {
        let base = CostModel::default();
        let model = BiometricProfiler::profile(&[], &base);
        assert_eq!(model.dynamic_rules().sequence_modifiers.map.len(), 0);

        let samples = vec![
            BiometricSample {
                key_a: 10,
                key_b: 11,
                duration_ms: 100
            };
            StreamingProfileBuilder::MIN_SAMPLES - 1
        ];
        let model = BiometricProfiler::profile(&samples, &base);
        assert_eq!(model.dynamic_rules().sequence_modifiers.map.len(), 0);

        let mut samples = Vec::new();
        for _ in 0..10 {
            samples.push(BiometricSample {
                key_a: 10,
                key_b: 11,
                duration_ms: 100,
            });
            samples.push(BiometricSample {
                key_a: 12,
                key_b: 13,
                duration_ms: 200,
            });
        }

        let model = BiometricProfiler::profile(&samples, &base);
        assert_eq!(model.dynamic_rules().sequence_modifiers.map.len(), 2);
    }
}
