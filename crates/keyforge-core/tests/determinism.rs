use keyforge_core::config::{LayoutDefinitions, ScoringWeights};
use keyforge_core::geometry::{KeyNode, KeyboardGeometry};
use keyforge_core::optimizer::mutation;
use keyforge_core::scorer::loader::{CorpusBundle, RawCostData};
use keyforge_core::scorer::ScorerBuildParams;

fn setup_test_environment() -> (keyforge_core::scorer::Scorer, Vec<u16>) {
    let mut keys = Vec::new();
    for r in 0..3 {
        for c in 0..10 {
            keys.push(KeyNode {
                id: format!("k_{}_{}", r, c),
                hand: if c < 5 { 0 } else { 1 },
                finger: (c % 5) as u8,
                row: r as i8,
                col: c as i8,
                x: c as f32,
                y: r as f32,
                w: 1.0,
                h: 1.0,
                is_stretch: false,
            });
        }
    }

    let mut geom = KeyboardGeometry {
        keys,
        prime_slots: vec![13, 14, 15, 16],
        med_slots: vec![1, 2, 3, 4],
        low_slots: vec![20, 21, 22],
        home_row: 1,
        finger_origins: [[(0.0, 0.0); 5]; 2],
    };
    geom.calculate_origins();

    let mut bundle = CorpusBundle::default();
    let chars = "abcdefghijklmnopqrstuvwxyz.,";
    for (i, c) in chars.chars().enumerate() {
        bundle.char_freqs[c as usize] = (1000 - i * 10) as f32;
    }
    // Add deterministic bigrams
    let mut count = 0;
    for c1 in chars.chars() {
        for c2 in chars.chars() {
            if count > 200 {
                break;
            }
            bundle.bigrams.push((c1 as u8, c2 as u8, 100.0));
            count += 1;
        }
    }

    let scorer = ScorerBuildParams::builder()
        .geometry(geom)
        .weights(ScoringWeights::default())
        .defs(LayoutDefinitions::default())
        .cost_data(RawCostData { entries: vec![] })
        .corpus(bundle)
        .debug(false)
        .build()
        .build_scorer()
        .expect("Failed to build scorer");

    let layout_str = "qwertyuiopasdfghjkl;zxcvbnm,./";
    let layout: Vec<u16> = layout_str.chars().map(|c| c as u16).collect();

    (scorer, layout)
}

#[test]
fn test_scorer_determinism() {
    println!("\n=== TEST: Scorer Determinism (Run A vs Run B) ===");
    let (scorer, layout) = setup_test_environment();
    let pos_map = mutation::build_pos_map(&layout);

    // Run A
    let (score_a, _, _) = scorer.score_full(&pos_map, 1000);
    let details_a = scorer.score_details(&pos_map, 1000);

    // Run B
    let (score_b, _, _) = scorer.score_full(&pos_map, 1000);
    let details_b = scorer.score_details(&pos_map, 1000);

    println!(
        "Run A: Total={:.2} | SFB={:.2}",
        score_a, details_a.mech_sfb
    );
    println!(
        "Run B: Total={:.2} | SFB={:.2}",
        score_b, details_b.mech_sfb
    );

    assert_eq!(score_a, score_b, "Scores drifted!");
    println!("✅ Determinism Verified.");
}
