// libs/keyforge-physics/src/kernel/compute.rs

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
use keyforge_model::constants::{MAX_KEYBOARD_KEYS, MAX_REPORTED_VIOLATIONS};
use tracing::instrument;

struct PosMap<'a> {
    starts: &'a [u16],
    counts: &'a [u8],
    indices: &'a [u16],
    used_keys: &'a [u16],
}

impl<'a> PosMap<'a> {
    /// Creates a PosMap by manually populating the provided scratch buffers.
    /// This avoids large array initialization on every call.
    fn from_scratch(
        layout: &[KeyCode],
        key_count: usize,
        starts: &'a mut [u16],
        counts: &'a mut [u8],
        indices: &'a mut [u16],
        used_keys: &'a mut Vec<u16>,
    ) -> Self {
        let limit = layout.len().min(key_count);
        used_keys.clear();

        // Pass 1: Count occurrences
        for &code in layout.iter().take(limit) {
            let c = code.0 as usize;
            if counts[c] == 0 {
                used_keys.push(code.0);
            }
            counts[c] += 1;
        }

        // Pass 2: Calculate starts (prefix sum)
        let mut offset = 0;
        // We only need to iterate over used_keys to set starts
        // But for prefix sum to work correctly with indices, we need them sorted
        used_keys.sort_unstable();
        for &code in used_keys.iter() {
            let c = code as usize;
            starts[c] = offset as u16;
            offset += counts[c] as usize;
        }

        // Pass 3: Fill indices
        // Temporary offset tracker
        let mut current_offsets = [0u8; MAX_KEYBOARD_KEYS];
        for (i, &code) in layout.iter().enumerate().take(limit) {
            let c = code.0 as usize;
            let base = starts[c] as usize;
            // Find current offset for this code. 
            // Since we don't want another 64k array, we can use a small linear search or a map,
            // but for O(N), let's just use the fact that we know which keys are used.
            // Actually, we can just use another scratch buffer of size 512 for 'current_offsets' 
            // if we map keycodes to a 0..used_keys.len() range.
            
            // Optimization: find index of code in used_keys
            if let Ok(u_idx) = used_keys.binary_search(&code.0) {
                let off = current_offsets[u_idx] as usize;
                indices[base + off] = i as u16;
                current_offsets[u_idx] += 1;
            }
        }

        Self { starts, counts, indices, used_keys }
    }

    #[inline(always)]
    fn get(&self, code: usize) -> &[u16] {
        if code >= 65536 { return &[]; }
        let start = self.starts[code] as usize;
        let count = self.counts[code] as usize;
        if count == 0 { return &[]; }
        &self.indices[start..start + count]
    }
}

pub fn score_layout(ctx: &EngineContext, layout: &ValidatedLayout<'_>, scratch: &mut PhysicsScratch) -> i64 {
    let mut total_score = Score::ZERO;
    let layout_slice = layout.as_slice();
    let pm = PosMap::from_scratch(
        layout_slice,
        ctx.key_count,
        &mut scratch.starts,
        &mut scratch.counts,
        &mut scratch.indices,
        &mut scratch.used_keys,
    );

    // 1. Monograms: Optimal Choice
    for &code in pm.used_keys.iter() {
        let c_val = code as usize;
        let freq = ctx.char_freqs[c_val];
        if freq == 0 { continue; }
        let candidates = pm.get(c_val);
        
        let mut min_cost = Score(i64::MAX);
        for &p in candidates {
            let cost = ctx.key_costs[p as usize];
            if cost < min_cost { min_cost = cost; }
        }
        total_score = total_score.saturating_add(min_cost.saturating_mul(freq as i64));
    }

    // 2. Bigrams: Optimal Choice
    for &code1 in pm.used_keys.iter() {
        let c1_val = code1 as usize;
        let candidates1 = pm.get(c1_val);
        let start = ctx.bigram_starts[c1_val];
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
    for &code1 in pm.used_keys.iter() {
        let c1_val = code1 as usize;
        let candidates1 = pm.get(c1_val);
        let start = ctx.trigram_starts[c1_val];
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

    // Clean up scratch for next use
    scratch.clear_used();
    total_score.0
}

/// Scratch space for physics operations to avoid re-allocating large arrays.
pub struct PhysicsScratch {
    starts: [u16; 65536],
    counts: [u8; 65536],
    indices: [u16; MAX_KEYBOARD_KEYS],
    used_keys: Vec<u16>,
    char_usage: [f32; 65536],
}

impl PhysicsScratch {
    pub fn new() -> Self {
        Self {
            starts: [0; 65536],
            counts: [0; 65536],
            indices: [0; MAX_KEYBOARD_KEYS],
            used_keys: Vec::with_capacity(128),
            char_usage: [0.0; 65536],
        }
    }

    fn clear_used(&mut self) {
        for &code in &self.used_keys {
            let c = code as usize;
            self.starts[c] = 0;
            self.counts[c] = 0;
            self.char_usage[c] = 0.0;
        }
    }
}

#[instrument(skip_all)]
pub fn analyze_layout(ctx: &EngineContext, layout: &ValidatedLayout<'_>) -> AnalysisReport {
    let mut report = AnalysisReport::default();
    let mut scratch = PhysicsScratch::new();
    let pm = PosMap::from_scratch(
        layout.as_slice(),
        ctx.key_count,
        &mut scratch.starts,
        &mut scratch.counts,
        &mut scratch.indices,
        &mut scratch.used_keys,
    );
    
    let mut heatmap = vec![0.0; ctx.key_count];
    let mut penalty_map = vec![0.0; ctx.key_count];

    let mut total_load = 0.0;
    let mut left_hand_load = 0.0;
    let mut total_bigrams = 0.0;
    let mut sfbs = Vec::new();
    let mut scissors = Vec::new();
    let mut redirs = Vec::new();

    // Track how much of each bigram's frequency has been handled by Trigrams
    // Key: (char1, char2), Value: consumed_frequency
    let mut bigram_usage: std::collections::HashMap<(u16, u16), f32> = std::collections::HashMap::new();

    // 1. Pass 1: Trigrams (Rigorous Flow & Usage)
    for &(c1, c2, c3, freq) in &ctx.all_trigrams {
        let candidates1 = pm.get(c1 as usize);
        let candidates2 = pm.get(c2 as usize);
        let candidates3 = pm.get(c3 as usize);
        if candidates1.is_empty() || candidates2.is_empty() || candidates3.is_empty() { continue; }

        let freq_f = freq as f32;
        let mut min_cost_val = Score(i64::MAX);
        let mut best_triplet = (0, 0, 0);

        for &p1 in candidates1 {
            for &p2 in candidates2 {
                for &p3 in candidates3 {
                    let mut cost = calculate_flow_cost(ctx, p1 as usize, p2 as usize, p3 as usize);
                    
                    // Add Travel Costs (Bigram Components)
                    let idx12 = (p1 as usize) * ctx.key_count + (p2 as usize);
                    let idx23 = (p2 as usize) * ctx.key_count + (p3 as usize);
                    cost = cost.saturating_add(ctx.cost_matrix[idx12])
                               .saturating_add(ctx.cost_matrix[idx23]);

                    // Add Base Key Costs (Monogram Components)
                    cost = cost.saturating_add(ctx.key_costs[p1 as usize])
                               .saturating_add(ctx.key_costs[p2 as usize])
                               .saturating_add(ctx.key_costs[p3 as usize]);

                    if cost < min_cost_val {
                        min_cost_val = cost;
                        best_triplet = (p1 as usize, p2 as usize, p3 as usize);
                    }
                }
            }
        }

        if min_cost_val.0 != i64::MAX {
            let (idx1, idx2, idx3) = best_triplet;
            
            // Attribute Usage to Target Keys (p2 and p3) based on this Trigram context
            heatmap[idx2] += freq_f;
            heatmap[idx3] += freq_f;
            
            report.distance += ctx.key_home_distances[idx2] * freq_f;
            report.distance += ctx.key_home_distances[idx3] * freq_f;

            scratch.char_usage[c2 as usize] += freq_f;
            scratch.char_usage[c3 as usize] += freq_f;

            // Mark these transitions as accounted for in the Bigram layer
            *bigram_usage.entry((c1, c2)).or_insert(0.0) += freq_f;
            *bigram_usage.entry((c2, c3)).or_insert(0.0) += freq_f;

            // Flow Effort (We attribute the Full Cost including travel here? No, let Bigrams handle Travel Effort)
            // Ideally: Penalty Map = Sum(Base Cost) + Sum(Travel Cost) + Sum(Flow Cost).
            // Base Cost -> Monogram/Bigram loop.
            // Travel Cost -> Bigram Loop.
            // Flow Cost -> Trigram Loop.
            
            // So we should ONLY add the Flow Cost component here.
            let flow_cost = calculate_flow_cost(ctx, idx1, idx2, idx3);
            let flow_cost_f32 = flow_cost.to_f32();
            
            penalty_map[idx1] += flow_cost_f32 * freq_f * 0.33;
            penalty_map[idx2] += flow_cost_f32 * freq_f * 0.33;
            penalty_map[idx3] += flow_cost_f32 * freq_f * 0.33;

            if flow_cost == ctx.penalty_redirect {
                report.redirects += freq_f;
                redirs.push(MetricViolation {
                    keys: format!("{}{}{}", c1 as u8 as char, c2 as u8 as char, c3 as u8 as char),
                    score: 1.0,
                    freq: freq_f,
                });
            } else if flow_cost < Score::ZERO {
                report.rolls += freq_f;
            }
        }
    }

    // 2. Pass 2: Bigrams (Rigorous Physical Metrics & Missing Usage)
    for &(c1, c2, freq) in &ctx.all_bigrams {
        let candidates1 = pm.get(c1 as usize);
        let candidates2 = pm.get(c2 as usize);
        if candidates1.is_empty() || candidates2.is_empty() { continue; }

        let freq_f = freq as f32;
        total_bigrams += freq_f;

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
        
        // Calculate remaining usage not covered by Trigrams
        let consumed = *bigram_usage.get(&(c1, c2)).unwrap_or(&0.0);
        let remaining = (freq_f - consumed).max(0.0);

        if remaining > 0.0 {
            // Attribute remaining usage to the target key (p2)
            heatmap[idx2] += remaining;
            report.distance += ctx.key_home_distances[idx2] * remaining;
            scratch.char_usage[c2 as usize] += remaining;
        }
        
        // Bigram Effort (Travel) - Applied to full frequency to capture SFBs/Scissors accurately
        penalty_map[idx1] += min_cost * freq_f * 0.5;
        penalty_map[idx2] += min_cost * freq_f * 0.5;

        if idx1 != idx2 && ctx.fingers[idx1] == ctx.fingers[idx2] && ctx.hands[idx1] == ctx.hands[idx2] {
            report.sfb_total += freq_f;
            let home_to_k2 = ctx.key_home_distances[idx2];
            let k1_to_k2 = ctx.dist_matrix[idx1 * ctx.key_count + idx2];
            report.distance -= home_to_k2 * freq_f; 
            report.distance += k1_to_k2 * freq_f;   
            
            sfbs.push(MetricViolation {
                keys: format!("{} {}", c1 as u8 as char, c2 as u8 as char),
                score: 1.0,
                freq: freq_f,
            });
        }
        
        let r1 = ctx.rows[idx1];
        let r2 = ctx.rows[idx2];
        if ctx.hands[idx1] == ctx.hands[idx2] && ctx.fingers[idx1].distance(ctx.fingers[idx2]) == 1 && (r1 - r2).abs() >= 2 {
            report.scissors += freq_f;
            scissors.push(MetricViolation {
                keys: format!("{} {}", c1 as u8 as char, c2 as u8 as char),
                score: 1.0,
                freq: freq_f,
            });
        }
    }

    // 3. Pass 3: Monograms (Base Cost & Cleanup)
    // Pass 3 (Revised): Finalize Usage and attribute Base Effort proportional to Heatmap
    for &code in pm.used_keys.iter() {
        let c_val = code as usize;
        let freq = ctx.char_freqs[c_val] as f32;
        if freq <= 0.0 { continue; }

        let remaining = freq - scratch.char_usage[c_val];
        if remaining > 0.0 {
            // Find static best for the remainder
            let candidates = pm.get(c_val);
            let mut min_c = f32::MAX;
            let mut bp = 0;
            for &p in candidates {
                let c = ctx.key_costs[p as usize].to_f32();
                if c < min_c { min_c = c; bp = p as usize; }
            }
            heatmap[bp] += remaining;
            report.distance += ctx.key_home_distances[bp] * remaining;
        }

        // Now attribute Base Effort (Monogram Cost) for this character 
        // distributed across all its keys based on the final heatmap distribution.
        let candidates = pm.get(c_val);
        let char_total_usage: f32 = candidates.iter().map(|&p| heatmap[p as usize]).sum();
        if char_total_usage > 0.0 {
            for &p in candidates {
                let p_idx = p as usize;
                let share = heatmap[p_idx] / char_total_usage;
                penalty_map[p_idx] += freq * share * ctx.key_costs[p_idx].to_f32();
            }
        }
    }

    // Pass 4: Finalize Load Metrics
    for (i, &val) in heatmap.iter().enumerate() {
        total_load += val;
        if ctx.hands[i].is_left() { left_hand_load += val; }
    }

    let sort_violations = |v: &mut Vec<MetricViolation>| {
        v.sort_by(|a, b| b.freq.partial_cmp(&a.freq).unwrap_or(std::cmp::Ordering::Equal));
        v.truncate(MAX_REPORTED_VIOLATIONS);
    };
    sort_violations(&mut sfbs);
    sort_violations(&mut scissors);
    sort_violations(&mut redirs);
    
    // Normalization
    let total_freq: u64 = ctx.char_freqs.iter().sum();
    let norm_100k = if total_freq > 0 { 100_000.0 / total_freq as f32 } else { 1.0 };
    let norm_pct = if total_freq > 0 { 100.0 / total_freq as f32 } else { 1.0 };

    if total_bigrams > 0.0 { report.sfb_ratio = report.sfb_total / total_bigrams; }
    if total_load > 0.0 { report.hand_balance = ((left_hand_load / total_load) - 0.5) * -2.0; }

    for h in &mut heatmap { *h *= norm_pct; }
    for p in &mut penalty_map { *p *= norm_100k; }
    for v in &mut sfbs { v.freq *= norm_pct; }
    for v in &mut scissors { v.freq *= norm_pct; }
    for v in &mut redirs { v.freq *= norm_pct; }

    report.top_sfbs = sfbs;
    report.top_scissors = scissors;
    report.top_redirs = redirs;
    report.heatmap = heatmap;
    report.penalty_map = penalty_map;
    
    // Final Score: Sum of context-aware normalized penalties
    report.score = report.penalty_map.iter().sum();
    
    // Normalized metrics
    report.distance *= norm_100k;
    report.sfb_total *= norm_pct;
    report.scissors *= norm_pct;
    report.redirects *= norm_pct;
    report.rolls *= norm_pct;

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

#[inline(always)]
fn get_p_effective(p: usize, idx_a: usize, idx_b: usize) -> usize {
    if p == idx_a {
        idx_b
    } else if p == idx_b {
        idx_a
    } else {
        p
    }
}

#[inline(always)]
fn get_flow_delta(
    ctx: &EngineContext,
    pos_map: &[u16],
    c1: KeyCode,
    c2: KeyCode,
    c3: KeyCode,
    idx_a: usize,
    idx_b: usize,
) -> i64 {
    let p1 = pos_map[c1.0 as usize] as usize;
    let p2 = pos_map[c2.0 as usize] as usize;
    let p3 = pos_map[c3.0 as usize] as usize;
    if p1 == 65535 || p2 == 65535 || p3 == 65535 {
        return 0;
    }

    let cost_old = calculate_flow_cost(ctx, p1, p2, p3).0;

    let p1_new = get_p_effective(p1, idx_a, idx_b);
    let p2_new = get_p_effective(p2, idx_a, idx_b);
    let p3_new = get_p_effective(p3, idx_a, idx_b);

    let cost_new = calculate_flow_cost(ctx, p1_new, p2_new, p3_new).0;
    cost_new - cost_old
}

pub fn calculate_swap_delta(
    ctx: &EngineContext,
    layout: &ValidatedLayout<'_>,
    pos_map: &[u16],
    idx_a: usize,
    idx_b: usize,
) -> i64 {
    let layout_slice = layout.as_slice();
    if idx_a >= layout_slice.len() || idx_b >= layout_slice.len() {
        return 0;
    }
    let code_a = layout_slice[idx_a];
    let code_b = layout_slice[idx_b];
    if code_a == code_b {
        return 0;
    }

    let mut delta = 0i64;
    let n = ctx.key_count;

    // 1. Monograms
    let freq_a = ctx.char_freqs[code_a.0 as usize] as i64;
    let freq_b = ctx.char_freqs[code_b.0 as usize] as i64;
    delta += (ctx.key_costs[idx_b].0 - ctx.key_costs[idx_a].0) * freq_a;
    delta += (ctx.key_costs[idx_a].0 - ctx.key_costs[idx_b].0) * freq_b;

    // 2. Bigrams
    let start_a = ctx.bigram_starts[code_a.0 as usize];
    let end_a = ctx.bigram_starts[code_a.0 as usize + 1];
    for k in start_a..end_a {
        let c2 = ctx.bigram_others[k];
        let p2 = pos_map[c2.0 as usize] as usize;
        if p2 == 65535 {
            continue;
        }
        let freq = ctx.bigram_freqs[k] as i64;
        let p2_effective = if p2 == idx_b {
            idx_a
        } else if p2 == idx_a {
            idx_b
        } else {
            p2
        };
        delta += (ctx.cost_matrix[idx_b * n + p2_effective].0 - ctx.cost_matrix[idx_a * n + p2].0)
            * freq;
    }

    let start_b = ctx.bigram_starts[code_b.0 as usize];
    let end_b = ctx.bigram_starts[code_b.0 as usize + 1];
    for k in start_b..end_b {
        let c2 = ctx.bigram_others[k];
        let p2 = pos_map[c2.0 as usize] as usize;
        if p2 == 65535 {
            continue;
        }
        let freq = ctx.bigram_freqs[k] as i64;
        let p2_effective = if p2 == idx_a {
            idx_b
        } else if p2 == idx_b {
            idx_a
        } else {
            p2
        };
        delta += (ctx.cost_matrix[idx_a * n + p2_effective].0 - ctx.cost_matrix[idx_b * n + p2].0)
            * freq;
    }

    let start_rev_a = ctx.bigram_rev_starts[code_a.0 as usize];
    let end_rev_a = ctx.bigram_rev_starts[code_a.0 as usize + 1];
    for k in start_rev_a..end_rev_a {
        let c1 = ctx.bigram_rev_others[k];
        if c1 == code_a || c1 == code_b {
            continue;
        }
        let p1 = pos_map[c1.0 as usize] as usize;
        if p1 == 65535 {
            continue;
        }
        let freq = ctx.bigram_rev_freqs[k] as i64;
        delta += (ctx.cost_matrix[p1 * n + idx_b].0 - ctx.cost_matrix[p1 * n + idx_a].0) * freq;
    }

    let start_rev_b = ctx.bigram_rev_starts[code_b.0 as usize];
    let end_rev_b = ctx.bigram_rev_starts[code_b.0 as usize + 1];
    for k in start_rev_b..end_rev_b {
        let c1 = ctx.bigram_rev_others[k];
        if c1 == code_a || c1 == code_b {
            continue;
        }
        let p1 = pos_map[c1.0 as usize] as usize;
        if p1 == 65535 {
            continue;
        }
        let freq = ctx.bigram_rev_freqs[k] as i64;
        delta += (ctx.cost_matrix[p1 * n + idx_a].0 - ctx.cost_matrix[p1 * n + idx_b].0) * freq;
    }

    // 3. Trigrams (Incremental)
    if !ctx.trigram_freqs.is_empty() {
        let ca = code_a.0 as usize;
        let cb = code_b.0 as usize;

        // Starts(a)
        let s_a = ctx.trigram_starts[ca];
        let e_a = ctx.trigram_starts[ca + 1];
        for k in s_a..e_a {
            let c2 = ctx.trigram_others1[k];
            let c3 = ctx.trigram_others2[k];
            let freq = ctx.trigram_freqs[k] as i64;
            delta += get_flow_delta(ctx, pos_map, code_a, c2, c3, idx_a, idx_b) * freq;
        }

        // Starts(b)
        let s_b = ctx.trigram_starts[cb];
        let e_b = ctx.trigram_starts[cb + 1];
        for k in s_b..e_b {
            let c2 = ctx.trigram_others1[k];
            let c3 = ctx.trigram_others2[k];
            let freq = ctx.trigram_freqs[k] as i64;
            delta += get_flow_delta(ctx, pos_map, code_b, c2, c3, idx_a, idx_b) * freq;
        }

        // Mid(a) where c1 != a and c1 != b
        let s_ma = ctx.trigram_mid_starts[ca];
        let e_ma = ctx.trigram_mid_starts[ca + 1];
        for k in s_ma..e_ma {
            let c1 = ctx.trigram_mid_others1[k];
            if c1 == code_a || c1 == code_b {
                continue;
            }
            let c3 = ctx.trigram_mid_others2[k];
            let freq = ctx.trigram_mid_freqs[k] as i64;
            delta += get_flow_delta(ctx, pos_map, c1, code_a, c3, idx_a, idx_b) * freq;
        }

        // Mid(b) where c1 != a and c1 != b
        let s_mb = ctx.trigram_mid_starts[cb];
        let e_mb = ctx.trigram_mid_starts[cb + 1];
        for k in s_mb..e_mb {
            let c1 = ctx.trigram_mid_others1[k];
            if c1 == code_a || c1 == code_b {
                continue;
            }
            let c3 = ctx.trigram_mid_others2[k];
            let freq = ctx.trigram_mid_freqs[k] as i64;
            delta += get_flow_delta(ctx, pos_map, c1, code_b, c3, idx_a, idx_b) * freq;
        }

        // Ends(a) where c1 != a, c1 != b, c2 != a, c2 != b
        let s_ea = ctx.trigram_end_starts[ca];
        let e_ea = ctx.trigram_end_starts[ca + 1];
        for k in s_ea..e_ea {
            let c1 = ctx.trigram_end_others1[k];
            let c2 = ctx.trigram_end_others2[k];
            if c1 == code_a || c1 == code_b || c2 == code_a || c2 == code_b {
                continue;
            }
            let freq = ctx.trigram_end_freqs[k] as i64;
            delta += get_flow_delta(ctx, pos_map, c1, c2, code_a, idx_a, idx_b) * freq;
        }

        // Ends(b) where c1 != a, c1 != b, c2 != a, c2 != b
        let s_eb = ctx.trigram_end_starts[cb];
        let e_eb = ctx.trigram_end_starts[cb + 1];
        for k in s_eb..e_eb {
            let c1 = ctx.trigram_end_others1[k];
            let c2 = ctx.trigram_end_others2[k];
            if c1 == code_a || c1 == code_b || c2 == code_a || c2 == code_b {
                continue;
            }
            let freq = ctx.trigram_end_freqs[k] as i64;
            delta += get_flow_delta(ctx, pos_map, c1, c2, code_b, idx_a, idx_b) * freq;
        }
    }

    delta
}
