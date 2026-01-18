use crate::kernel::{EngineContext, types::{Score, ValidatedLayout}};
use super::state::{PosMap, PhysicsScratch};
use super::flow::calculate_flow_cost;

pub fn score_layout(ctx: &EngineContext, layout: &ValidatedLayout<'_>, scratch: &mut PhysicsScratch) -> i64 {
    let mut total_score = Score::ZERO;
    let layout_slice = layout.as_slice();
    let pm = PosMap::from_scratch(
        layout_slice,
        ctx.key_count,
        &mut scratch.starts,
        &mut scratch.counts,
        &mut scratch.indices,
        &mut scratch.current_offsets,
        &ctx.sorted_unique_keys,
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
                    let mut cost = ctx.cost_matrix[idx];
                    
                    // Apply sequence-specific modifiers (e.g. from biometrics)
                    if let Some(&mod_val) = ctx.sequence_modifiers.get(&(code1, c2.0)) {
                        cost = cost.saturating_add(mod_val);
                    }

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

            if candidates1.len() * candidates2.len() * candidates3.len() > 8 {
                 // Slow path
                 for &p1 in candidates1 {
                    for &p2 in candidates2 {
                        for &p3 in candidates3 {
                            let cost = calculate_flow_cost(ctx, p1 as usize, p2 as usize, p3 as usize);
                            if cost < min_cost { min_cost = cost; }
                        }
                    }
                }
            } else {
                // Fast path
                for &p1 in candidates1 {
                    for &p2 in candidates2 {
                        for &p3 in candidates3 {
                            let cost = calculate_flow_cost(ctx, p1 as usize, p2 as usize, p3 as usize);
                            if cost < min_cost { min_cost = cost; }
                        }
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
