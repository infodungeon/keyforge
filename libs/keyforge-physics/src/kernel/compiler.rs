// libs/keyforge-physics/src/kernel/compiler.rs

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

use super::mechanics::calculate_pair_cost;
use super::types::{KeyCode, KeyIndex, Score};
use super::EngineContext;
use keyforge_model::{Corpus, Keyboard, Rubric, CostModel, KeyNode};
use keyforge_model::cost_model::{FingerDefinition, HandDefinition};
use keyforge_model::types::{HandIndex, FingerIndex};
use crate::errors::PhysicsError;
use tracing::{info, instrument, warn};

pub struct Compiler;

impl Compiler {
    #[instrument(skip_all)]
    pub fn compile(
        kb: &Keyboard,
        corpus: &Corpus,
        rubric: &Rubric,
        cost_model: &CostModel,
    ) -> Result<EngineContext, PhysicsError> {
        let key_count = kb.count();
        info!(key_count = key_count, "Compiling scoring engine...");

        // 1. Select the appropriate physical model based on keyboard metadata/type
        // For now, default to "model_a_row_staggered" if not specified.
        let model_key = "model_a_row_staggered"; 
        let phys_model = cost_model.models.get(model_key)
            .ok_or_else(|| PhysicsError::Config(format!("Missing cost model: {}", model_key)))?;

        let mut hands = Vec::with_capacity(key_count);
        let mut fingers = Vec::with_capacity(key_count);
        let mut rows = Vec::with_capacity(key_count);
        let mut cols = Vec::with_capacity(key_count);
        let mut key_costs = Vec::with_capacity(key_count);
        let mut key_home_distances = Vec::with_capacity(key_count);

        for k in &kb.keys {
            hands.push(k.hand);
            fingers.push(k.finger);
            rows.push(k.row);
            cols.push(k.col);

            // Resolve Static Cost from CostModel
            let cost_val = resolve_key_cost(k, &phys_model.static_costs);
            key_costs.push(Score::from_f32(cost_val));

            let mut dist_from_home = 0.0;
            if let Some(origin) = kb.finger_origins.get(k.hand.as_usize()).and_then(|h| h.get(k.finger.as_usize())) {
                let dx = (k.x - origin.0).abs();
                let dy = (k.y - origin.1).abs();
                dist_from_home = (dx * dx + dy * dy).sqrt();
            }
            key_home_distances.push(dist_from_home);
        }

        let mut internal_cost_matrix = vec![Score::ZERO; key_count * key_count];
        let mut dist_matrix = vec![0.0f32; key_count * key_count];

        for i in 0..key_count {
            for j in 0..key_count {
                let cost = calculate_pair_cost(kb, rubric, KeyIndex::from(i), KeyIndex::from(j));
                internal_cost_matrix[i * key_count + j] = Score::from_f32(cost);

                if i == j {
                    dist_matrix[i * key_count + j] = 0.0;
                } else {
                    let k1 = &kb.keys[i];
                    let k2 = &kb.keys[j];
                    let dx = (k1.x - k2.x).abs();
                    let dy = (k1.y - k2.y).abs();
                    dist_matrix[i * key_count + j] = (dx * dx + dy * dy).sqrt();
                }
            }
        }

        let (bigram_starts, bigram_others, bigram_freqs) = flatten_bigrams(&corpus.bigrams);
        let (bigram_rev_starts, bigram_rev_others, bigram_rev_freqs) = flatten_bigrams_rev(&corpus.bigrams);
        
        let pruned_trigrams = prune_trigrams(corpus.trigrams.clone(), rubric.trigram_coverage, rubric.trigram_limit);
        
        let (trigram_starts, trigram_others1, trigram_others2, trigram_freqs) = flatten_trigrams_start(&pruned_trigrams);
        let (trigram_mid_starts, trigram_mid_others1, trigram_mid_others2, trigram_mid_freqs) = flatten_trigrams_mid(&pruned_trigrams);
        let (trigram_end_starts, trigram_end_others1, trigram_end_others2, trigram_end_freqs) = flatten_trigrams_end(&pruned_trigrams);
        
        let char_freqs = corpus.char_freqs.clone();

        info!("Engine compilation complete.");

        Ok(EngineContext {
            key_count, hands, fingers, rows, cols, cost_matrix: internal_cost_matrix, dist_matrix, key_home_distances, key_costs, char_freqs,
            bigram_starts, bigram_others, bigram_freqs,
            bigram_rev_starts, bigram_rev_others, bigram_rev_freqs,
            trigram_starts, trigram_others1, trigram_others2, trigram_freqs,
            trigram_mid_starts, trigram_mid_others1, trigram_mid_others2, trigram_mid_freqs,
            trigram_end_starts, trigram_end_others1, trigram_end_others2, trigram_end_freqs,
            all_bigrams: corpus.bigrams.clone(),
            all_trigrams: corpus.trigrams.clone(),
            penalty_redirect: Score::from_f32(rubric.redirect),
            penalty_skip: Score::ZERO,
            bonus_roll: Score::from_f32(rubric.roll_bonus),
        })
    }
}

fn resolve_key_cost(key: &KeyNode, static_costs: &std::collections::HashMap<String, HandDefinition>) -> f32 {
    let hand_key = if key.hand == HandIndex::LEFT { "left_hand" } else { "right_hand" };
    let hand_def = static_costs.get(hand_key).or_else(|| static_costs.get("universal_hand"));

    if let Some(hand) = hand_def {
        let finger_key = match key.finger {
            FingerIndex::THUMB => "thumb",
            FingerIndex::INDEX => "index",
            FingerIndex::MIDDLE => "middle",
            FingerIndex::RING => "ring",
            FingerIndex::PINKY => "pinky",
            _ => "unknown",
        };

        if let Some(finger_def) = hand.fingers.get(finger_key) {
            match finger_def {
                FingerDefinition::Standard(zones) => {
                    let zone_key = if key.col.0.abs() > 1 && key.finger == FingerIndex::INDEX {
                        "inner"
                    } else if key.col.0.abs() > 1 && key.finger == FingerIndex::PINKY {
                        "outer"
                    } else {
                        "base"
                    };

                    if let Some(zone) = zones.get(zone_key).or_else(|| zones.get("base")) {
                        let row_key = format!("r{}", key.row.0);
                        if let Some(cost) = zone.get(&row_key) {
                            return *cost;
                        }
                    }
                },
                FingerDefinition::Thumb(positions) => {
                    return *positions.values().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&100.0);
                }
            }
        }
    }
    
    warn!("Cost lookup failed for key {:?}, using default 100.0", key);
    100.0
}

// ... (Keep existing flatten_* and prune_trigrams functions unchanged) ...
fn flatten_bigrams(source: &[(u16, u16, u32)]) -> (Vec<usize>, Vec<KeyCode>, Vec<u32>) {
    let mut sorted = source.to_vec();
    sorted.sort_unstable_by_key(|&(c1, _, _)| c1);

    let mut starts = vec![0; 65537];
    let mut others = Vec::with_capacity(source.len());
    let mut freqs = Vec::with_capacity(source.len());

    let mut current_char = 0usize;
    let mut current_offset = 0usize;

    for &(c1, c2, freq) in &sorted {
        let c1 = c1 as usize;
        while current_char <= c1 {
            starts[current_char] = current_offset;
            current_char += 1;
        }
        others.push(KeyCode(c2));
        freqs.push(freq);
        current_offset += 1;
    }

    while current_char <= 65536 {
        starts[current_char] = current_offset;
        current_char += 1;
    }

    (starts, others, freqs)
}

fn flatten_bigrams_rev(source: &[(u16, u16, u32)]) -> (Vec<usize>, Vec<KeyCode>, Vec<u32>) {
    let mut sorted = source.to_vec();
    sorted.sort_unstable_by_key(|&(_, c2, _)| c2);

    let mut starts = vec![0; 65537];
    let mut others = Vec::with_capacity(source.len());
    let mut freqs = Vec::with_capacity(source.len());

    let mut current_char = 0usize;
    let mut current_offset = 0usize;

    for &(c1, c2, freq) in &sorted {
        let c2 = c2 as usize;
        while current_char <= c2 {
            starts[current_char] = current_offset;
            current_char += 1;
        }
        others.push(KeyCode(c1));
        freqs.push(freq);
        current_offset += 1;
    }

    while current_char <= 65536 {
        starts[current_char] = current_offset;
        current_char += 1;
    }

    (starts, others, freqs)
}

fn prune_trigrams(mut source: Vec<(u16, u16, u16, u32)>, coverage: f32, limit: usize) -> Vec<(u16, u16, u16, u32)> {
    if source.is_empty() { return source; }
    source.sort_unstable_by(|a, b| b.3.cmp(&a.3).then_with(|| a.0.cmp(&b.0)).then_with(|| a.1.cmp(&b.1)).then_with(|| a.2.cmp(&b.2)));
    let total_freq: u64 = source.iter().map(|x| x.3 as u64).sum();
    let target = (total_freq as f64 * coverage as f64) as u64;
    let mut acc = 0;
    let mut cutoff = source.len();
    for (i, item) in source.iter().enumerate() {
        acc += item.3 as u64;
        if acc >= target { cutoff = i + 1; break; }
    }
    if cutoff > limit { cutoff = limit; }
    source.truncate(cutoff);
    source
}

fn flatten_trigrams_start(source: &[(u16, u16, u16, u32)]) -> (Vec<usize>, Vec<KeyCode>, Vec<KeyCode>, Vec<u32>) {
    let mut sorted = source.to_vec();
    sorted.sort_unstable_by_key(|&(c1, _, _, _)| c1);

    let mut starts = vec![0; 65537];
    let mut o1 = Vec::with_capacity(source.len());
    let mut o2 = Vec::with_capacity(source.len());
    let mut freqs = Vec::with_capacity(source.len());

    let mut current_char = 0usize;
    let mut current_offset = 0usize;

    for &(c1, c2, c3, freq) in &sorted {
        let c1 = c1 as usize;
        while current_char <= c1 {
            starts[current_char] = current_offset;
            current_char += 1;
        }
        o1.push(KeyCode(c2));
        o2.push(KeyCode(c3));
        freqs.push(freq);
        current_offset += 1;
    }

    while current_char <= 65536 {
        starts[current_char] = current_offset;
        current_char += 1;
    }

    (starts, o1, o2, freqs)
}

fn flatten_trigrams_mid(source: &[(u16, u16, u16, u32)]) -> (Vec<usize>, Vec<KeyCode>, Vec<KeyCode>, Vec<u32>) {
    let mut sorted = source.to_vec();
    sorted.sort_unstable_by_key(|&(_, c2, _, _)| c2);

    let mut starts = vec![0; 65537];
    let mut o1 = Vec::with_capacity(source.len());
    let mut o2 = Vec::with_capacity(source.len());
    let mut freqs = Vec::with_capacity(source.len());

    let mut current_char = 0usize;
    let mut current_offset = 0usize;

    for &(c1, c2, c3, freq) in &sorted {
        let c2 = c2 as usize;
        while current_char <= c2 {
            starts[current_char] = current_offset;
            current_char += 1;
        }
        o1.push(KeyCode(c1));
        o2.push(KeyCode(c3));
        freqs.push(freq);
        current_offset += 1;
    }

    while current_char <= 65536 {
        starts[current_char] = current_offset;
        current_char += 1;
    }

    (starts, o1, o2, freqs)
}

fn flatten_trigrams_end(source: &[(u16, u16, u16, u32)]) -> (Vec<usize>, Vec<KeyCode>, Vec<KeyCode>, Vec<u32>) {
    let mut sorted = source.to_vec();
    sorted.sort_unstable_by_key(|&(_, _, c3, _)| c3);

    let mut starts = vec![0; 65537];
    let mut o1 = Vec::with_capacity(source.len());
    let mut o2 = Vec::with_capacity(source.len());
    let mut freqs = Vec::with_capacity(source.len());

    let mut current_char = 0usize;
    let mut current_offset = 0usize;

    for &(c1, c2, c3, freq) in &sorted {
        let c3 = c3 as usize;
        while current_char <= c3 {
            starts[current_char] = current_offset;
            current_char += 1;
        }
        o1.push(KeyCode(c1));
        o2.push(KeyCode(c2));
        freqs.push(freq);
        current_offset += 1;
    }

    while current_char <= 65536 {
        starts[current_char] = current_offset;
        current_char += 1;
    }

    (starts, o1, o2, freqs)
}

#[cfg(test)]
mod tests {
    use crate::ScoringEngine;
    use keyforge_model::{
        Corpus, KeyNode, Keyboard, Rubric, CostModel,
        types::{HandIndex, FingerIndex, KeyCode}
    };

    fn setup_kb_compiler() -> Keyboard {
        let keys: Vec<KeyNode> = (0..5).map(|i| KeyNode {
            index: i,
            hand: HandIndex(0),
            finger: FingerIndex(i as u8),
            x: i as f32,
            ..Default::default()
        }).collect();
        Keyboard::new(keys, 0).unwrap()
    }

    fn mock_cost_model() -> CostModel {
        let json = r#"{
            "meta": { "version": "2.0", "description": "Test", "unit": "pts" },
            "models": {
                "model_a_row_staggered": {
                    "description": "Test Model",
                    "static_costs": {
                        "universal_hand": {
                            "thumb": { "pos_1": 100.0 },
                            "index": { "base": { "r0": 100.0 } },
                            "middle": { "base": { "r0": 100.0 } },
                            "ring": { "base": { "r0": 100.0 } },
                            "pinky": { "base": { "r0": 100.0 } }
                        }
                    }
                }
            },
            "dynamic_rules": { "sequence_modifiers": {}, "penalties": {}, "constraints": {} }
        }"#;
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn test_compiler_trigram_pruning() {
        let kb = setup_kb_compiler();
        let mut corpus = Corpus::default();
        for i in 0..20 {
            corpus.trigrams.push((0, 1, i as u16, 100));
        }
        
        let mut rubric = Rubric::default();
        rubric.trigram_limit = 5;
        
        let engine = ScoringEngine::new(&kb, &corpus, &rubric, &mock_cost_model()).unwrap();
        
        assert_eq!(engine.trigram_count(), 5);
    }

    #[test]
    fn test_finger_origin_fallback() {
        let keys = vec![
            KeyNode { index: 0, finger: FingerIndex(1), is_home: false, ..Default::default() }
        ];
        let kb = Keyboard::new(keys, 0).unwrap();
        let corpus = Corpus::default();
        let rubric = Rubric::default();
        
        let result = ScoringEngine::new(&kb, &corpus, &rubric, &mock_cost_model());
        assert!(result.is_ok());
    }
}
