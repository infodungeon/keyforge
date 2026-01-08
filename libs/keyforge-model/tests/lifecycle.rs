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


use keyforge_model::{Corpus, Rubric, Score, KeyboardDefinition};
use serde_json;

#[test]
fn test_corpus_lifecycle() {
    // 1. Default Construction
    let mut c = Corpus::default();
    assert_eq!(
        c.char_freqs.len(),
        65536,
        "Corpus should initialize full unicode frequency map"
    );
    assert!(c.bigrams.is_empty());
    assert!(c.trigrams.is_empty());
    assert!(c.words.is_empty());

    // 2. Mutation
    c.char_freqs['a' as usize] = 100;
    c.bigrams.push(('a' as u16, 'b' as u16, 50));
    c.trigrams.push(('a' as u16, 'b' as u16, 'c' as u16, 10));
    c.words.push(("test".to_string(), 5));

    // 3. Serialization Round-trip
    let json = serde_json::to_string(&c).expect("Failed to serialize Corpus");
    let recovered: Corpus = serde_json::from_str(&json).expect("Failed to deserialize Corpus");

    // 4. Verification
    assert_eq!(recovered.char_freqs['a' as usize], 100);
    assert_eq!(recovered.bigrams.len(), 1);
    assert_eq!(recovered.bigrams[0], ('a' as u16, 'b' as u16, 50));
    assert_eq!(recovered.trigrams.len(), 1);
    assert_eq!(recovered.words.len(), 1);
    assert_eq!(recovered.words[0].0, "test");
}

#[test]
fn test_rubric_lifecycle() {
    // 1. Default Construction
    let r = Rubric::default();

    // Check key defaults to ensure physics engine gets sensible start values
    assert!(r.sfb_base > 0.0);
    assert!(r.travel_lat > 0.0);
    assert!(r.travel_vert > 0.0);
    assert_eq!(r.finger_effort.len(), 5);

    // 2. Serialization Round-trip
    let json = serde_json::to_string(&r).expect("Failed to serialize Rubric");
    let recovered: Rubric = serde_json::from_str(&json).expect("Failed to deserialize Rubric");

    // 3. Verification
    assert_eq!(r.sfb_base, recovered.sfb_base);
    assert_eq!(r.finger_effort, recovered.finger_effort);
}

#[test]
fn test_rubric_modification() {
    let mut r = Rubric::default();
    r.sfb_base = 1000.0;
    r.finger_effort[4] = 5.0; // Pinky penalty

    assert_eq!(r.sfb_base, 1000.0);
    assert_eq!(r.finger_effort[4], 5.0);
}

#[test]
fn test_score_overflow_logging() {
    let max = Score::MAX;
    
    // This will trigger a tracing::error! log but should NOT crash.
    // It should saturate to MAX.
    let added = max + Score(100);
    assert_eq!(added, Score::MAX);

    let min = Score::MIN;
    
    // This will trigger a tracing::error! log but should NOT crash.
    // It should saturate to MIN.
    let subbed = min - Score(100);
    assert_eq!(subbed, Score::MIN);
}

#[test]
fn test_keyboard_definition_deserialization() {
    // Tests deserialization of a known complex geometry (SZR35)
    let json = r#"{
      "meta": {
        "name": "SZR35",
        "author": "KeyForge",
        "version": "1.0",
        "notes": "36-key Split Column-Staggered (3x5+3).",
        "type": "split_column_staggered"
      },
      "geometry": {
        "keys": [
          {"id": "KeyQ", "x": 0, "y": 0.5, "hand": 0, "finger": 4, "row": 0, "col": 0},
          {"id": "KeyW", "x": 1, "y": 0.25, "hand": 0, "finger": 3, "row": 0, "col": 1}
        ],
        "prime_slots": [0, 1],
        "med_slots": [],
        "low_slots": [],
        "home_row": 1
      },
      "layouts": {}
    }"#;

    let def: KeyboardDefinition = serde_json::from_str(json).expect("Failed to deserialize KeyboardDefinition");
    assert_eq!(def.geometry.keys.len(), 2, "Should have 2 keys");
    assert_eq!(def.geometry.keys[0].label, "KeyQ", "Label should be KeyQ");
}