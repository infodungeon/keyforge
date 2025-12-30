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
use keyforge_model::{Corpus, KeyNode, Keyboard, Rubric, types::{HandIndex, FingerIndex, RowIndex, ColIndex, KeyCode}};
use keyforge_physics::ScoringEngine;

#[test]
fn test_analyze_layout_comprehensive() {
    let keys: Vec<_> = (0..30)
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
    let kb = Keyboard::new(keys, 1).unwrap();

    let mut corpus = Corpus::default();
    // SFB: 0 and 2 are same hand (0) and finger (0)
    corpus.bigrams.push((0, 2, 100));
    // SFB: 0 and 20 (hand0, finger0, r0 vs r2)
    corpus.bigrams.push((0, 20, 100));

    let engine = ScoringEngine::new(&kb, &corpus, &Rubric::default(), &[]).unwrap();
    let layout = keyforge_model::Layout::new_unchecked((0..30u16).map(KeyCode).collect());
    let report = engine.analyze(&layout).unwrap();

    assert!(report.score >= 0.0);
    assert!(report.sfb_total > 0.0);
}
