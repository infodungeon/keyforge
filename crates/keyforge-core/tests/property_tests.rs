// ===== keyforge/crates/keyforge-core/tests/property_tests.rs =====
use keyforge_core::config::{LayoutDefinitions, ScoringWeights};
use keyforge_core::geometry::{KeyNode, KeyboardGeometry};
use keyforge_core::optimizer::mutation;
use keyforge_core::scorer::ScorerBuildParams;
use proptest::prelude::*;
use tempfile::tempdir;

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
            w: 1.0,
            h: 1.0,
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
    #![proptest_config(ProptestConfig::with_cases(10))] // Reduce cases to avoid FS spam

    #[test]
    fn test_physics_scorer_stability(
        weights in arb_weights(),
        geom in arb_geometry()
    ) {
        let dir = tempdir().unwrap();
        let cost_path = dir.path().join("cost.csv");
        let corpus_dir = dir.path().join("corpus");
        std::fs::create_dir(&corpus_dir).unwrap();

        std::fs::write(&cost_path, "From,To,Cost\n").unwrap();
        std::fs::write(corpus_dir.join("1grams.csv"), "c,f\na,100\nb,100").unwrap();
        std::fs::write(corpus_dir.join("2grams.csv"), "c1,c2,f\na,b,50").unwrap();
        std::fs::write(corpus_dir.join("3grams.csv"), "c1,c2,c3,f\n").unwrap();

        let scorer_res = ScorerBuildParams::load_from_disk(
            cost_path,
            corpus_dir,
            geom.clone(),
            Some(weights),
            Some(LayoutDefinitions::default()),
            false
        );

        if let Ok(scorer) = scorer_res {
            let mut layout_codes = vec![0u16; scorer.key_count];
            if scorer.key_count > 0 { layout_codes[0] = b'a' as u16; }
            if scorer.key_count > 1 { layout_codes[1] = b'b' as u16; }

            let pos_map = mutation::build_pos_map(&layout_codes);
            let (score, _left, _total) = scorer.score_full(&pos_map, 100);

            prop_assert!(score.is_finite(), "Score was not finite: {}", score);
        }
    }
}
