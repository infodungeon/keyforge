use super::flow::calculate_flow_cost;
use super::state::{PhysicsScratch, PosMap};
use crate::error::PhysicsError;
use crate::kernel::{
    types::{Score, ValidatedLayout},
    EngineContext, EvaluationContext,
};

/// Computes the total biomechanical score for a layout.
///
/// # Errors
/// Returns `PhysicsError::Config` if scratch initialization fails or
/// `PhysicsError::ScoreOverflow` if arithmetic fails.
pub fn score_layout(
    ctx: &EngineContext,
    layout: &ValidatedLayout<'_>,
    scratch: &mut PhysicsScratch,
) -> Result<i64, PhysicsError> {
    let layout_slice = layout.as_slice();
    let pm = PosMap::from_scratch(
        layout_slice,
        ctx.key_count,
        scratch.starts.as_mut(),
        scratch.counts.as_mut(),
        scratch.indices.as_mut_slice(),
        scratch.current_offsets.as_mut(),
        &mut scratch.used_keys,
    );

    let eval_ctx = EvaluationContext {
        engine: ctx,
        pos_map: &pm,
    };

    let m = score_monograms(&eval_ctx)?;
    let b = score_bigrams(&eval_ctx)?;
    let t = score_trigrams(&eval_ctx)?;

    let total = m
        .checked_add(b)
        .and_then(|sum| sum.checked_add(t))
        .ok_or_else(|| PhysicsError::ScoreOverflow {
            context: "Final kernel total score accumulation".to_string(),
        })?;

    // Clean up scratch for next use
    scratch.clear_used();
    Ok(total.raw())
}

/// Scores monograms (single key usage).
///
/// # Errors
/// Returns `PhysicsError` if:
/// - Score multiplication or accumulation overflows.
#[allow(clippy::cast_possible_wrap)]
pub fn score_monograms(ctx: &EvaluationContext<'_>) -> Result<Score, PhysicsError> {
    let mut total = Score::ZERO;
    for &code in ctx.pos_map.used_keys() {
        let freq = ctx.engine.corpus.char_freqs[code.raw() as usize];
        if freq == 0 {
            continue;
        }
        let candidates = ctx.pos_map.get(code);
        if candidates.is_empty() {
            continue;
        }

        let mut min_cost = Score::INFINITY_SENTINEL;
        for &p in candidates {
            let cost = ctx.engine.geometry.key_costs[p.as_usize()];
            if cost < min_cost {
                min_cost = cost;
            }
        }

        if min_cost != Score::INFINITY_SENTINEL {
            let contrib =
                min_cost
                    .checked_mul(freq as i64)
                    .ok_or_else(|| PhysicsError::ScoreOverflow {
                        context: format!("Monogram freq scale for code {code:?}"),
                    })?;
            total = total
                .checked_add(contrib)
                .ok_or_else(|| PhysicsError::ScoreOverflow {
                    context: format!("Monogram total accumulation at code {code:?}"),
                })?;
        }
    }
    Ok(total)
}

/// Scores bigrams (two-key sequences).
///
/// # Errors
/// Returns `PhysicsError` if:
/// - Score addition for modifiers overflows.
/// - Score multiplication or accumulation overflows.
#[allow(clippy::cast_possible_wrap)]
pub fn score_bigrams(ctx: &EvaluationContext<'_>) -> Result<Score, PhysicsError> {
    let mut total = Score::ZERO;
    for &code1 in ctx.pos_map.used_keys() {
        let c1_val = code1.raw() as usize;
        let candidates1 = ctx.pos_map.get(code1);
        let start = ctx.engine.corpus.bigram_starts[c1_val];
        let end = ctx.engine.corpus.bigram_starts[c1_val + 1];

        for k in start..end {
            let c2 = ctx.engine.corpus.bigram_others[k];
            let candidates2 = ctx.pos_map.get(c2);
            if candidates2.is_empty() {
                continue;
            }

            let mut min_cost = Score::INFINITY_SENTINEL;
            for &p1 in candidates1 {
                for &p2 in candidates2 {
                    let idx = p1.as_usize() * ctx.engine.key_count + p2.as_usize();
                    let mut cost = ctx.engine.geometry.cost_matrix[idx];

                    if let Some(&mod_val) =
                        ctx.engine.sequence_modifiers.get(&(code1.raw(), c2.raw()))
                    {
                        cost = cost.checked_add(mod_val).ok_or_else(|| {
                            PhysicsError::ScoreOverflow {
                                context: format!("Bigram modifier for ({}, {})", code1, c2.raw()),
                            }
                        })?;
                    }

                    if cost < min_cost {
                        min_cost = cost;
                    }
                }
            }

            if min_cost != Score::INFINITY_SENTINEL {
                let freq = i64::from(ctx.engine.corpus.bigram_freqs[k]);
                let contrib =
                    min_cost
                        .checked_mul(freq)
                        .ok_or_else(|| PhysicsError::ScoreOverflow {
                            context: format!("Bigram freq scale for ({:?}, {:?})", code1, c2),
                        })?;
                total = total
                    .checked_add(contrib)
                    .ok_or_else(|| PhysicsError::ScoreOverflow {
                        context: format!("Bigram total accumulation at ({:?}, {:?})", code1, c2),
                    })?;
            }
        }
    }
    Ok(total)
}

/// Scores trigrams (three-key sequences).
///
/// # Errors
/// Returns `PhysicsError` if:
/// - Score multiplication or accumulation overflows.
#[allow(clippy::cast_possible_wrap)]
pub fn score_trigrams(ctx: &EvaluationContext<'_>) -> Result<Score, PhysicsError> {
    let mut total = Score::ZERO;
    for &code1 in ctx.pos_map.used_keys() {
        let c1_val = code1.raw() as usize;
        let candidates1 = ctx.pos_map.get(code1);
        let start = ctx.engine.corpus.trigram_starts[c1_val];
        let end = ctx.engine.corpus.trigram_starts[c1_val + 1];

        for k in start..end {
            let c2 = ctx.engine.corpus.trigram_others1[k];
            let c3 = ctx.engine.corpus.trigram_others2[k];
            let candidates2 = ctx.pos_map.get(c2);
            let candidates3 = ctx.pos_map.get(c3);

            if candidates2.is_empty() || candidates3.is_empty() {
                continue;
            }

            let mut min_cost = Score::INFINITY_SENTINEL;
            for &p1 in candidates1 {
                for &p2 in candidates2 {
                    for &p3 in candidates3 {
                        let cost = calculate_flow_cost(
                            ctx.engine,
                            p1.as_usize(),
                            p2.as_usize(),
                            p3.as_usize(),
                        );
                        if cost < min_cost {
                            min_cost = cost;
                        }
                    }
                }
            }

            if min_cost != Score::INFINITY_SENTINEL {
                let freq = i64::from(ctx.engine.corpus.trigram_freqs[k]);
                let contrib =
                    min_cost
                        .checked_mul(freq)
                        .ok_or_else(|| PhysicsError::ScoreOverflow {
                            context: format!(
                                "Trigram freq scale for sequence starting with {code1:?}"
                            ),
                        })?;
                total = total
                    .checked_add(contrib)
                    .ok_or_else(|| PhysicsError::ScoreOverflow {
                        context: format!(
                            "Trigram total accumulation for sequence starting with {code1:?}"
                        ),
                    })?;
            }
        }
    }
    Ok(total)
}
