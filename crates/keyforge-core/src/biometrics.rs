use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BiometricSample {
    pub bigram: String,
    pub ms: f64,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct UserStatsStore {
    pub sessions: u64,
    pub total_keystrokes: u64,
    pub biometrics: Vec<BiometricSample>,
}

/// Analyzes biometric samples and generates a Cost Matrix CSV content.
/// Implements Interquartile Range (IQR) filtering to remove outliers (pauses/interruptions).
pub fn generate_cost_matrix_from_stats(store: &UserStatsStore) -> String {
    let mut buckets: HashMap<String, Vec<f64>> = HashMap::new();

    // 1. Group Data
    for sample in &store.biometrics {
        if sample.bigram.len() != 2 {
            continue;
        }
        // Sanity Check: < 10ms is likely a sensor error, > 5000ms is definitely a break
        if sample.ms < 10.0 || sample.ms > 5000.0 {
            continue;
        }
        buckets
            .entry(sample.bigram.to_lowercase())
            .or_default()
            .push(sample.ms);
    }

    let mut output = String::from("From_Key,To_Key,Cost_MS,Confidence_Samples\n");

    for (bigram, mut times) in buckets {
        let raw_count = times.len();
        if raw_count < 5 {
            continue; // Not enough statistical significance
        }

        // 2. Sort for Quartile Analysis
        times.sort_by(|a, b| a.partial_cmp(b).unwrap());

        // 3. IQR Filtering
        // If we have enough data (>= 20), apply IQR. Otherwise just use median.
        let avg_ms = if raw_count >= 20 {
            let q1_idx = raw_count / 4;
            let q3_idx = raw_count * 3 / 4;

            let q1 = times[q1_idx];
            let q3 = times[q3_idx];
            let iqr = q3 - q1;

            // Standard outlier definition: Q1 - 1.5*IQR  to  Q3 + 1.5*IQR
            // We only care about upper bound (slowdowns) mostly, but lower bound catches glitches
            let low_fence = q1 - (1.5 * iqr);
            let high_fence = q3 + (1.5 * iqr);

            let clean_values: Vec<f64> = times
                .iter()
                .filter(|&&v| v >= low_fence && v <= high_fence)
                .cloned()
                .collect();

            if clean_values.is_empty() {
                // Should rare happen, fallback to median
                times[raw_count / 2]
            } else {
                let sum: f64 = clean_values.iter().sum();
                sum / clean_values.len() as f64
            }
        } else {
            // Small sample size: Trim min/max and average
            let sum: f64 = times[1..raw_count - 1].iter().sum();
            sum / (raw_count - 2) as f64
        };

        let chars: Vec<char> = bigram.chars().collect();
        let k1 = char_to_key_id(chars[0]);
        let k2 = char_to_key_id(chars[1]);

        if let (Some(id1), Some(id2)) = (k1, k2) {
            output.push_str(&format!("{},{},{:.2},{}\n", id1, id2, avg_ms, raw_count));
        }
    }

    output
}

fn char_to_key_id(c: char) -> Option<String> {
    match c {
        'a'..='z' => Some(format!("Key{}", c.to_ascii_uppercase())),
        '0'..='9' => Some(format!("Digit{}", c)),
        ' ' => Some("Space".to_string()),
        '.' => Some("Period".to_string()),
        ',' => Some("Comma".to_string()),
        ';' => Some("Semicolon".to_string()),
        '/' => Some("Slash".to_string()),
        '[' => Some("BracketLeft".to_string()),
        ']' => Some("BracketRight".to_string()),
        '-' => Some("Minus".to_string()),
        '=' => Some("Equal".to_string()),
        '\'' => Some("Quote".to_string()),
        _ => None,
    }
}
