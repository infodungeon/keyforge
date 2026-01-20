use super::flow::calculate_flow_cost;
use super::state::{PhysicsScratch, PosMap};
use crate::kernel::{
    types::{Score, ValidatedLayout},
    EngineContext,
};
use crate::error::PhysicsError;

#[allow(clippy::cast_possible_wrap)]
pub fn score_layout(
    ctx: &EngineContext,
    layout: &ValidatedLayout<'_>,
    scratch: &mut PhysicsScratch,
) -> Result<i64, PhysicsError> {
    let layout_slice = layout.as_slice();
    let pm = PosMap::from_scratch(
        layout_slice,
        ctx.key_count,
        scratch.starts.as_mut_slice(),
        scratch.counts.as_mut_slice(),
        scratch.indices.as_mut_slice(),
        scratch.current_offsets.as_mut_slice(),
        &mut scratch.used_keys,
    );

    let m = score_monograms(ctx, &pm)?;
    let b = score_bigrams(ctx, &pm)?;
    let t = score_trigrams(ctx, &pm)?;

    let total = m.checked_add(b)
        .and_then(|sum| sum.checked_add(t))
        .ok_or_else(|| PhysicsError::ScoreOverflow {
            context: "Final kernel total score accumulation".to_string()
        })?;

    // Clean up scratch for next use
    scratch.clear_used();
    Ok(total.0)
}

#[allow(clippy::cast_possible_wrap)]
pub fn score_monograms(ctx: &EngineContext, pm: &PosMap<'_>) -> Result<Score, PhysicsError> {
    let mut total = Score::ZERO;
    for &code in pm.used_keys {
        let c_val = code as usize;
        let freq = ctx.char_freqs[c_val];
        if freq == 0 {
            continue;
        }
        let candidates = pm.get(c_val);
        if candidates.is_empty() {
            continue;
        }

        let mut min_cost = Score(i64::MAX);
        for &p in candidates {
            let cost = ctx.key_costs[p as usize];
            if cost < min_cost {
                min_cost = cost;
            }
        }
        
        if min_cost.0 != i64::MAX {
            let contrib = min_cost.checked_mul(freq as i64).ok_or_else(|| PhysicsError::ScoreOverflow {
                context: format!("Monogram freq scale for code {}", code)
            })?;
            total = total.checked_add(contrib).ok_or_else(|| PhysicsError::ScoreOverflow {
                context: format!("Monogram total accumulation at code {}", code)
            })?;
        }
    }
    Ok(total)
}

#[allow(clippy::cast_possible_wrap)]
pub fn score_bigrams(ctx: &EngineContext, pm: &PosMap<'_>) -> Result<Score, PhysicsError> {
    let mut total = Score::ZERO;
    for &code1 in pm.used_keys {
        let c1_val = code1 as usize;
        let candidates1 = pm.get(c1_val);
        let start = ctx.bigram_starts[c1_val];
        let end = ctx.bigram_starts[c1_val + 1];

        for k in start..end {
            let c2 = ctx.bigram_others[k];
            let candidates2 = pm.get(c2.0 as usize);
            if candidates2.is_empty() {
                continue;
            }

            let mut min_cost = Score(i64::MAX);
            for &p1 in candidates1 {
                for &p2 in candidates2 {
                    let idx = (p1 as usize) * ctx.key_count + (p2 as usize);
                    let mut cost = ctx.cost_matrix[idx];

                    if let Some(&mod_val) = ctx.sequence_modifiers.get(&(code1, c2.0)) {
                        cost = cost.checked_add(mod_val).ok_or_else(|| PhysicsError::ScoreOverflow {
                            context: format!("Bigram modifier for ({}, {})", code1, c2.0)
                        })?;
                    }

                    if cost < min_cost {
                        min_cost = cost;
                    }
                }
            }
            
            if min_cost.0 != i64::MAX {
                let freq = i64::from(ctx.bigram_freqs[k]);
                let contrib = min_cost.checked_mul(freq).ok_or_else(|| PhysicsError::ScoreOverflow {
                    context: format!("Bigram freq scale for ({}, {})", code1, c2.0)
                })?;
                total = total.checked_add(contrib).ok_or_else(|| PhysicsError::ScoreOverflow {
                    context: format!("Bigram total accumulation at ({}, {})", code1, c2.0)
                })?;
            }
        }
    }
    Ok(total)
}

#[allow(clippy::cast_possible_wrap)]
pub fn score_trigrams(ctx: &EngineContext, pm: &PosMap<'_>) -> Result<Score, PhysicsError> {
    let mut total = Score::ZERO;
    for &code1 in pm.used_keys {
        let c1_val = code1 as usize;
        let candidates1 = pm.get(c1_val);
        let start = ctx.trigram_starts[c1_val];
        let end = ctx.trigram_starts[c1_val + 1];

        for k in start..end {
            let c2 = ctx.trigram_others1[k];
            let c3 = ctx.trigram_others2[k];
            let candidates2 = pm.get(c2.0 as usize);
            let candidates3 = pm.get(c3.0 as usize);
    
            if candidates2.is_empty() || candidates3.is_empty() {
                continue;
            }
    
            let mut min_cost = Score(i64::MAX);
            for &p1 in candidates1 {
                for &p2 in candidates2 {
                    for &p3 in candidates3 {
                        let cost = calculate_flow_cost(ctx, p1 as usize, p2 as usize, p3 as usize);
                        if cost < min_cost {
                            min_cost = cost;
                        }
                    }
                }
            }
    
            if min_cost.0 != i64::MAX {
                let freq = i64::from(ctx.trigram_freqs[k]);
                let contrib = min_cost.checked_mul(freq).ok_or_else(|| PhysicsError::ScoreOverflow {
                    context: format!("Trigram freq scale for sequence starting with {}", code1)
                })?;
                total = total.checked_add(contrib).ok_or_else(|| PhysicsError::ScoreOverflow {
                    context: format!("Trigram total accumulation for sequence starting with {}", code1)
                })?;
            }
        }
    }
    Ok(total)
}