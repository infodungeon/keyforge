// ===== keyforge/crates/keyforge-core/src/biometrics.rs =====
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
pub fn generate_cost_matrix_from_stats(store: &UserStatsStore) -> String {
    let mut sums: HashMap<String, f64> = HashMap::new();
    let mut counts: HashMap<String, usize> = HashMap::new();

    // 1. Aggregate Data
    for sample in &store.biometrics {
        if sample.bigram.len() != 2 { continue; }
        
        // Simple outlier filtering (e.g., pause > 2000ms is not a physical cost)
        if sample.ms > 2000.0 { continue; }

        let k = sample.bigram.to_lowercase();
        *sums.entry(k.clone()).or_default() += sample.ms;
        *counts.entry(k).or_default() += 1;
    }

    // 2. Build Output
    let mut output = String::from("From_Key,To_Key,Cost_MS,Confidence_Samples\n");

    for (bigram, sum) in sums {
        let count = counts[&bigram];
        if count < 3 { continue; } // Noise filter

        let avg_ms = sum / count as f64;
        
        let chars: Vec<char> = bigram.chars().collect();
        let k1 = char_to_key_id(chars[0]);
        let k2 = char_to_key_id(chars[1]);

        if let (Some(id1), Some(id2)) = (k1, k2) {
            output.push_str(&format!("{},{},{:.2},{}\n", id1, id2, avg_ms, count));
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
        _ => None,
    }
}