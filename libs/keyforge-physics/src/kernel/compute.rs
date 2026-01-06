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
use super::types::{KeyCode, Score, ValidatedLayout};
use super::EngineContext;
use keyforge_model::{AnalysisReport, MetricViolation};
use keyforge_model::constants::SCORE_SCALE;
use tracing::instrument;

/// Internal structure to track duplicate key positions without heap allocation.
struct PosMap {
    starts: [u16; 65536],
    counts: [u8; 65536],
    indices: [u16; 512], 
}

impl PosMap {
    fn new(layout: &[KeyCode], key_count: usize) -> Self {
        let mut pm = Self {
            starts: [0; 65536],
            counts: [0; 65536],
            indices: [0; 512],
        };
        
        let limit = layout.len().min(key_count);
        
        // First pass: count occurrences
        for &code in layout.iter().take(limit) {
            pm.counts[code.0 as usize] += 1;
        }

        // Second pass: calculate starts (prefix sum)
        let mut offset = 0;
        for i in 0..65536 {
            if pm.counts[i] > 0 {
                pm.starts[i] = offset as u16;
                offset += pm.counts[i] as usize;
            }
        }

        // Third pass: fill indices
        let mut current_offsets = [0u8; 65536];
        for (i, &code) in layout.iter().enumerate().take(limit) {
            let c_val = code.0 as usize;
            let base = pm.starts[c_val] as usize;
            let off = current_offsets[c_val] as usize;
            let target = base + off;
            if target < 512 {
                pm.indices[target] = i as u16;
                current_offsets[c_val] += 1;
            }
        }
        pm
    }

    #[inline(always)]
    fn get(&self, code: usize) -> &[u16] {
        let start = self.starts[code] as usize;
        let end = start + (self.counts[code] as usize);
        &self.indices[start..end]
    }
}

pub fn score_layout(ctx: &EngineContext, layout: &ValidatedLayout, _pos_map_scratch: &mut [u16]) -> i64 {
    let mut total_score = Score::ZERO;
    let layout_slice = layout.as_slice();
    let pm = PosMap::new(layout_slice, ctx.key_count);

    // 1. Monograms: Optimal Choice
    for (c_val, &freq) in ctx.char_freqs.iter().enumerate() {
        if freq == 0 { continue; }
        let candidates = pm.get(c_val);
        if candidates.is_empty() { continue; }

        let mut min_cost = Score(i64::MAX);
        for &p in candidates {
            let cost = ctx.key_costs[p as usize];
            if cost < min_cost { min_cost = cost; }
        }
        total_score = total_score.saturating_add(min_cost.saturating_mul(freq as i64));
    }

    // 2. Bigrams: Optimal Choice
    for (c1_val, &start) in ctx.bigram_starts.iter().enumerate().take(65536) {
        let candidates1 = pm.get(c1_val);
        if candidates1.is_empty() { continue; }

        let end = ctx.bigram_starts[c1_val + 1];
        for k in start..end {
            let c2 = ctx.bigram_others[k];
            let candidates2 = pm.get(c2.0 as usize);
            if candidates2.is_empty() { continue; }

            let mut min_cost = Score(i64::MAX);
            for &p1 in candidates1 {
                for &p2 in candidates2 {
                    let idx = (p1 as usize) * ctx.key_count + (p2 as usize);
                    let cost = ctx.cost_matrix[idx];
                    if cost < min_cost { min_cost = cost; }
                }
            }
            let freq = ctx.bigram_freqs[k] as i64;
            total_score = total_score.saturating_add(min_cost.saturating_mul(freq));
        }
    }

    // 3. Trigrams: Optimal Choice
    for (c1_val, &start) in ctx.trigram_starts.iter().enumerate().take(65536) {
        let candidates1 = pm.get(c1_val);
        if candidates1.is_empty() { continue; }

        let end = ctx.trigram_starts[c1_val + 1];
        for k in start..end {
            let c2 = ctx.trigram_others1[k];
            let c3 = ctx.trigram_others2[k];
            let candidates2 = pm.get(c2.0 as usize);
            let candidates3 = pm.get(c3.0 as usize);
            
            if candidates2.is_empty() || candidates3.is_empty() { continue; }

            let mut min_cost = Score(i64::MAX);
            for &p1 in candidates1 {
                for &p2 in candidates2 {
                    for &p3 in candidates3 {
                        let cost = calculate_flow_cost(ctx, p1 as usize, p2 as usize, p3 as usize);
                        if cost < min_cost { min_cost = cost; }
                    }
                }
            }
            if min_cost.0 != i64::MAX && min_cost.0 != 0 {
                let freq = ctx.trigram_freqs[k] as i64;
                total_score = total_score.saturating_add(min_cost.saturating_mul(freq));
            }
        }
    }
    total_score.0
}

#[instrument(skip_all)]
pub fn analyze_layout(ctx: &EngineContext, layout: &ValidatedLayout) -> AnalysisReport {
    let mut report = AnalysisReport::default();
    let layout_slice = layout.as_slice();
    let pm = PosMap::new(layout_slice, ctx.key_count);
    
    let mut heatmap = vec![0.0; ctx.key_count];
    let mut penalty_map = vec![0.0; ctx.key_count];

    let mut total_load = 0.0;
    let mut left_hand_load = 0.0;

    // Monograms
    for (c_val, &freq_u64) in ctx.char_freqs.iter().enumerate() {
        let freq = freq_u64 as f32;
        if freq <= 0.0 { continue; }
        let candidates = pm.get(c_val);
        if candidates.is_empty() { continue; }

        let mut min_cost = f32::MAX;
        let mut best_p = 0;
        for &p in candidates {
            let cost = ctx.key_costs[p as usize].to_f32();
            if cost < min_cost {
                min_cost = cost;
                best_p = p as usize;
            }
        }

        total_load += freq;
        heatmap[best_p] += freq;
        if ctx.hands[best_p].is_left() { left_hand_load += freq; }
        penalty_map[best_p] += freq * min_cost;
    }

    let mut total_bigrams = 0.0;
    let mut sfbs = Vec::new();
    let mut scissors = Vec::new();

    // Bigrams
    for (c1_val, &start) in ctx.bigram_starts.iter().enumerate().take(65536) {
        let candidates1 = pm.get(c1_val);
        if candidates1.is_empty() { continue; }

        let end = ctx.bigram_starts[c1_val + 1];
        for k in start..end {
            let c2 = ctx.bigram_others[k];
            let candidates2 = pm.get(c2.0 as usize);
            if candidates2.is_empty() { continue; }

            let freq = ctx.bigram_freqs[k] as f32;
            total_bigrams += freq;

            let mut min_cost = f32::MAX;
            let mut best_pair = (0, 0);
            for &p1 in candidates1 {
                for &p2 in candidates2 {
                    let cost = ctx.cost_matrix[(p1 as usize) * ctx.key_count + (p2 as usize)].to_f32();
                    if cost < min_cost {
                        min_cost = cost;
                        best_pair = (p1 as usize, p2 as usize);
                    }
                }
            }

            let (idx1, idx2) = best_pair;
            report.distance += min_cost * freq;
            penalty_map[idx1] += min_cost * freq * 0.5;
            penalty_map[idx2] += min_cost * freq * 0.5;

            if ctx.fingers[idx1] == ctx.fingers[idx2] && ctx.hands[idx1] == ctx.hands[idx2] {
                report.sfb_total += freq;
                sfbs.push(MetricViolation {
                    keys: format!("{} {}", c1_val as u8 as char, c2.0 as u8 as char),
                    score: 1.0,
                    freq,
                });
            }
            
            let r1 = ctx.rows[idx1];
            let r2 = ctx.rows[idx2];
            if ctx.hands[idx1] == ctx.hands[idx2] && ctx.fingers[idx1].distance(ctx.fingers[idx2]) == 1 && (r1 - r2).abs() >= 2 {
                report.scissors += freq;
                scissors.push(MetricViolation {
                    keys: format!("{} {}", c1_val as u8 as char, c2.0 as u8 as char),
                    score: 1.0,
                    freq,
                });
            }
        }
    }

    let mut redirs = Vec::new();
    // Trigrams
    for (c1_val, &start) in ctx.trigram_starts.iter().enumerate().take(65536) {
        let candidates1 = pm.get(c1_val);
        if candidates1.is_empty() { continue; }

        let end = ctx.trigram_starts[c1_val + 1];
        for k in start..end {
            let c2 = ctx.trigram_others1[k];
            let c3 = ctx.trigram_others2[k];
            let candidates2 = pm.get(c2.0 as usize);
            let candidates3 = pm.get(c3.0 as usize);
            if candidates2.is_empty() || candidates3.is_empty() { continue; }

            let freq = ctx.trigram_freqs[k] as f32;
            let mut min_cost_val = Score(i64::MAX);
            let mut best_triplet = (0, 0, 0);

            for &p1 in candidates1 {
                for &p2 in candidates2 {
                    for &p3 in candidates3 {
                        let cost = calculate_flow_cost(ctx, p1 as usize, p2 as usize, p3 as usize);
                        if cost < min_cost_val {
                            min_cost_val = cost;
                            best_triplet = (p1 as usize, p2 as usize, p3 as usize);
                        }
                    }
                }
            }

            if min_cost_val.0 != i64::MAX {
                let (idx1, idx2, idx3) = best_triplet;
                let cost_f32 = min_cost_val.to_f32();
                penalty_map[idx1] += cost_f32 * freq * 0.33;
                penalty_map[idx2] += cost_f32 * freq * 0.33;
                penalty_map[idx3] += cost_f32 * freq * 0.33;

                if min_cost_val == ctx.penalty_redirect {
                    report.redirects += freq;
                    redirs.push(MetricViolation {
                        keys: format!(
                            "{}{}{}",
                            c1_val as u8 as char, c2.0 as u8 as char, c3.0 as u8 as char
                        ),
                        score: 1.0,
                        freq,
                    });
                } else if min_cost_val < Score::ZERO {
                    report.rolls += freq;
                }
            }
        }
    }

    let sort_violations = |v: &mut Vec<MetricViolation>| {
        v.sort_by(|a, b| b.freq.partial_cmp(&a.freq).unwrap_or(std::cmp::Ordering::Equal));
        v.truncate(10);
    };
    sort_violations(&mut sfbs);
    sort_violations(&mut scissors);
    sort_violations(&mut redirs);
    
    report.top_sfbs = sfbs;
    report.top_scissors = scissors;
    report.top_redirs = redirs;
    report.heatmap = heatmap;
    report.penalty_map = penalty_map;
    
    let mut scratch = vec![65535u16; 65536];
    report.score = score_layout(ctx, layout, &mut scratch) as f32 / SCORE_SCALE;
    report.distance /= SCORE_SCALE;
    if total_bigrams > 0.0 { report.sfb_ratio = report.sfb_total / total_bigrams; }
    if total_load > 0.0 { report.hand_balance = ((left_hand_load / total_load) - 0.5) * -2.0; }
    report
}

#[inline(always)]
fn calculate_flow_cost(ctx: &EngineContext, p1: usize, p2: usize, p3: usize) -> Score {
    let h1 = ctx.hands[p1];
    let h2 = ctx.hands[p2];
    let h3 = ctx.hands[p3];
    if h1 != h2 || h2 != h3 { return Score::ZERO; }

    if ctx.fingers[p1] == ctx.fingers[p3] && ctx.fingers[p1] != ctx.fingers[p2] { return ctx.penalty_redirect; }
    
    let dir1 = ctx.fingers[p2].diff(ctx.fingers[p1]);
    let dir2 = ctx.fingers[p3].diff(ctx.fingers[p2]);
    if dir1 == 0 || dir2 == 0 { return Score::ZERO; }
    if dir1.signum() != dir2.signum() { return ctx.penalty_redirect; }
    if dir1 < 0 { return Score::ZERO.saturating_sub(ctx.bonus_roll); }
    Score::ZERO
}

pub fn calculate_swap_delta(ctx: &EngineContext, layout: &ValidatedLayout, _pos_map: &[u16], idx_a: usize, idx_b: usize) -> i64 {
    let layout_slice = layout.as_slice();
    if idx_a >= layout_slice.len() || idx_b >= layout_slice.len() { return 0; }
    if idx_a >= ctx.key_count || idx_b >= ctx.key_count { return 0; }
    if layout_slice[idx_a] == layout_slice[idx_b] { return 0; }

    let score_before = score_layout(ctx, layout, &mut []);
    
    let mut swapped_keys = layout_slice.to_vec();
    swapped_keys.swap(idx_a, idx_b);
    let validated_after = ValidatedLayout::new(&swapped_keys, ctx.key_count).unwrap();
    let score_after = score_layout(ctx, &validated_after, &mut []);
    
    score_after - score_before
}