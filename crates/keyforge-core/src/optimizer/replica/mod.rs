pub mod anneal;
pub mod delta;

use crate::core_types::{KeyCode, Layout, PosMap};
use crate::optimizer::mutation;
use crate::scorer::Scorer;
use fastrand::Rng;
use std::sync::Arc;

#[repr(align(64))]
pub struct Replica {
    pub scorer: Arc<Scorer>,

    // Cached Scorer Data
    pub local_cost_matrix: Vec<f32>,
    pub local_trigram_costs: Vec<f32>,
    pub local_monogram_costs: Vec<f32>,

    pub layout: Layout,
    pub pos_map: PosMap,

    pub score: f32,
    pub left_load: f32,
    pub total_freq: f32,

    pub temperature: f32,
    pub current_limit: usize,
    pub limit_fast: usize,
    pub limit_slow: usize,

    pub rng: Rng,
    pub pinned_slots: Vec<Option<KeyCode>>,
    pub locked_indices: Vec<usize>,

    // Weighted Mutation Fields
    pub mutation_weights: Vec<f32>,
    pub total_weight: f32,
}

impl Replica {
    pub fn new(
        scorer: Arc<Scorer>,
        temperature: f32,
        seed: Option<u64>,
        limit_fast: usize,
        limit_slow: usize,
        pinned_keys_str: &str,
    ) -> Self {
        let mut rng = if let Some(s) = seed {
            Rng::with_seed(s)
        } else {
            Rng::new()
        };

        let key_count = scorer.key_count;
        let (pinned_slots, mut locked_indices) = parse_pins(pinned_keys_str, key_count);

        locked_indices.sort();

        let mut layout;
        let mut pos_map;

        loop {
            layout = mutation::generate_tiered_layout(
                &mut rng,
                &scorer.defs,
                &scorer.geometry,
                key_count,
                &pinned_slots,
            );
            pos_map = mutation::build_pos_map(&layout);

            let critical = scorer.defs.get_critical_bigrams();
            if !mutation::fails_sanity(&pos_map, &critical, &scorer.geometry) {
                break;
            }
        }

        let start_limit = if temperature > 10.0 {
            limit_fast
        } else {
            limit_slow
        };
        let (base, left, total) = scorer.score_full(&pos_map, start_limit);

        // Clone hefty data for thread independence
        let local_cost_matrix = scorer.full_cost_matrix.clone();
        let local_trigram_costs = scorer.trigram_cost_table.clone();
        let local_monogram_costs = scorer.slot_monogram_costs.clone();

        let mut r = Replica {
            scorer,
            local_cost_matrix,
            local_trigram_costs,
            local_monogram_costs,
            layout,
            pos_map,
            score: base,
            left_load: left,
            total_freq: total,
            temperature,
            current_limit: start_limit,
            limit_fast,
            limit_slow,
            rng,
            pinned_slots,
            locked_indices,
            mutation_weights: vec![1.0; key_count],
            total_weight: key_count as f32,
        };

        let imb = r.imbalance_penalty(left);
        r.score += imb;
        r.update_mutation_weights();

        r
    }

    pub fn inject_layout(&mut self, new_layout: &[KeyCode]) {
        self.layout = new_layout.to_vec();
        self.pos_map = mutation::build_pos_map(&self.layout);

        let (base, left, total) = self.scorer.score_full(&self.pos_map, self.current_limit);
        let imb = self.imbalance_penalty(left);

        self.score = base + imb;
        self.left_load = left;
        self.total_freq = total;

        self.update_mutation_weights();
    }

    pub fn update_mutation_weights(&mut self) {
        let costs = self.scorer.get_element_costs(&self.pos_map);
        let mut sum = 0.0;

        for (i, &c) in costs.iter().enumerate() {
            if self.locked_indices.contains(&i) {
                self.mutation_weights[i] = 0.0;
            } else {
                self.mutation_weights[i] = (c + 1.0).powf(1.5);
            }
            sum += self.mutation_weights[i];
        }
        self.total_weight = sum;
    }

    #[inline(always)]
    pub fn imbalance_penalty(&self, left: f32) -> f32 {
        if self.total_freq > 0.0 {
            let ratio = left / self.total_freq;
            let diff = (ratio - 0.5).abs();
            let allowed = self.scorer.weights.allowed_hand_balance_deviation();
            if diff > allowed {
                return diff * self.scorer.weights.penalty_imbalance;
            }
        }
        0.0
    }
}

fn parse_pins(input: &str, count: usize) -> (Vec<Option<KeyCode>>, Vec<usize>) {
    let mut slots = vec![None; count];
    let mut indices = Vec::new();

    if input.is_empty() {
        return (slots, indices);
    }

    for part in input.split(',') {
        let parts: Vec<&str> = part.split(':').collect();
        if parts.len() == 2 {
            if let Ok(idx) = parts[0].trim().parse::<usize>() {
                if idx < count {
                    if let Some(c) = parts[1].trim().chars().next() {
                        let code = c.to_ascii_lowercase() as KeyCode;
                        slots[idx] = Some(code);
                        indices.push(idx);
                    }
                }
            }
        }
    }
    (slots, indices)
}
