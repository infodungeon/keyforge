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
use super::*;
use crate::types::KeyCode;

#[test]
fn test_layout_validation_duplicates() {
    let keys = vec![KeyCode(65), KeyCode(66), KeyCode(65)]; // A, B, A
    let layout = Layout::try_from(keys);
    assert!(layout.is_err(), "Layout should reject duplicates");
}

#[test]
fn test_layout_validation_valid() {
    let keys = vec![KeyCode(65), KeyCode(66), KeyCode(67)]; // A, B, C
    let layout = Layout::try_from(keys);
    assert!(layout.is_ok(), "Layout should accept unique keys");
}

#[test]
fn test_score_saturation() {
    let max = Score::MAX;
    let added = max + Score(100);
    assert_eq!(added, Score::MAX, "Score should saturate at MAX");

    let min = Score::MIN;
    let subbed = min - Score(100);
    assert_eq!(subbed, Score::MIN, "Score should saturate at MIN");
}

#[test]
fn test_search_config_validation() {
    let config = SearchConfig::Annealing {
        steps: 0, // Invalid
        start_temp: 100.0,
        end_temp: 0.01,
        seed: 0,
        patience: 100,
        reheats: 0,
        reheat_factor: 0.5,
    };
    assert!(config.validate().is_err());
}
