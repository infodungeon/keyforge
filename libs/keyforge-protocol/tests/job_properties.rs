use keyforge_protocol::config::{ScoringWeights, SearchParams};
use keyforge_protocol::geometry::{KeyNode, KeyboardGeometry};
use keyforge_protocol::job::JobIdentifier;
use keyforge_protocol::{CostMatrixSource, KeyConstraint};
use keyforge_protocol::types::{KeyIndex, HandIndex, FingerIndex};
use proptest::prelude::*;

// --- Strategies ---

fn arb_geometry() -> impl Strategy<Value = KeyboardGeometry> {
    prop::collection::vec(
        (any::<f32>(), any::<f32>(), 0u8..2, 0u8..5),
        1..50, // 1 to 50 keys
    )
    .prop_map(|keys| {
        let nodes = keys
            .into_iter()
            .enumerate()
            .map(|(i, (x, y, h, f))| KeyNode {
                index: i,
                label: format!("k{}", i),
                x,
                y,
                hand: HandIndex(h),
                finger: FingerIndex(f),
                ..Default::default()
            })
            .collect();
        KeyboardGeometry {
            keys: nodes,
            prime_slots: vec![],
            med_slots: vec![],
            low_slots: vec![],
            home_row: 1,
        }
    })
}

fn arb_weights() -> impl Strategy<Value = ScoringWeights> {
    (any::<f32>(), any::<f32>()).prop_map(|(sfb, scis)| ScoringWeights {
        penalty_sfb_base: sfb,
        penalty_scissor: scis,
        ..Default::default()
    })
}

fn arb_params() -> impl Strategy<Value = SearchParams> {
    (1usize..1000, 1usize..1000).prop_map(|(epochs, steps)| SearchParams {
        search_epochs: epochs,
        search_steps: steps,
        ..Default::default()
    })
}

fn arb_constraints() -> impl Strategy<Value = Vec<KeyConstraint>> {
    prop::collection::vec((any::<u16>(), "[a-zA-Z0-9]+"), 0..10).prop_map(|vec| {
        vec.into_iter()
            .map(|(idx, key)| KeyConstraint { index: KeyIndex(idx), key })
            .collect()
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn test_job_id_determinism(
        geo in arb_geometry(),
        weights in arb_weights(),
        params in arb_params(),
        pins in arb_constraints(),
        corpus_name in "[a-z0-9_]+",
    ) {
        let cost = CostMatrixSource::default();

        let id1 = JobIdentifier::try_from_parts(&geo, &weights, &params, &pins, &corpus_name, &cost).unwrap();
        let id2 = JobIdentifier::try_from_parts(&geo, &weights, &params, &pins, &corpus_name, &cost).unwrap();

        prop_assert_eq!(id1.hash, id2.hash);
    }

    #[test]
    fn test_job_id_sensitivity_geometry(
        geo in arb_geometry(),
        weights in arb_weights(),
        params in arb_params(),
        pins in arb_constraints(),
        corpus_name in "[a-z0-9_]+",
    ) {
        let cost = CostMatrixSource::default();
        let base_hash = JobIdentifier::try_from_parts(&geo, &weights, &params, &pins, &corpus_name, &cost).unwrap().hash;

        // Mutate geometry slightly
        let mut geo2 = geo.clone();
        if let Some(k) = geo2.keys.first_mut() {
            // Ensure mutation changes the value regardless of float precision
            k.x = if k.x == 0.0 { 1.0 } else { 0.0 };
        }

        let new_hash = JobIdentifier::try_from_parts(&geo2, &weights, &params, &pins, &corpus_name, &cost).unwrap().hash;
        prop_assert_ne!(base_hash, new_hash);
    }
}
