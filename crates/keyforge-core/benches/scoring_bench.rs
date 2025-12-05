// ===== keyforge/crates/keyforge-core/benches/scoring_bench.rs =====
use criterion::{criterion_group, criterion_main, Criterion};
use keyforge_core::config::{LayoutDefinitions, ScoringWeights};
use keyforge_core::geometry::{KeyNode, KeyboardGeometry};
use keyforge_core::optimizer::mutation;
use keyforge_core::scorer::loader::{CorpusBundle, RawCostData}; // Updated
use keyforge_core::scorer::ScorerBuildParams;
use std::hint::black_box;

fn setup_scorer() -> keyforge_core::scorer::Scorer {
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

    // Manually build corpus bundle for benchmark
    let mut bundle = CorpusBundle::default();
    let chars = "abcdefghijklmnopqrstuvwxyz.,";
    for c in chars.chars() {
        bundle.char_freqs[c as usize] = 1000.0;
    }
    // Mock some trigrams
    let char_vec: Vec<char> = chars.chars().collect();
    let mut count = 0;
    for &c1 in &char_vec {
        for &c2 in &char_vec {
            for &c3 in &char_vec {
                if count >= 3000 {
                    break;
                }
                bundle.trigrams.push((c1 as u8, c2 as u8, c3 as u8, 100.0));
                count += 1;
            }
        }
    }

    ScorerBuildParams::builder()
        .geometry(geom)
        .weights(ScoringWeights::default())
        .defs(LayoutDefinitions::default())
        .cost_data(RawCostData { entries: vec![] })
        .corpus(bundle)
        .debug(false)
        .build()
        .build_scorer()
        .expect("Failed to build scorer")
}

fn criterion_benchmark(c: &mut Criterion) {
    let scorer = setup_scorer();
    let layout_str = "qwertyuiopasdfghjkl;zxcvbnm,./";
    let layout_codes: Vec<u16> = layout_str.chars().map(|c| c as u16).collect();
    let pos_map = mutation::build_pos_map(&layout_codes);

    c.bench_function("score_full (3k trigrams)", |b| {
        b.iter(|| scorer.score_full(black_box(&pos_map), black_box(3000)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
