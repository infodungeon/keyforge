use keyforge::config::{LayoutDefinitions, ScoringWeights};
use keyforge::geometry::{KeyNode, KeyboardGeometry};
use keyforge::optimizer::mutation;
use keyforge::scorer::ScorerBuilder;
use proptest::prelude::*;
use std::io::Cursor;

// --- STRATEGIES ---

// 1. Generate random weights (reasonable but wide ranges)
prop_compose! {
    fn arb_weights()(
        base in 100.0..1000.0f32,
        lat in 10.0..200.0f32,
        sfr in 10.0..100.0f32,
        scissor in 10.0..200.0f32
    ) -> ScoringWeights {
        ScoringWeights {
            penalty_sfb_base: base,
            penalty_sfb_lateral: lat,
            penalty_sfr_bad_row: sfr,
            penalty_scissor: scissor,
            ..Default::default()
        }
    }
}

// ... [Rest of file remains exactly the same] ...
// (arb_key_node, arb_geometry, proptest! block are unchanged)
prop_compose! {
    fn arb_key_node()(
        hand in 0u8..2,
        finger in 0u8..5,
        row in -5i8..5,
        col in -10i8..10,
        x in -20.0..20.0f32,
        y in -20.0..20.0f32,
        is_stretch in any::<bool>()
    ) -> KeyNode {
        KeyNode {
            id: "prop".to_string(),
            hand,
            finger,
            row,
            col,
            x,
            y,
            is_stretch
        }
    }
}

prop_compose! {
    fn arb_geometry()(
        keys in proptest::collection::vec(arb_key_node(), 1..50)
    ) -> KeyboardGeometry {
        let mut geom = KeyboardGeometry {
            keys,
            prime_slots: vec![],
            med_slots: vec![],
            low_slots: vec![],
            home_row: 1,
            finger_origins: [[(0.0, 0.0); 5]; 2],
        };
        geom.calculate_origins();
        geom
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    #[test]
    fn test_physics_scorer_stability(
        weights in arb_weights(),
        geom in arb_geometry()
    ) {
        let mut ngram_data = String::new();
        ngram_data.push_str("a\t100\n");
        ngram_data.push_str("b\t100\n");
        ngram_data.push_str("ab\t50\n");

        let cursor = Cursor::new(ngram_data);
        let defs = LayoutDefinitions::default();

        let scorer_res = ScorerBuilder::new()
            .with_weights(weights)
            .with_defs(defs)
            .with_geometry(geom.clone())
            .with_ngrams_from_reader(cursor);

        if let Ok(builder) = scorer_res {
             if let Ok(scorer) = builder.build() {
                let mut layout_bytes = vec![0u8; scorer.key_count];
                if scorer.key_count > 0 { layout_bytes[0] = b'a'; }
                if scorer.key_count > 1 { layout_bytes[1] = b'b'; }

                let pos_map = mutation::build_pos_map(&layout_bytes);
                let (score, _left, _total) = scorer.score_full(&pos_map, 100);

                prop_assert!(score.is_finite(), "Score was not finite: {}", score);
             }
        }
    }
}
