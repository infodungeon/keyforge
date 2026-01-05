// Copyright (c) 2025 KeyForge Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
use keyforge_model::{Corpus, KeyNode, Keyboard, Layout, Rubric, types::{HandIndex, FingerIndex, RowIndex, ColIndex, KeyCode}};
use keyforge_physics::ScoringEngine;

fn setup_kb() -> Keyboard {
    let keys: Vec<KeyNode> = (0..5)
        .map(|i| KeyNode {
            index: i,
            label: format!("k{}", i),
            hand: HandIndex(0),
            finger: FingerIndex(i as u8),
            row: RowIndex(0),
            col: ColIndex(i as i8),
            x: i as f32,
            y: 0.0,
            is_home: true,
            ..Default::default()
        })
        .collect();
    Keyboard::new(keys, 0).unwrap()
}

#[test]
fn test_penalty_map_population() {
    let kb = setup_kb();
    let mut corpus = Corpus::default();
    
    // Add frequency for 'a' (97) and 'b' (98)
    corpus.char_freqs[97] = 1000;
    corpus.char_freqs[98] = 1000;
    
    // Add bigram cost
    corpus.bigrams.push((97, 98, 500));

    let rubric = Rubric::default();
    let engine = ScoringEngine::new(&kb, &corpus, &rubric, &[]).unwrap();
    
    let layout = Layout::new_unchecked(vec![KeyCode(97), KeyCode(98), KeyCode(99), KeyCode(100), KeyCode(101)]);
    
    let report = engine.analyze(&layout).unwrap();
    
    // Check Heatmap (Frequency)
    assert!(report.heatmap[0] > 0.0, "Key 0 should have frequency");
    assert!(report.heatmap[1] > 0.0, "Key 1 should have frequency");
    
    // Check Penalty Map (Effort)
    // Even without bigrams, base cost should be non-zero due to monogram loop
    assert!(report.penalty_map[0] > 0.0, "Key 0 should have penalty");
    assert!(report.penalty_map[1] > 0.0, "Key 1 should have penalty");
    
    // Key 2 has 0 frequency, should have 0 penalty
    assert_eq!(report.penalty_map[2], 0.0);
}

#[test]
fn test_szr35_penalty_map() {
    // Simulate SZR35 Geometry (simplified)
    let keys: Vec<KeyNode> = (0..36)
        .map(|i| KeyNode {
            index: i,
            label: format!("k{}", i),
            hand: if i < 18 { HandIndex(0) } else { HandIndex(1) },
            finger: FingerIndex((i % 5) as u8),
            row: RowIndex((i / 10) as i8),
            col: ColIndex((i % 10) as i8),
            x: i as f32,
            y: 0.0,
            is_home: false,
            ..Default::default()
        })
        .collect();
    let kb = Keyboard::new(keys, 1).unwrap();

    let mut corpus = Corpus::default();
    // 'o' = 111 (Lowercase ASCII)
    corpus.char_freqs[111] = 1000;
    // '.' = 46
    corpus.char_freqs[46] = 500;
    
    let rubric = Rubric::default();
    let engine = ScoringEngine::new(&kb, &corpus, &rubric, &[]).unwrap();
    
    // Layout with KC_O (111 normalized) and KC_DOT (46)
    // KC_O is 79 in JSON, normalized to 111 by Registry.
    // KC_DOT is 46.
    
    let mut layout_vec = vec![KeyCode(0); 36];
    layout_vec[27] = KeyCode(111); // KC_O position
    layout_vec[31] = KeyCode(46);  // KC_DOT position
    
    let layout = Layout::new_unchecked(layout_vec);
    
    let report = engine.analyze(&layout).unwrap();
    
    // Check Heatmap (Frequency)
    assert!(report.heatmap[27] > 0.0, "KC_O (idx 27) should have frequency");
    assert!(report.heatmap[31] > 0.0, "KC_DOT (idx 31) should have frequency");
    
    // Check Penalty Map (Effort)
    assert!(report.penalty_map[27] > 0.0, "KC_O (idx 27) should have penalty");
    assert!(report.penalty_map[31] > 0.0, "KC_DOT (idx 31) should have penalty");
}
