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
use super::mechanics::calculate_pair_cost;
use super::types::{KeyCode, KeyIndex, Score};
use super::EngineContext;
use keyforge_model::{Corpus, Keyboard, Rubric};
use crate::errors::PhysicsError;
use tracing::instrument;

pub struct Compiler;

impl Compiler {
    #[instrument(skip_all)]
    pub fn compile(
        kb: &Keyboard,
        corpus: &Corpus,
        rubric: &Rubric,
        overrides: &[(usize, usize, f32)],
        space_preference: keyforge_model::types::SpaceHandPreference,
    ) -> Result<EngineContext, PhysicsError> {
        let key_count = kb.count();

        let mut hands = Vec::with_capacity(key_count);
        let mut fingers = Vec::with_capacity(key_count);
        let mut rows = Vec::with_capacity(key_count);
        let mut cols = Vec::with_capacity(key_count);
        let mut key_costs = Vec::with_capacity(key_count);

        for k in &kb.keys {
            hands.push(k.hand);
            fingers.push(k.finger);
            rows.push(k.row);
            cols.push(k.col);

            // Calculate Static Monogram Cost
            // Cost = Finger Effort + Travel (from origin)
            // Note: calculate_pair_cost handles relative travel.
            // Here we want absolute cost of pressing the key.
            // We can approximate this using the finger effort weight.
            // Travel cost is usually relative to home row, which is implicit in finger effort for some models,
            // but explicit in others.
            // Let's use finger_effort + row penalty (if we had it) + travel from finger origin.
            
            let mut cost = rubric.finger_effort[k.finger.as_usize()];
            
            // Add travel cost from finger origin (home row)
            if let Some(origin) = kb.finger_origins.get(k.hand.as_usize()).and_then(|h| h.get(k.finger.as_usize())) {
                let dx = (k.x - origin.0).abs();
                let dy = (k.y - origin.1).abs();
                cost += (dx * dx * rubric.travel_lat) + (dy * dy * rubric.travel_vert);
            }

            // DEBUG: Log cost for specific keys
            if k.label == "KeyO" || k.label == "Dot" || k.label == "SpaceL" {
                tracing::info!("Compiler: Key {} ({}) Cost: {}", k.index, k.label, cost);
            }

            key_costs.push(Score::from_f32(cost));
        }

        let mut cost_matrix = vec![Score::ZERO; key_count * key_count];

        for i in 0..key_count {
            for j in 0..key_count {
                let cost = calculate_pair_cost(kb, rubric, KeyIndex::from(i), KeyIndex::from(j));
                cost_matrix[i * key_count + j] = Score::from_f32(cost);
            }
        }

        for &(i, j, cost) in overrides {
            if i < key_count && j < key_count {
                cost_matrix[i * key_count + j] = Score::from_f32(cost);
            }
        }

        let (bigram_starts, bigram_others, bigram_freqs) = flatten_bigrams(&corpus.bigrams);
        let (bigram_rev_starts, bigram_rev_others, bigram_rev_freqs) = flatten_bigrams_rev(&corpus.bigrams);
        let pruned_trigrams = prune_trigrams(corpus.trigrams.clone(), rubric.trigram_coverage, rubric.trigram_limit);
        let (trigram_starts, trigram_others1, trigram_others2, trigram_freqs) = flatten_trigrams_start(&pruned_trigrams);
        let (trigram_mid_starts, trigram_mid_others1, trigram_mid_others2, trigram_mid_freqs) = flatten_trigrams_mid(&pruned_trigrams);
        let (trigram_end_starts, trigram_end_others1, trigram_end_others2, trigram_end_freqs) = flatten_trigrams_end(&pruned_trigrams);
        let char_freqs = corpus.char_freqs.clone();

        Ok(EngineContext {
            key_count, hands, fingers, rows, cols, cost_matrix, key_costs, char_freqs,
            bigram_starts, bigram_others, bigram_freqs,
            bigram_rev_starts, bigram_rev_others, bigram_rev_freqs,
            trigram_starts, trigram_others1, trigram_others2, trigram_freqs,
            trigram_mid_starts, trigram_mid_others1, trigram_mid_others2, trigram_mid_freqs,
            trigram_end_starts, trigram_end_others1, trigram_end_others2, trigram_end_freqs,
            penalty_redirect: Score::from_f32(rubric.redirect),
            penalty_skip: Score::ZERO,
            bonus_roll: Score::from_f32(rubric.roll_bonus),
            space_preference,
        })
    }
}

fn flatten_bigrams(source: &[(u16, u16, u32)]) -> (Vec<usize>, Vec<KeyCode>, Vec<u32>) {
    let mut buckets = vec![Vec::new(); 65536];
    for &(c1, c2, freq) in source { buckets[c1 as usize].push((KeyCode(c2), freq)); }
    flatten_buckets(buckets)
}

fn flatten_bigrams_rev(source: &[(u16, u16, u32)]) -> (Vec<usize>, Vec<KeyCode>, Vec<u32>) {
    let mut buckets = vec![Vec::new(); 65536];
    for &(c1, c2, freq) in source { buckets[c2 as usize].push((KeyCode(c1), freq)); }
    flatten_buckets(buckets)
}

fn prune_trigrams(mut source: Vec<(u16, u16, u16, u32)>, coverage: f32, limit: usize) -> Vec<(u16, u16, u16, u32)> {
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
    let mut buckets = vec![Vec::new(); 65536];
    for &(c1, c2, c3, freq) in source { buckets[c1 as usize].push((KeyCode(c2), KeyCode(c3), freq)); }
    flatten_buckets_tri(buckets)
}

fn flatten_trigrams_mid(source: &[(u16, u16, u16, u32)]) -> (Vec<usize>, Vec<KeyCode>, Vec<KeyCode>, Vec<u32>) {
    let mut buckets = vec![Vec::new(); 65536];
    for &(c1, c2, c3, freq) in source { buckets[c2 as usize].push((KeyCode(c1), KeyCode(c3), freq)); }
    flatten_buckets_tri(buckets)
}

fn flatten_trigrams_end(source: &[(u16, u16, u16, u32)]) -> (Vec<usize>, Vec<KeyCode>, Vec<KeyCode>, Vec<u32>) {
    let mut buckets = vec![Vec::new(); 65536];
    for &(c1, c2, c3, freq) in source { buckets[c3 as usize].push((KeyCode(c1), KeyCode(c2), freq)); }
    flatten_buckets_tri(buckets)
}

fn flatten_buckets(buckets: Vec<Vec<(KeyCode, u32)>>) -> (Vec<usize>, Vec<KeyCode>, Vec<u32>) {
    let mut starts = vec![0; 65537];
    let mut others = Vec::new();
    let mut freqs = Vec::new();
    let mut offset = 0;
    for i in 0..65536 {
        starts[i] = offset;
        for (o, f) in &buckets[i] { others.push(*o); freqs.push(*f); }
        offset += buckets[i].len();
    }
    starts[65536] = offset;
    (starts, others, freqs)
}

fn flatten_buckets_tri(buckets: Vec<Vec<(KeyCode, KeyCode, u32)>>) -> (Vec<usize>, Vec<KeyCode>, Vec<KeyCode>, Vec<u32>) {
    let mut starts = vec![0; 65537];
    let mut o1 = Vec::new();
    let mut o2 = Vec::new();
    let mut freqs = Vec::new();
    let mut offset = 0;
    for i in 0..65536 {
        starts[i] = offset;
        for (a, b, f) in &buckets[i] { o1.push(*a); o2.push(*b); freqs.push(*f); }
        offset += buckets[i].len();
    }
    starts[65536] = offset;
    (starts, o1, o2, freqs)
}
