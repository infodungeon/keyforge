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
use keyforge_model::{Corpus, Keyboard, Rubric};
use crate::errors::PhysicsError;
use tracing::{info, instrument};

pub struct Compiler;

impl Compiler {
    #[instrument(skip_all)]
    pub fn compile(
        kb: &Keyboard,
        corpus: &Corpus,
        rubric: &Rubric,
        cost_matrix_entries: &[(usize, usize, f32)],
    ) -> Result<EngineContext, PhysicsError> {
        let key_count = kb.count();
        info!(key_count = key_count, "Compiling scoring engine...");

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

            let mut cost = rubric.finger_effort[k.finger.as_usize()];
            let mut dist_from_home = 0.0;
            
            if let Some(origin) = kb.finger_origins.get(k.hand.as_usize()).and_then(|h| h.get(k.finger.as_usize())) {
                let dx = (k.x - origin.0).abs();
                let dy = (k.y - origin.1).abs();
                let dx2 = dx * dx;
                let dy2 = dy * dy;
                dist_from_home = (dx2 + dy2).sqrt();
                cost += (dx2 * rubric.travel_lat) + (dy2 * rubric.travel_vert);
            }

            key_costs.push(Score::from_f32(cost));
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

        // Apply manual cost matrix entries
        for &(i, j, cost) in cost_matrix_entries {
            if i < key_count && j < key_count {
                internal_cost_matrix[i * key_count + j] = Score::from_f32(cost);
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
            penalty_redirect: Score::from_f32(rubric.redirect),
            penalty_skip: Score::ZERO,
            bonus_roll: Score::from_f32(rubric.roll_bonus),
        })
    }
}

/// Efficiently flattens bigrams into CSR-like structures without massive array allocations.
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

    // Fill remaining starts
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
