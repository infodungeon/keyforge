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

use super::types::{KeyCode, Score, ValidatedLayout, FingerIndex};
use super::EngineContext;
use keyforge_model::{AnalysisReport, MetricViolation};
use keyforge_model::constants::{MAX_KEYBOARD_KEYS, MAX_REPORTED_VIOLATIONS};
use tracing::instrument;

pub(crate) struct PosMap<'a> {
    pub(crate) starts: &'a [u16],
    pub(crate) counts: &'a [u8],
    pub(crate) indices: &'a [u16],
    pub(crate) used_keys: &'a [u16],
}

impl<'a> PosMap<'a> {
    /// Creates a PosMap by manually populating the provided scratch buffers.
    /// This avoids large array initialization on every call.
    pub(crate) fn from_scratch(
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
    pub(crate) fn get(&self, code: usize) -> &[u16] {
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
    pub(crate) starts: [u16; 65536],
    pub(crate) counts: [u8; 65536],
    pub(crate) indices: [u16; MAX_KEYBOARD_KEYS],
    pub(crate) used_keys: Vec<u16>,
    pub(crate) char_usage: [f32; 65536],
}

impl Default for PhysicsScratch {
    fn default() -> Self {
        Self {
            starts: [0; 65536],
            counts: [0; 65536],
            indices: [0; MAX_KEYBOARD_KEYS],
            used_keys: Vec::with_capacity(MAX_KEYBOARD_KEYS),
            char_usage: [0.0; 65536],
        }
    }
}

impl PhysicsScratch {
    /// Creates a new scratch instance.
    pub fn new() -> Self {
        Self::default()
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

/// Safely converts a u16 character code to a displayable character.
/// Handles invalid Unicode surrogate pairs and control characters.
#[inline]
fn u16_to_char(code: u16) -> String {
    // Try direct conversion (for ASCII and most Unicode)
    if let Some(c) = char::from_u32(code as u32) {
        // Filter out control characters that aren't printable
        if !c.is_control() {
            return c.to_string();
        }
        // Special handling for common control characters
        match code {
            8 => return "⌫".to_string(),   // Backspace
            9 => return "⇥".to_string(),   // Tab
            10 => return "↵".to_string(),  // Newline
            32 => return "␣".to_string(),  // Space
            _ => return format!("[0x{:02X}]", code),
        }
    }
    // Fallback for invalid Unicode (like surrogate pairs)
    format!("[0x{:04X}]", code)
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

    // 1. Pass 1: Trigrams (Flow ONLY)
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
                    // Score includes flow penalty and travel costs
                    let mut cost = calculate_flow_cost(ctx, p1 as usize, p2 as usize, p3 as usize);
                    let idx12 = (p1 as usize) * ctx.key_count + (p2 as usize);
                    let idx23 = (p2 as usize) * ctx.key_count + (p3 as usize);
                    cost = cost.saturating_add(ctx.cost_matrix[idx12])
                               .saturating_add(ctx.cost_matrix[idx23]);

                    if cost < min_cost_val {
                        min_cost_val = cost;
                        best_triplet = (p1 as usize, p2 as usize, p3 as usize);
                    }
                }
            }
        }

        if min_cost_val.0 != i64::MAX {
            let (idx1, idx2, idx3) = best_triplet;
            
            // Flow Effort (Redirects/Rolls) - distributed across triplet
            let flow_cost = calculate_flow_cost(ctx, idx1, idx2, idx3);
            let flow_cost_f32 = flow_cost.to_f32();
            penalty_map[idx1] += flow_cost_f32 * freq_f * 0.33;
            penalty_map[idx2] += flow_cost_f32 * freq_f * 0.33;
            penalty_map[idx3] += flow_cost_f32 * freq_f * 0.33;

            if flow_cost == ctx.penalty_redirect {
                report.redirects += freq_f;
                
                // Accumulate redirect penalty contribution
                report.redir_penalty += flow_cost_f32 * freq_f;
                
                redirs.push(MetricViolation {
                    keys: format!("{}{}{}", u16_to_char(c1), u16_to_char(c2), u16_to_char(c3)),
                    score: flow_cost_f32 * freq_f,
                    freq: freq_f,
                });
            } else if flow_cost < Score::ZERO {
                report.rolls += freq_f;
                
                // Accumulate roll penalty contribution (negative, so it's a bonus)
                report.roll_penalty += flow_cost_f32 * freq_f;
            }
        }
    }

    // 2. Pass 2: Bigrams (ALL TRANSITIONS, DISTANCE, USAGE)
    for &(c1, c2, freq) in &ctx.all_bigrams {
        let candidates1 = pm.get(c1 as usize);
        let candidates2 = pm.get(c2 as usize);
        if candidates1.is_empty() || candidates2.is_empty() { continue; }

        let freq_f = freq as f32;
        total_bigrams += freq_f;

        // Choose OPTIMAL key pair by evaluating candidate costs
        let mut min_score = Score(i64::MAX);
        let mut best_pair = (0, 0);

        if candidates1.len() == 1 && candidates2.len() == 1 {
            // Case 1: Single Key - Irrelevant to evaluate choice
            best_pair = (candidates1[0] as usize, candidates2[0] as usize);
        } else {
            // Case 2: Multiple Selection (Duplicated Keys like Space)
            // Pick pair resulting in best score contribution
            for &p1 in candidates1 {
                for &p2 in candidates2 {
                    let cost = ctx.cost_matrix[(p1 as usize) * ctx.key_count + (p2 as usize)];
                    if cost < min_score {
                        min_score = cost;
                        best_pair = (p1 as usize, p2 as usize);
                    }
                }
            }
        }

        let (idx1, idx2) = best_pair;
        
        // --- TRANSITION ACCOUNTING ---
        // Usage (Heatmap) attributed to target character c2
        heatmap[idx2] += freq_f;
        scratch.char_usage[c2 as usize] += freq_f;

        // Distance Calculation
        if idx1 == idx2 {
            // Same key: No movement
        } else if ctx.hands[idx1] == ctx.hands[idx2] {
            // Same Hand: Euclidean Distance
            report.distance += ctx.dist_matrix[idx1 * ctx.key_count + idx2] * freq_f;
            
            // SFB Check (Specific to same-finger move)
            if ctx.fingers[idx1] == ctx.fingers[idx2] {
                report.sfb_total += freq_f;
                
                // Accumulate SFB penalty contribution
                let sfb_cost = ctx.cost_matrix[idx1 * ctx.key_count + idx2].to_f32();
                report.sfb_penalty += sfb_cost * freq_f;
                
                sfbs.push(MetricViolation {
                    keys: format!("{}{}", u16_to_char(c1), u16_to_char(c2)),
                    score: sfb_cost * freq_f,
                    freq: freq_f,
                });
            }
        } else {
            // Different Hand: Movement from home position
            report.distance += ctx.key_home_distances[idx2] * freq_f;
        }

        // Scissor Detection
        let r1 = ctx.rows[idx1];
        let r2 = ctx.rows[idx2];
        let f1 = ctx.fingers[idx1];
        let f2 = ctx.fingers[idx2];
        if ctx.hands[idx1] == ctx.hands[idx2] 
            && f1.distance(f2) == 1 
            && (r1 - r2).abs() >= 2
            && f1 != FingerIndex::THUMB
            && f2 != FingerIndex::THUMB 
        {
            report.scissors += freq_f;
            
            // Accumulate scissor penalty contribution
            let scissor_cost = ctx.cost_matrix[idx1 * ctx.key_count + idx2].to_f32();
            report.scissor_penalty += scissor_cost * freq_f;
            
            scissors.push(MetricViolation {
                keys: format!("{}{}", u16_to_char(c1), u16_to_char(c2)),
                score: scissor_cost * freq_f,
                freq: freq_f,
            });
        }

        // Effort Attribution
        let trans_cost = ctx.cost_matrix[idx1 * ctx.key_count + idx2].to_f32();
        penalty_map[idx1] += trans_cost * freq_f * 0.5;
        penalty_map[idx2] += trans_cost * freq_f * 0.5;
    }

    // 3. Pass 3: Monograms (Base Usage & Remaining Characters)
    for &code in pm.used_keys.iter() {
        let c_val = code as usize;
        let freq = ctx.char_freqs[c_val] as f32;
        if freq <= 0.0 { continue; }

        let candidates = pm.get(c_val);
        
        // Monogram Effort (Base Key Cost)
        // Attribute to keys based on their usage heatmap or unique position
        let total_key_usage: f32 = candidates.iter().map(|&p| heatmap[p as usize]).sum();
        if total_key_usage > 0.0 {
            for &p in candidates {
                let p_idx = p as usize;
                let share = heatmap[p_idx] / total_key_usage;
                penalty_map[p_idx] += freq * share * ctx.key_costs[p_idx].to_f32();
            }
        } else {
            // Unused in transitions (e.g. monogram only): use best static key
            let mut min_c = f32::MAX;
            let mut bp = 0;
            for &p in candidates {
                let c = ctx.key_costs[p as usize].to_f32();
                if c < min_c { min_c = c; bp = p as usize; }
            }
            heatmap[bp] += freq;
            penalty_map[bp] += freq * ctx.key_costs[bp].to_f32();
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
    if total_freq > 0 {
        report.travel_per_key = report.distance / total_freq as f32;
    }
    let norm_100k = if total_freq > 0 { 100_000.0 / total_freq as f32 } else { 1.0 };
    let norm_pct = if total_freq > 0 { 100.0 / total_freq as f32 } else { 1.0 };

    if total_bigrams > 0.0 { report.sfb_ratio = report.sfb_total / total_bigrams; }
    if total_load > 0.0 { report.hand_balance = ((left_hand_load / total_load) - 0.5) * -2.0; }

    for h in &mut heatmap { *h *= norm_pct; }
    for p in &mut penalty_map { *p *= norm_100k; }
    for v in &mut sfbs { v.freq *= norm_pct; v.score *= norm_100k; }
    for v in &mut scissors { v.freq *= norm_pct; v.score *= norm_100k; }
    for v in &mut redirs { v.freq *= norm_pct; v.score *= norm_100k; }

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
    
    // Normalize penalty contributions to match score scale
    report.sfb_penalty *= norm_100k;
    report.scissor_penalty *= norm_100k;
    report.redir_penalty *= norm_100k;
    report.roll_penalty *= norm_100k;

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
    pos_map: &PosMap<'_>,
    c1: KeyCode,
    c2: KeyCode,
    c3: KeyCode,
    idx_a: usize,
    idx_b: usize,
) -> i64 {
    let candidates1 = pos_map.get(c1.0 as usize);
    let candidates2 = pos_map.get(c2.0 as usize);
    let candidates3 = pos_map.get(c3.0 as usize);
    if candidates1.is_empty() || candidates2.is_empty() || candidates3.is_empty() {
        return 0;
    }

    let mut min_old = Score(i64::MAX);
    for &p1 in candidates1 {
        for &p2 in candidates2 {
            for &p3 in candidates3 {
                let cost = calculate_flow_cost(ctx, p1 as usize, p2 as usize, p3 as usize);
                if cost < min_old { min_old = cost; }
            }
        }
    }

    let mut min_new = Score(i64::MAX);
    for &p1 in candidates1 {
        for &p2 in candidates2 {
            for &p3 in candidates3 {
                let p1_new = get_p_effective(p1 as usize, idx_a, idx_b);
                let p2_new = get_p_effective(p2 as usize, idx_a, idx_b);
                let p3_new = get_p_effective(p3 as usize, idx_a, idx_b);
                let cost = calculate_flow_cost(ctx, p1_new, p2_new, p3_new);
                if cost < min_new { min_new = cost; }
            }
        }
    }

    min_new.0 - min_old.0
}

pub(crate) fn calculate_swap_delta(
    ctx: &EngineContext,
    layout: &ValidatedLayout<'_>,
    pos_map: &PosMap<'_>,
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

    // 1. Monograms
    let freq_a = ctx.char_freqs[code_a.0 as usize] as i64;
    let freq_b = ctx.char_freqs[code_b.0 as usize] as i64;

    let candidates_a = pos_map.get(code_a.0 as usize);
    let candidates_b = pos_map.get(code_b.0 as usize);

    // code_a delta
    let mut min_old_a = Score(i64::MAX);
    let mut min_new_a = Score(i64::MAX);
    for &p in candidates_a {
        let p_idx = p as usize;
        let c_old = ctx.key_costs[p_idx];
        if c_old < min_old_a { min_old_a = c_old; }
        
        let p_new = get_p_effective(p_idx, idx_a, idx_b);
        let c_new = ctx.key_costs[p_new];
        if c_new < min_new_a { min_new_a = c_new; }
    }
    delta += (min_new_a.0 - min_old_a.0) * freq_a;

    // code_b delta
    let mut min_old_b = Score(i64::MAX);
    let mut min_new_b = Score(i64::MAX);
    for &p in candidates_b {
        let p_idx = p as usize;
        let c_old = ctx.key_costs[p_idx];
        if c_old < min_old_b { min_old_b = c_old; }

        let p_new = get_p_effective(p_idx, idx_a, idx_b);
        let c_new = ctx.key_costs[p_new];
        if c_new < min_new_b { min_new_b = c_new; }
    }
    delta += (min_new_b.0 - min_old_b.0) * freq_b;

    // 2. Bigrams
    // Bigrams(a, x)
    let start_a = ctx.bigram_starts[code_a.0 as usize];
    let end_a = ctx.bigram_starts[code_a.0 as usize + 1];
    for k in start_a..end_a {
        let c2 = ctx.bigram_others[k];
        let candidates2 = pos_map.get(c2.0 as usize);
        if candidates2.is_empty() { continue; }

        let mut min_old = Score(i64::MAX);
        let mut min_new = Score(i64::MAX);
        for &p1 in candidates_a {
            let p1_new = get_p_effective(p1 as usize, idx_a, idx_b);
            for &p2 in candidates2 {
                let p2_new = get_p_effective(p2 as usize, idx_a, idx_b);
                
                let cost_old = ctx.cost_matrix[(p1 as usize) * ctx.key_count + (p2 as usize)];
                if cost_old < min_old { min_old = cost_old; }
                
                let cost_new = ctx.cost_matrix[p1_new * ctx.key_count + p2_new];
                if cost_new < min_new { min_new = cost_new; }
            }
        }
        delta += (min_new.0 - min_old.0) * ctx.bigram_freqs[k] as i64;
    }

    // Bigrams(b, x)
    let start_b = ctx.bigram_starts[code_b.0 as usize];
    let end_b = ctx.bigram_starts[code_b.0 as usize + 1];
    for k in start_b..end_b {
        let c2 = ctx.bigram_others[k];
        let candidates2 = pos_map.get(c2.0 as usize);
        if candidates2.is_empty() { continue; }

        let mut min_old = Score(i64::MAX);
        let mut min_new = Score(i64::MAX);
        for &p1 in candidates_b {
            let p1_new = get_p_effective(p1 as usize, idx_a, idx_b);
            for &p2 in candidates2 {
                let p2_new = get_p_effective(p2 as usize, idx_a, idx_b);
                
                let cost_old = ctx.cost_matrix[(p1 as usize) * ctx.key_count + (p2 as usize)];
                if cost_old < min_old { min_old = cost_old; }
                
                let cost_new = ctx.cost_matrix[p1_new * ctx.key_count + p2_new];
                if cost_new < min_new { min_new = cost_new; }
            }
        }
        delta += (min_new.0 - min_old.0) * ctx.bigram_freqs[k] as i64;
    }

    // Bigrams(x, a) where x != a, x != b
    let start_rev_a = ctx.bigram_rev_starts[code_a.0 as usize];
    let end_rev_a = ctx.bigram_rev_starts[code_a.0 as usize + 1];
    for k in start_rev_a..end_rev_a {
        let c1 = ctx.bigram_rev_others[k];
        if c1 == code_a || c1 == code_b { continue; }
        let candidates1 = pos_map.get(c1.0 as usize);
        if candidates1.is_empty() { continue; }

        let mut min_old = Score(i64::MAX);
        let mut min_new = Score(i64::MAX);
        for &p1 in candidates1 {
            let p1_new = get_p_effective(p1 as usize, idx_a, idx_b);
            for &p2 in candidates_a {
                let p2_new = get_p_effective(p2 as usize, idx_a, idx_b);
                
                let cost_old = ctx.cost_matrix[(p1 as usize) * ctx.key_count + (p2 as usize)];
                if cost_old < min_old { min_old = cost_old; }
                
                let cost_new = ctx.cost_matrix[p1_new * ctx.key_count + p2_new];
                if cost_new < min_new { min_new = cost_new; }
            }
        }
        delta += (min_new.0 - min_old.0) * ctx.bigram_rev_freqs[k] as i64;
    }

    // Bigrams(x, b) where x != a, x != b
    let start_rev_b = ctx.bigram_rev_starts[code_b.0 as usize];
    let end_rev_b = ctx.bigram_rev_starts[code_b.0 as usize + 1];
    for k in start_rev_b..end_rev_b {
        let c1 = ctx.bigram_rev_others[k];
        if c1 == code_a || c1 == code_b { continue; }
        let candidates1 = pos_map.get(c1.0 as usize);
        if candidates1.is_empty() { continue; }

        let mut min_old = Score(i64::MAX);
        let mut min_new = Score(i64::MAX);
        for &p1 in candidates1 {
            let p1_new = get_p_effective(p1 as usize, idx_a, idx_b);
            for &p2 in candidates_b {
                let p2_new = get_p_effective(p2 as usize, idx_a, idx_b);
                
                let cost_old = ctx.cost_matrix[(p1 as usize) * ctx.key_count + (p2 as usize)];
                if cost_old < min_old { min_old = cost_old; }
                
                let cost_new = ctx.cost_matrix[p1_new * ctx.key_count + p2_new];
                if cost_new < min_new { min_new = cost_new; }
            }
        }
        delta += (min_new.0 - min_old.0) * ctx.bigram_rev_freqs[k] as i64;
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
            if c1 == code_a || c1 == code_b { continue; }
            let c3 = ctx.trigram_mid_others2[k];
            let freq = ctx.trigram_mid_freqs[k] as i64;
            delta += get_flow_delta(ctx, pos_map, c1, code_a, c3, idx_a, idx_b) * freq;
        }

        // Mid(b) where c1 != a and c1 != b
        let s_mb = ctx.trigram_mid_starts[cb];
        let e_mb = ctx.trigram_mid_starts[cb + 1];
        for k in s_mb..e_mb {
            let c1 = ctx.trigram_mid_others1[k];
            if c1 == code_a || c1 == code_b { continue; }
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
            if c1 == code_a || c1 == code_b || c2 == code_a || c2 == code_b { continue; }
            let freq = ctx.trigram_end_freqs[k] as i64;
            delta += get_flow_delta(ctx, pos_map, c1, c2, code_a, idx_a, idx_b) * freq;
        }

        // Ends(b) where c1 != a, c1 != b, c2 != a, c2 != b
        let s_eb = ctx.trigram_end_starts[cb];
        let e_eb = ctx.trigram_end_starts[cb + 1];
        for k in s_eb..e_eb {
            let c1 = ctx.trigram_end_others1[k];
            let c2 = ctx.trigram_end_others2[k];
            if c1 == code_a || c1 == code_b || c2 == code_a || c2 == code_b { continue; }
            let freq = ctx.trigram_end_freqs[k] as i64;
            delta += get_flow_delta(ctx, pos_map, c1, c2, code_b, idx_a, idx_b) * freq;
        }
    }

    delta
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ScoringEngine;
    use keyforge_model::{
        Corpus, KeyNode, Keyboard, Layout, Rubric, CostModel,
        types::{HandIndex, FingerIndex, KeyCode}
    };

    fn setup_kb_robust() -> Keyboard {
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
    fn test_math_boundaries_infinity() {
        let kb = setup_kb_robust();
        let layout = Layout::new_unchecked(vec![KeyCode(97), KeyCode(98), KeyCode(99), KeyCode(100), KeyCode(101)]);
        let mut corpus = Corpus::default();
        corpus.bigrams.push((97, 98, 1000));

        let rubric = Rubric {
            travel_lat: f32::INFINITY,
            ..Rubric::default()
        };

        let engine = ScoringEngine::new(&kb, &corpus, &rubric, &mock_cost_model()).unwrap();
        let score = engine.score(&layout).unwrap();

        assert!(score > 1_000_000.0);
        assert!(score.is_finite());
    }

    #[test]
    fn test_math_boundaries_nan() {
        let kb = setup_kb_robust();
        let layout = Layout::new_unchecked(vec![KeyCode(97), KeyCode(98), KeyCode(99), KeyCode(100), KeyCode(101)]);
        let mut corpus = Corpus::default();
        corpus.bigrams.push((97, 98, 1000));

        let rubric = Rubric {
            travel_lat: f32::NAN,
            ..Rubric::default()
        };

        let engine = ScoringEngine::new(&kb, &corpus, &rubric, &mock_cost_model()).unwrap();
        let score = engine.score(&layout).unwrap();

        assert!(score >= 0.0);
        assert!(!score.is_nan());
    }

    #[test]
    fn test_saturation_protection() {
        let kb = setup_kb_robust();
        let layout = Layout::new_unchecked(vec![KeyCode(97), KeyCode(98), KeyCode(99), KeyCode(100), KeyCode(101)]);
        let mut corpus = Corpus::default();
        corpus.bigrams.push((97, 98, u32::MAX));

        let rubric = Rubric {
            travel_lat: 1_000_000.0,
            ..Rubric::default()
        };

        let engine = ScoringEngine::new(&kb, &corpus, &rubric, &mock_cost_model()).unwrap();
        let score = engine.score(&layout).unwrap();
        assert!(score.is_finite());
    }

    #[test]
    fn test_missing_keys_in_layout() {
        let kb = setup_kb_robust();
        let layout = Layout::new_unchecked(vec![KeyCode(97), KeyCode(0), KeyCode(99), KeyCode(100), KeyCode(101)]);
        let mut corpus = Corpus::default();
        corpus.bigrams.push((97, 98, 100)); 

        let engine = ScoringEngine::new(&kb, &corpus, &Rubric::default(), &mock_cost_model()).unwrap();
        let score = engine.score(&layout).unwrap();
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_swap_delta_bounds() {
        let kb = setup_kb_robust();
        let layout = Layout::new_unchecked(vec![KeyCode(97), KeyCode(98), KeyCode(99), KeyCode(100), KeyCode(101)]);
        let mut corpus = Corpus::default();
        corpus.bigrams.push((97, 98, 100));

        let engine = ScoringEngine::new(&kb, &corpus, &Rubric::default(), &mock_cost_model()).unwrap();
        let mut pos_map_data = vec![65535u16; 65536];
        for (i, &code) in layout.keys.iter().enumerate() {
            pos_map_data[code.0 as usize] = i as u16;
        }

        let validated = ValidatedLayout::new(&layout.keys, engine.key_count()).unwrap();
        let mut starts = [0u16; 65536];
        let mut counts = [0u8; 65536];
        let mut indices = [0u16; 512];
        let mut used_keys = Vec::new();
        let pm = PosMap::from_scratch(&layout.keys, engine.key_count(), &mut starts, &mut counts, &mut indices, &mut used_keys);

        let delta = calculate_swap_delta(engine.context(), &validated, &pm, 0, 100);
        assert_eq!(delta, 0);
    }

    #[test]
    fn test_analyze_layout_empty() {
        let kb = setup_kb_robust();
        let corpus = Corpus::default();
        let rubric = Rubric::default();
        let engine = ScoringEngine::new(&kb, &corpus, &rubric, &mock_cost_model()).unwrap();
        
        let layout = Layout::new_unchecked(vec![]);
        let validated_res = ValidatedLayout::new(&layout.keys, engine.key_count());
        assert!(validated_res.is_err());
    }

    #[test]
    fn test_delta_internals_manual() {
        let keys = vec![
            KeyNode { index: 0, x: 0.0, ..Default::default() },
            KeyNode { index: 1, x: 10.0, ..Default::default() },
            KeyNode { index: 2, x: 20.0, ..Default::default() },
        ];
        let kb = Keyboard::new(keys, 0).unwrap();
        
        let mut corpus = Corpus::default();
        corpus.bigrams.push((0, 1, 100));
        corpus.trigrams.push((0, 1, 2, 100));
        
        let mut rubric = Rubric::default();
        rubric.travel_lat = 1.0;
        
        let engine = ScoringEngine::new(&kb, &corpus, &rubric, &mock_cost_model()).unwrap();
        
        let layout_keys = vec![KeyCode(0), KeyCode(1), KeyCode(2)];
        let mut pos_map_data = vec![65535u16; 65536];
        pos_map_data[0] = 0; pos_map_data[1] = 1; pos_map_data[2] = 2;
        
        let validated = ValidatedLayout::new(&layout_keys, engine.key_count()).unwrap();
        let mut starts = [0u16; 65536];
        let mut counts = [0u8; 65536];
        let mut indices = [0u16; 512];
        let mut used_keys = Vec::new();
        let pm = PosMap::from_scratch(&layout_keys, engine.key_count(), &mut starts, &mut counts, &mut indices, &mut used_keys);

        let score_before = engine.score_raw(&layout_keys).unwrap();
        let delta = calculate_swap_delta(engine.context(), &validated, &pm, 1, 2);
        
        let mut swapped_keys = layout_keys.clone();
        swapped_keys.swap(1, 2);
        let score_after = engine.score_raw(&swapped_keys).unwrap();
        
        assert_eq!(score_after - score_before, delta, "Manual delta check failed");
    }

    #[test]
    fn test_delta_self_loop() {
        let keys = vec![
            KeyNode { index: 0, x: 0.0, ..Default::default() },
            KeyNode { index: 1, x: 10.0, ..Default::default() },
        ];
        let kb = Keyboard::new(keys, 0).unwrap();
        
        let mut corpus = Corpus::default();
        corpus.bigrams.push((0, 0, 100));
        
        let mut rubric = Rubric::default();
        rubric.travel_lat = 1.0;
        rubric.trigram_limit = 0; 
        
        let engine = ScoringEngine::new(&kb, &corpus, &rubric, &mock_cost_model()).unwrap();
        
        let layout_keys = vec![KeyCode(0), KeyCode(1)];
        let mut pos_map_data = vec![65535u16; 65536];
        pos_map_data[0] = 0; pos_map_data[1] = 1;
        
        let validated = ValidatedLayout::new(&layout_keys, engine.key_count()).unwrap();
        let mut starts = [0u16; 65536];
        let mut counts = [0u8; 65536];
        let mut indices = [0u16; 512];
        let mut used_keys = Vec::new();
        let pm = PosMap::from_scratch(&layout_keys, engine.key_count(), &mut starts, &mut counts, &mut indices, &mut used_keys);

        let score_before = engine.score_raw(&layout_keys).unwrap();
        let delta = calculate_swap_delta(engine.context(), &validated, &pm, 0, 1);
        
        let mut swapped_keys = layout_keys.clone();
        swapped_keys.swap(0, 1);
        let score_after = engine.score_raw(&swapped_keys).unwrap();
        
        assert_eq!(score_after - score_before, delta, "Self loop delta check failed");
    }
}
