// libs/keyforge-compute/src/biometrics.rs

use keyforge_model::CostModel;
use keyforge_protocol::BiometricSample;
use std::collections::HashMap;

/// Aggregates raw biometric timing data into a personalized Physics Cost Model.
#[derive(Debug)]
pub struct BiometricProfiler;

impl BiometricProfiler {
    /// Transforms a set of biometric samples into a `CostModel`.
    ///
    /// This implementation calculates average latencies per bigram and uses them
    /// to build sequence-specific modifiers in the dynamic rules of the cost model.
    #[must_use]
    pub fn profile(samples: &[BiometricSample], base_model: &CostModel) -> CostModel {
        let mut model = base_model.clone();

        if samples.is_empty() {
            return model;
        }

        // 1. Group by bigram and calculate averages
        let mut totals: HashMap<String, (f64, u32)> = HashMap::new();
        for s in samples {
            let entry = totals.entry(s.bigram.clone()).or_insert((0.0, 0));
            entry.0 += s.ms;
            entry.1 += 1;
        }

        // 2. Calculate Median Latency for Relative Normalization
        let mut all_avgs: Vec<f64> = totals
            .values()
            .map(|(sum, count)| sum / f64::from(*count))
            .collect();
        if all_avgs.is_empty() {
            return model;
        }
        all_avgs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let median_latency = all_avgs[all_avgs.len() / 2];

        // 3. Map to sequence modifiers
        // Use median_latency * 0.66 as the "100 effort points" baseline
        // This ensures the model is centered around the user's typical speed.
        let effort_baseline = median_latency.max(50.0); // Floor to prevent extreme scores

        for (bigram, (total_ms, count)) in totals {
            if count < 5 {
                continue;
            } // Statistical Significance Threshold

            let avg = total_ms / f64::from(count);
            #[allow(clippy::cast_possible_truncation)]
            let effort_f32 = (avg / effort_baseline * 100.0) as f32;
            model
                .dynamic_rules
                .sequence_modifiers
                .insert(bigram, effort_f32);
        }

        model
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_biometric_profiler_logic() {
        let base = CostModel::default();

        // 1. Empty samples returns base
        let model = BiometricProfiler::profile(&[], &base);
        assert_eq!(model.dynamic_rules.sequence_modifiers.len(), 0);

        // 2. Below threshold (5 samples)
        let samples = vec![
            BiometricSample {
                bigram: "th".into(),
                ms: 100.0,
                timestamp: 0
            };
            4
        ];
        let model = BiometricProfiler::profile(&samples, &base);
        assert_eq!(model.dynamic_rules.sequence_modifiers.len(), 0);

        // 3. Above threshold
        let samples = vec![
            BiometricSample {
                bigram: "th".into(),
                ms: 150.0,
                timestamp: 0
            };
            10
        ];
        let model = BiometricProfiler::profile(&samples, &base);
        assert_eq!(model.dynamic_rules.sequence_modifiers.len(), 1);
        // 150ms / 150.0 * 100.0 = 100.0
        assert_eq!(
            *model.dynamic_rules.sequence_modifiers.get("th").unwrap(),
            100.0
        );
    }
}
