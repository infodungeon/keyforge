// crates/keyforge-hive/tests/fuzz_api.rs
use keyforge_protocol::config::ScoringWeights;
use keyforge_protocol::geometry::{KeyNode, KeyboardGeometry};
use keyforge_protocol::Validator;
use proptest::prelude::*;

// Strategy for generating random ScoringWeights
// We want to test edge cases: Negatives, Zero, Massive numbers, NaN
fn weights_strategy() -> impl Strategy<Value = ScoringWeights> {
    (
        any::<f32>(),   // penalty_sfb_base
        any::<f32>(),   // penalty_scissor
        any::<f32>(),   // penalty_redirect
        any::<usize>(), // loader_trigram_limit
    )
        .prop_map(|(sfb, scis, redir, limit)| ScoringWeights {
            penalty_sfb_base: sfb,
            penalty_scissor: scis,
            penalty_redirect: redir,
            loader_trigram_limit: limit,
            ..Default::default()
        })
}

// Strategy for generating random Geometry
fn geometry_strategy() -> impl Strategy<Value = KeyboardGeometry> {
    proptest::collection::vec(
        (
            any::<f32>(), // w
            any::<f32>(), // h
            0u8..10,      // hand (intentionally out of bounds 0-1)
            0u8..10,      // finger (intentionally out of bounds 0-4)
        ),
        0..300, // 0 to 300 keys (Max is 200)
    )
    .prop_map(|keys| {
        let nodes = keys
            .into_iter()
            .map(|(w, h, hand, finger)| KeyNode {
                w,
                h,
                hand: keyforge_model::types::HandIndex(hand),
                finger: keyforge_model::types::FingerIndex(finger),
                ..Default::default()
            })
            .collect();

        KeyboardGeometry {
            keys: nodes,
            ..Default::default()
        }
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn fuzz_weights_validation(w in weights_strategy()) {
        // The validator should return Ok or Err, but NEVER panic.
        let _ = w.validate();
    }

    #[test]
    fn fuzz_geometry_validation(g in geometry_strategy()) {
        // The validator should catch:
        // 1. Empty keys
        // 2. Too many keys (>200)
        // 3. Invalid dimensions (<=0)
        // 4. Invalid hand/finger indices
        let _ = g.validate();
    }

    #[test]
    fn fuzz_json_deserialization(s in "\\PC*") {
        // Feed random unicode garbage to the JSON parser for JobRequest
        // It should fail gracefully (Err), never crash the process.
        let _ = serde_json::from_str::<keyforge_protocol::JobRequest>(&s);
    }
}
