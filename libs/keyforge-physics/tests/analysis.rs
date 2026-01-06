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

use keyforge_model::{
    Corpus, KeyNode, Keyboard, Layout, Rubric, 
    types::{HandIndex, FingerIndex, RowIndex, ColIndex, KeyCode, SpaceHandPreference}
};
use keyforge_physics::ScoringEngine;

fn setup_kb(size: usize) -> Keyboard {
    let keys: Vec<KeyNode> = (0..size)
        .map(|i| KeyNode {
            index: i,
            label: format!("k{}", i),
            hand: HandIndex((i % 2) as u8),
            finger: FingerIndex((i % 5) as u8),
            row: RowIndex((i / 10) as i8),
            col: ColIndex((i % 10) as i8),
            x: (i % 10) as f32,
            y: (i / 10) as f32,
            is_home: false,
            ..Default::default()
        })
        .collect();
    Keyboard::new(keys, 1).unwrap()
}

#[test]
fn test_metric_detection_sfb_scissors() {
    // Manually construct a scenario to guarantee Scissor conditions
    // Scissor: Same Hand, Adjacent Fingers, Row Diff >= 2
    let keys = vec![
        // Key 0: Hand 0, Finger 1, Row 0
        KeyNode { index: 0, hand: HandIndex(0), finger: FingerIndex(1), row: RowIndex(0), ..Default::default() },
        // Key 1: Hand 0, Finger 1, Row 0 (SFB with 0)
        KeyNode { index: 1, hand: HandIndex(0), finger: FingerIndex(1), row: RowIndex(0), ..Default::default() },
        // Key 2: Hand 0, Finger 2, Row 2 (Scissor with 0: Adj Finger, Row Diff 2)
        KeyNode { index: 2, hand: HandIndex(0), finger: FingerIndex(2), row: RowIndex(2), ..Default::default() },
    ];
    let kb_manual = Keyboard::new(keys, 1).unwrap();
    
    let mut corpus_manual = Corpus::default();
    corpus_manual.bigrams.push((0, 1, 100)); // SFB
    corpus_manual.bigrams.push((0, 2, 100)); // Scissor

    let engine = ScoringEngine::new(&kb_manual, &corpus_manual, &Rubric::default(), &[]).unwrap();
    // Layout maps char 0->Key0, char 1->Key1, char 2->Key2
    let layout = Layout::new_unchecked(vec![KeyCode(0), KeyCode(1), KeyCode(2)]);
    
    let report = engine.analyze(&layout).unwrap();

    assert!(report.sfb_total > 0.0, "Should detect SFBs");
    assert!(report.scissors > 0.0, "Should detect Scissors");
    assert!(!report.top_sfbs.is_empty());
    assert!(!report.top_scissors.is_empty());
}

#[test]
fn test_metric_detection_rolls_redirects() {
    // Simple 1-row keyboard
    let keys: Vec<KeyNode> = (0..5).map(|i| KeyNode {
        index: i,
        hand: HandIndex(0),
        finger: FingerIndex(i as u8), // 0..4
        ..Default::default()
    }).collect();
    let kb = Keyboard::new(keys, 0).unwrap();

    // Layout: 0=a, 1=b, 2=c, 3=d, 4=e
    let layout = Layout::new_unchecked(vec![KeyCode(97), KeyCode(98), KeyCode(99), KeyCode(100), KeyCode(101)]);
    let mut corpus = Corpus::default();

    // Roll In: Pinky(4) -> Ring(3) -> Mid(2) => 'e' -> 'd' -> 'c'
    corpus.trigrams.push((101, 100, 99, 100));
    // Redirect: Mid(2) -> Ring(3) -> Mid(2) => 'c' -> 'd' -> 'c'
    corpus.trigrams.push((99, 100, 99, 100));

    let rubric = Rubric {
        roll_bonus: 10.0,
        redirect: 50.0,
        trigram_coverage: 1.0,
        trigram_limit: 100,
        ..Rubric::default()
    };

    let engine = ScoringEngine::new(&kb, &corpus, &rubric, &[]).unwrap();
    let report = engine.analyze(&layout).unwrap();

    assert!(report.rolls > 0.0, "Expected rolls");
    assert!(report.redirects > 0.0, "Expected redirects");
}

#[test]
fn test_heatmap_and_penalty_map() {
    let kb = setup_kb(5);
    let mut corpus = Corpus::default();
    
    // 'a'(97) and 'b'(98) have frequency
    corpus.char_freqs[97] = 1000;
    corpus.char_freqs[98] = 1000;
    corpus.bigrams.push((97, 98, 500));

    let engine = ScoringEngine::new(&kb, &corpus, &Rubric::default(), &[]).unwrap();
    let layout = Layout::new_unchecked(vec![KeyCode(97), KeyCode(98), KeyCode(99), KeyCode(100), KeyCode(101)]);
    
    let report = engine.analyze(&layout).unwrap();
    
    // Heatmap (Usage)
    assert!(report.heatmap[0] > 0.0);
    assert!(report.heatmap[1] > 0.0);
    assert_eq!(report.heatmap[2], 0.0); // Unused key
    
    // Penalty Map (Effort)
    assert!(report.penalty_map[0] > 0.0);
    assert!(report.penalty_map[1] > 0.0);
}

#[test]
fn test_space_hand_preference() {
    // Setup: 2 keys. Key 0 (Left Hand), Key 1 (Right Hand).
    let keys = vec![
        KeyNode { index: 0, hand: HandIndex(0), ..Default::default() },
        KeyNode { index: 1, hand: HandIndex(1), ..Default::default() },
    ];
    let kb = Keyboard::new(keys, 0).unwrap();

    // Corpus: Only Space (32) has frequency.
    let mut corpus = Corpus::default();
    corpus.char_freqs[32] = 1000;

    // Layout: Space on BOTH keys.
    let layout = Layout::new_unchecked(vec![KeyCode(32), KeyCode(32)]);
    let rubric = Rubric::default();

    // 1. Left Preference: Should count Key 0, ignore Key 1.
    let engine_l = ScoringEngine::new_with_options(&kb, &corpus, &rubric, &[], SpaceHandPreference::Left).unwrap();
    let report_l = engine_l.analyze(&layout).unwrap();
    assert!(report_l.heatmap[0] > 0.0);
    assert_eq!(report_l.heatmap[1], 0.0);

    // 2. Right Preference: Should ignore Key 0, count Key 1.
    let engine_r = ScoringEngine::new_with_options(&kb, &corpus, &rubric, &[], SpaceHandPreference::Right).unwrap();
    let report_r = engine_r.analyze(&layout).unwrap();
    assert_eq!(report_r.heatmap[0], 0.0);
    assert!(report_r.heatmap[1] > 0.0);

    // 3. Bilateral: Should count both (split load).
    let engine_b = ScoringEngine::new_with_options(&kb, &corpus, &rubric, &[], SpaceHandPreference::Bilateral).unwrap();
    let report_b = engine_b.analyze(&layout).unwrap();
    assert!(report_b.heatmap[0] > 0.0);
    assert!(report_b.heatmap[1] > 0.0);
}

#[test]
fn test_lateral_sfb_mechanics() {
    // Setup: 2 keys. Same Hand, Same Finger, Same Row. Adjacent Cols (0 vs 1).
    let keys = vec![
        KeyNode { index: 0, col: ColIndex(0), ..Default::default() },
        KeyNode { index: 1, col: ColIndex(1), ..Default::default() },
    ];
    let kb = Keyboard::new(keys, 0).unwrap();

    let mut corpus = Corpus::default();
    corpus.bigrams.push((0, 1, 1)); // 1 occurrence

    let mut rubric = Rubric::default();
    rubric.sfb_base = 100.0;
    rubric.sfb_lateral = 200.0; // Distinct value

    let engine = ScoringEngine::new(&kb, &corpus, &rubric, &[]).unwrap();
    let layout = Layout::new_unchecked(vec![KeyCode(0), KeyCode(1)]);
    
    let score = engine.score(&layout).unwrap();
    
    // Should trigger Lateral SFB (200) + Distance Cost (small)
    assert!(score >= 200.0);
    // Should NOT be Base SFB (100)
    assert!(score > 150.0);
}

#[test]
fn test_lateral_stretch() {
    // Setup: 2 keys. Same Hand, Same Row. Adjacent Fingers (1 vs 2).
    // But Cols are far apart (0 vs 2). This is a "Lateral Stretch".
    let keys = vec![
        KeyNode { index: 0, finger: FingerIndex(1), col: ColIndex(0), ..Default::default() },
        KeyNode { index: 1, finger: FingerIndex(2), col: ColIndex(2), ..Default::default() },
    ];
    let kb = Keyboard::new(keys, 0).unwrap();

    let mut corpus = Corpus::default();
    corpus.bigrams.push((0, 1, 1));

    let mut rubric = Rubric::default();
    rubric.sfb_lateral = 500.0; // Used for lateral stretch penalty too

    let engine = ScoringEngine::new(&kb, &corpus, &rubric, &[]).unwrap();
    let layout = Layout::new_unchecked(vec![KeyCode(0), KeyCode(1)]);
    
    let score = engine.score(&layout).unwrap();
    assert!(score >= 500.0);
}