use criterion::{criterion_group, criterion_main, Criterion};
use keyforge_core::config::{LayoutDefinitions, ScoringWeights};
use keyforge_core::geometry::{KeyNode, KeyboardGeometry};
use keyforge_core::optimizer::mutation;
use keyforge_core::scorer::ScorerBuilder;
use std::hint::black_box;
use std::io::Cursor;

fn setup_scorer() -> keyforge_core::scorer::Scorer {
    let mut keys = Vec::new();
    for r in 0..3 {
        for c in 0..10 {
            keys.push(KeyNode {
                id: format!("k_{}_{}", r, c),
                hand: if c < 5 { 0 } else { 1 },
                finger: match c {
                    0 | 9 => 4,
                    1 | 8 => 3,
                    2 | 7 => 2,
                    _ => 1,
                },
                row: r as i8,
                col: c as i8,
                x: c as f32,
                y: r as f32,
                w: 1.0,
                h: 1.0,
                is_stretch: c == 4 || c == 5,
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

    let mut ngram_data = String::new();
    let chars = "abcdefghijklmnopqrstuvwxyz.,";
    for c in chars.chars() {
        ngram_data.push_str(&format!("{}\t1000\n", c));
    }
    for i in 0..chars.len() - 1 {
        let b = &chars[i..i + 2];
        ngram_data.push_str(&format!("{}\t500\n", b));
    }

    let mut count = 0;
    let char_vec: Vec<char> = chars.chars().collect();
    for &c1 in &char_vec {
        for &c2 in &char_vec {
            for &c3 in &char_vec {
                if count >= 3000 {
                    break;
                }
                ngram_data.push_str(&format!("{}{}{}\t100\n", c1, c2, c3));
                count += 1;
            }
        }
    }

    let cursor = Cursor::new(ngram_data);
    let weights = ScoringWeights::default();
    let defs = LayoutDefinitions::default();

    ScorerBuilder::new()
        .with_weights(weights)
        .with_defs(defs)
        .with_geometry(geom)
        .with_ngrams_from_reader(cursor)
        .expect("Failed to load ngrams")
        .build()
        .expect("Failed to build scorer")
}

fn criterion_benchmark(c: &mut Criterion) {
    let scorer = setup_scorer();

    let layout_str = "qwertyuiopasdfghjkl;zxcvbnm,./";
    // FIXED: Convert to Vec<u16>
    let layout_codes: Vec<u16> = layout_str.chars().map(|c| c as u16).collect();

    // FIXED: pos_map is now Box<[u8; 65536]>
    let pos_map = mutation::build_pos_map(&layout_codes);

    c.bench_function("score_full (3k trigrams)", |b| {
        b.iter(|| scorer.score_full(black_box(&pos_map), black_box(3000)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
