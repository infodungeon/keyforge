// libs/keyforge-compute/src/biometrics.rs

use keyforge_model::CostModel;
use keyforge_protocol::BiometricSample;
use std::collections::HashMap;

/// Aggregates raw biometric timing data into a personalized Physics Cost Model.
pub struct BiometricProfiler;

impl BiometricProfiler {
    /// Transforms a set of biometric samples into a CostModel.
    /// 
    /// This implementation calculates average latencies per bigram and uses them
    /// to build sequence-specific modifiers in the dynamic rules of the cost model.
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

        // 2. Map to sequence modifiers
        // We normalize latencies relative to a "standard" speed (e.g. 150ms)
        // Latency / 150.0 * 100.0 gives us an effort point value.
        for (bigram, (total_ms, count)) in totals {
            if count < 5 { continue; } // Statistical Significance Threshold
            
            let avg = total_ms / (count as f64);
            let effort = (avg / 150.0) * 100.0;
            
            model.dynamic_rules.sequence_modifiers.insert(bigram, effort as f32);
        }

        model
    }
}
