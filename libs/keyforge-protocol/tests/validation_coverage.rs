use keyforge_protocol::config::{ScoringWeights, SearchParams};
use keyforge_protocol::geometry::{KeyNode, KeyboardGeometry};
use keyforge_protocol::types::{KeyIndex, HandIndex, FingerIndex};
use keyforge_protocol::Validator;
use proptest::prelude::*;
use std::collections::HashSet;

// --- Strategies ---

fn arb_weights() -> impl Strategy<Value = ScoringWeights> {
    (
        any::<f32>(), // penalty_sfb_base
        any::<f32>(), // penalty_scissor
        any::<f32>(), // penalty_redirect
        any::<usize>(), // loader_trigram_limit
    ).prop_map(|(sfb, scis, redir, limit)| {
        let mut w = ScoringWeights::default();
        w.penalty_sfb_base = sfb;
        w.penalty_scissor = scis;
        w.penalty_redirect = redir;
        w.loader_trigram_limit = limit;
        w
    })
}

fn arb_params() -> impl Strategy<Value = SearchParams> {
    (
        any::<usize>(), // epochs
        any::<usize>(), // steps
        any::<f32>(),   // temp_min
        any::<f32>(),   // temp_max
    ).prop_map(|(epochs, steps, min, max)| {
        let mut p = SearchParams::default();
        p.search_epochs = epochs;
        p.search_steps = steps;
        p.temp_min = min;
        p.temp_max = max;
        p
    })
}

fn arb_geometry() -> impl Strategy<Value = KeyboardGeometry> {
    (
        prop::collection::vec(any::<u8>(), 0..10), // keys (just count matters mostly for slots)
        prop::collection::vec(any::<usize>(), 0..5), // prime
        prop::collection::vec(any::<usize>(), 0..5), // med
        prop::collection::vec(any::<usize>(), 0..5), // low
    ).prop_map(|(keys_hand, prime, med, low)| {
        let keys = keys_hand.into_iter().map(|h| KeyNode {
            hand: HandIndex(h % 2),
            finger: FingerIndex(1),
            w: 1.0, h: 1.0,
            ..Default::default()
        }).collect();
        
        KeyboardGeometry {
            keys,
            prime_slots: prime.into_iter().map(KeyIndex::from).collect(),
            med_slots: med.into_iter().map(KeyIndex::from).collect(),
            low_slots: low.into_iter().map(KeyIndex::from).collect(),
            home_row: 1,
        }
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn test_weights_validation_properties(w in arb_weights()) {
        let res = w.validate();
        
        if res.is_ok() {
            prop_assert!(w.penalty_sfb_base >= 0.0);
            prop_assert!(w.penalty_scissor >= 0.0);
            prop_assert!(w.penalty_sfb_base <= 100_000_000.0);
            prop_assert!(w.loader_trigram_limit <= 50_000);
        }
    }

    #[test]
    fn test_params_validation_properties(p in arb_params()) {
        let res = p.validate();
        
        if res.is_ok() {
            prop_assert!(p.search_epochs > 0);
            prop_assert!(p.search_epochs <= 1_000_000);
            prop_assert!(p.search_steps > 0);
            prop_assert!(p.temp_min >= 0.0001);
            prop_assert!(p.temp_max <= 1000.0);
            prop_assert!(p.temp_min < p.temp_max);
        }
    }
    
    #[test]
    fn test_geometry_validation_properties(g in arb_geometry()) {
        let res = g.validate();
        
        if res.is_ok() {
            prop_assert!(!g.keys.is_empty());
            
            let prime: HashSet<_> = g.prime_slots.iter().collect();
            let med: HashSet<_> = g.med_slots.iter().collect();
            let low: HashSet<_> = g.low_slots.iter().collect();
            
            prop_assert!(prime.is_disjoint(&med));
            prop_assert!(prime.is_disjoint(&low));
            prop_assert!(med.is_disjoint(&low));
            
            let len = g.keys.len();
            for idx in g.prime_slots.iter().chain(&g.med_slots).chain(&g.low_slots) {
                prop_assert!(usize::from(*idx) < len);
            }
        }
    }
}
