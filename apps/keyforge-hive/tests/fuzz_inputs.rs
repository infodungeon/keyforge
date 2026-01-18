// apps/keyforge-hive/tests/fuzz_inputs.rs

//! # Input Fuzzing Tests for KeyForge
//!
//! Property-based testing for data structures and validation logic.


use keyforge_model::config::ScoringWeights;
use keyforge_model::geometry::{KeyNode, KeyboardGeometry};
use keyforge_model::Validator;
use proptest::prelude::*;

/// Generates randomized `ScoringWeights` for validation fuzzing.
fn weights_strategy() -> impl Strategy<Value = ScoringWeights> {
    (any::<f32>(), any::<f32>(), any::<f32>(), any::<usize>())
        .prop_map(|(sfb, scis, redir, limit)| {
            let mut w = ScoringWeights::default();
            w.weights.insert("penalty_sfb_base".to_string(), sfb);
            w.weights.insert("penalty_scissor".to_string(), scis);
            w.weights.insert("penalty_redirect".to_string(), redir);
            w.weights.insert("loader_trigram_limit".to_string(), limit as f32);
            w
        })
}

/// Generates arbitrary `KeyboardGeometry` configurations for validation fuzzing.
fn geometry_strategy() -> impl Strategy<Value = KeyboardGeometry> {
    proptest::collection::vec(
        (any::<f32>(), any::<f32>(), 0u8..10, 0u8..10),
        0..300,
    )
    .prop_map(|keys| {
        let nodes = keys.into_iter().map(|(w, h, hand, finger)| KeyNode {
            w, h, hand: keyforge_model::types::HandIndex(hand.min(1)), 
            finger: keyforge_model::types::FingerIndex(finger.min(4)), 
            ..Default::default()
        }).collect();
        KeyboardGeometry { keys: nodes, ..Default::default() }
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Verifies that `ScoringWeights` validation handles randomized inputs.
    #[test]
    fn fuzz_weights_validation(w in weights_strategy()) {
        let _ = w.validate();
    }

    /// Verifies that `KeyboardGeometry` validation handles arbitrary configurations.
    #[test]
    fn fuzz_geometry_validation(g in geometry_strategy()) {
        let _ = g.validate();
    }

    /// Verifies that `JobRequest` deserialization is stable against arbitrary strings.
    #[test]
    fn fuzz_json_deserialization(s in "\\PC*") {
        let _ = serde_json::from_str::<keyforge_protocol::JobRequest>(&s);
    }
}