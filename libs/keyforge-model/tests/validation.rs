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
    Corpus, Rubric, SearchConfig, Layout, KeyCode, Validator,
    KeyboardGeometry, KeyNode, HandIndex
};

#[test]
fn test_layout_validation() {
    // Duplicates
    let keys = vec![KeyCode(65), KeyCode(66), KeyCode(65)];
    assert!(Layout::try_from(keys).is_err());

    // Valid
    let keys = vec![KeyCode(65), KeyCode(66), KeyCode(67)];
    assert!(Layout::try_from(keys).is_ok());
}

#[test]
fn test_search_config_validation() {
    // Valid default
    let c = SearchConfig::default();
    assert!(c.validate().is_ok());

    // Invalid Steps
    let invalid_steps = SearchConfig::Annealing {
        steps: 0,
        start_temp: 100.0,
        end_temp: 0.01,
        seed: 42,
        patience: 500,
        reheats: 3,
        reheat_factor: 0.5,
    };
    assert!(invalid_steps.validate().is_err());

    // Invalid Temp
    let invalid_temp = SearchConfig::Annealing {
        steps: 100,
        start_temp: -1.0,
        end_temp: 0.01,
        seed: 42,
        patience: 500,
        reheats: 3,
        reheat_factor: 0.5,
    };
    assert!(invalid_temp.validate().is_err());
}

#[test]
fn test_rubric_validation() {
    let mut r = Rubric::default();
    assert!(r.validate().is_ok());

    // Coverage bounds
    r.trigram_coverage = 1.5; // > 1.0
    assert!(r.validate().is_err());
    r.trigram_coverage = -0.1;
    assert!(r.validate().is_err());

    // Reset to valid
    r.trigram_coverage = 0.99;

    // Limits
    r.trigram_limit = 0;
    assert!(r.validate().is_err());
    r.trigram_limit = 100;

    // Weights
    r.sfb_base = -10.0; // Negative penalty
    assert!(r.validate().is_err());
}

#[test]
fn test_corpus_validation() {
    let mut c = Corpus::default();
    assert!(c.validate().is_ok());

    // Too short
    c.char_freqs = vec![0; 10]; 
    assert!(c.validate().is_err());

    // Too long
    c.char_freqs = vec![0; 70000];
    assert!(c.validate().is_err());

    // Valid mutation
    let mut c2 = Corpus::default();
    c2.char_freqs['a' as usize] = 100;
    c2.bigrams.push(('a' as u16, 'b' as u16, 50));
    assert!(c2.validate().is_ok());
}

#[test]
fn test_keyboard_geometry_validation() {
    let mut geom = KeyboardGeometry::default();
    // Empty keys
    assert!(geom.validate().is_err());

    // Invalid Key (Hand > 1) - forcing invalid state manually
    // Note: HandIndex::try_from prevents this usually, but we test the geometry validator's check
    geom.keys.push(KeyNode {
        hand: HandIndex(0), 
        w: 0.0, // Invalid dimension
        ..Default::default()
    });
    
    assert!(geom.validate().is_err());
}