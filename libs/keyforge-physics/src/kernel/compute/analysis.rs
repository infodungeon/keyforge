use super::flow::calculate_flow_cost;
use super::state::PosMap;
use crate::kernel::{
    types::{FingerIndex, Score, ValidatedLayout},
    EngineContext,
};
use keyforge_model::constants::{MAX_REPORTED_VIOLATIONS, SCORE_SCALE};
use keyforge_model::types::{FixedPointMath, IterationCount, ScalingFactor};
use keyforge_model::{AnalysisReport, KeyCode, MetricId, MetricViolation};

/// Safely converts a u16 character code to a displayable character.
/// Handles invalid Unicode surrogate pairs and control characters.
#[inline]
pub(crate) fn u16_to_char(code: u16) -> String {
    // 1. Common control/special characters
    match code {
        8 => return "⌫".to_string(),  // Backspace
        9 => return "⇥".to_string(),  // Tab
        10 => return "↵".to_string(), // Newline
        32 => return "␣".to_string(), // Space
        _ => {}
    }

    // 2. Try printable Unicode
    if let Some(c) = char::from_u32(u32::from(code)) {
        if !c.is_control() {
            return c.to_string();
        }
        return format!("[0x{code:02X}]");
    }

    // 3. Fallback for invalid Unicode
    format!("[0x{code:04X}]")
}

/// Mandated bit-perfect normalization for the `KeyForge` physical model.
/// This implementation ensures symmetric rounding and is overflow-proof at i128 scale.
#[must_use]
pub fn deterministic_normalize(
    accumulated: Score,
    scale: ScalingFactor,
    total_freq: IterationCount,
) -> Score {
    let t_raw = total_freq.raw();
    if t_raw == 0 {
        return Score::ZERO;
    }

    let a_128 = i128::from(accumulated.raw());
    let s_128 = i128::from(scale.raw());
    let t_128 = i128::from(u64::try_from(t_raw).unwrap_or(0));

    let product = a_128 * s_128;
    let half = t_128 / 2;

    // Split-division strategy to prevent (product + bias) overflow:
    // result = (product / total_freq) + (remainder + bias) / total_freq
    let result_raw = if product >= 0 {
        (product / t_128) + (product % t_128 + half) / t_128
    } else {
        (product / t_128) + (product % t_128 - half) / t_128
    };

    // Clamp to i64 range to ensure the final Score remains valid even under extreme scaling
    #[allow(clippy::cast_possible_truncation)]
    Score::from_raw(
        i64::try_from(result_raw.clamp(i128::from(i64::MIN), i128::from(i64::MAX))).unwrap_or(0),
    )
}

struct MetricsAccumulator<'a> {
    heatmap: &'a mut [u64],
    penalty_map: &'a mut [i64],
    total_load: u64,
    left_hand_load: u64,
    total_bigrams: u64,
    sfbs: Vec<MetricViolation>,
    scissors: Vec<MetricViolation>,
    redirs: Vec<MetricViolation>,
    dist_accum: Score,
    sfb_total_freq: u64,
    sfb_penalty_accum: Score,
    scissor_freq: u64,
    scissor_penalty_accum: Score,
    redirect_freq: u64,
    redir_penalty_accum: Score,
    roll_freq: u64,
    roll_penalty_accum: Score,
    mono_accum: Score,
    bigram_accum: Score,
    trigram_accum: Score,
}

impl<'a> MetricsAccumulator<'a> {
    fn new(heatmap: &'a mut [u64], penalty_map: &'a mut [i64]) -> Self {
        Self {
            heatmap,
            penalty_map,
            total_load: 0,
            left_hand_load: 0,
            total_bigrams: 0,
            sfbs: Vec::new(),
            scissors: Vec::new(),
            redirs: Vec::new(),
            dist_accum: Score::ZERO,
            sfb_total_freq: 0,
            sfb_penalty_accum: Score::ZERO,
            scissor_freq: 0,
            scissor_penalty_accum: Score::ZERO,
            redirect_freq: 0,
            redir_penalty_accum: Score::ZERO,
            roll_freq: 0,
            roll_penalty_accum: Score::ZERO,
            mono_accum: Score::ZERO,
            bigram_accum: Score::ZERO,
            trigram_accum: Score::ZERO,
        }
    }
}

/// Analyzes a layout and returns a detailed report.
///
/// # Errors
/// Returns `PhysicsError` if:
/// - The layout is invalid for the context.
/// - Calculation overflows occur during accumulation.
pub fn analyze_layout(
    ctx: &EngineContext,
    layout: &ValidatedLayout<'_>,
) -> Result<AnalysisReport, crate::error::PhysicsError> {
    let mut report = AnalysisReport::default();

    super::state::with_scratch(|scratch| {
        let key_count = ctx.key_count;
        let (starts, counts, indices, offsets, used, char_usage, _flat_map, heatmap, penalty_map) =
            scratch.get_mut_scratch();

        let pm = PosMap::from_scratch(
            layout.as_slice(),
            key_count,
            starts,
            counts,
            indices,
            offsets,
            used,
        );

        let mut acc = MetricsAccumulator::new(heatmap, penalty_map);

        // 1. Pass 1: Trigrams (Flow ONLY)
        process_trigrams(ctx, &pm, &mut acc)?;

        // 2. Pass 2: Bigrams (ALL TRANSITIONS, DISTANCE, USAGE)
        process_bigrams(ctx, &pm, &mut acc, char_usage)?;

        // 3. Pass 3: Monograms (Base Usage & Remaining Characters)
        process_monograms(ctx, &pm, &mut acc)?;

        // Pass 4: Finalize Load Metrics
        for i in 0..key_count {
            let val = acc.heatmap[i];
            acc.total_load += val;
            if ctx.geometry.hands[i].is_left() {
                acc.left_hand_load += val;
            }
        }

        finalize_report(ctx, acc, &mut report)?;

        // Clean up
        scratch.clear_used();
        Ok::<(), crate::error::PhysicsError>(())
    })??;

    Ok(report)
}

#[allow(clippy::too_many_lines)]
fn process_trigrams(
    ctx: &EngineContext,
    pm: &PosMap<'_>,
    acc: &mut MetricsAccumulator<'_>,
) -> Result<(), crate::error::PhysicsError> {
    let key_count = ctx.key_count;
    for &(c1, c2, c3, freq) in ctx.all_trigrams.iter() {
        let candidates1 = pm.get(KeyCode::new(c1));
        let candidates2 = pm.get(KeyCode::new(c2));
        let candidates3 = pm.get(KeyCode::new(c3));
        if candidates1.is_empty() || candidates2.is_empty() || candidates3.is_empty() {
            continue;
        }

        let freq_i = i64::from(freq);
        let mut min_cost_val = Score::from_scaled_i64(i64::MAX);
        let mut best_triplet = (0, 0, 0);

        for &p1 in candidates1 {
            for &p2 in candidates2 {
                for &p3 in candidates3 {
                    let mut cost =
                        calculate_flow_cost(ctx, p1.as_usize(), p2.as_usize(), p3.as_usize());
                    let idx12 = p1.as_usize() * key_count + p2.as_usize();
                    let idx23 = p2.as_usize() * key_count + p3.as_usize();
                    cost = cost
                        .checked_add(ctx.geometry.cost_matrix[idx12])
                        .and_then(|c| c.checked_add(ctx.geometry.cost_matrix[idx23]))
                        .ok_or_else(|| crate::error::PhysicsError::ScoreOverflow {
                            context: format!("Trigram cost accumulation for ({c1}, {c2}, {c3})"),
                        })?;

                    if cost < min_cost_val {
                        min_cost_val = cost;
                        best_triplet = (p1.as_usize(), p2.as_usize(), p3.as_usize());
                    }
                }
            }
        }

        if min_cost_val.raw() != i64::MAX {
            let (idx1, idx2, idx3) = best_triplet;
            let flow_cost = calculate_flow_cost(ctx, idx1, idx2, idx3);
            let contribution = flow_cost.checked_mul(freq_i).ok_or_else(|| {
                crate::error::PhysicsError::ScoreOverflow {
                    context: format!("Trigram contribution for ({c1}, {c2}, {c3})"),
                }
            })?;

            // Refactor scaling logic to use i128 intermediate arithmetic
            let contrib_raw = i128::from(contribution.raw());
            let part_raw = i64::try_from(contrib_raw / 3).map_err(|_| {
                crate::error::PhysicsError::ScoreOverflow {
                    context: "Trigram part overflow".to_string(),
                }
            })?;
            let rem_raw = i64::try_from(contrib_raw % 3).map_err(|_| {
                crate::error::PhysicsError::ScoreOverflow {
                    context: "Trigram remainder overflow".to_string(),
                }
            })?;

            let part = Score::from_raw(part_raw);
            let rem = Score::from_raw(rem_raw);

            acc.penalty_map[idx1] = Score::from_raw(acc.penalty_map[idx1])
                .checked_add(part)
                .ok_or_else(|| crate::error::PhysicsError::ScoreOverflow {
                    context: "Penalty map update idx1".to_string(),
                })?
                .raw();
            acc.penalty_map[idx2] = Score::from_raw(acc.penalty_map[idx2])
                .checked_add(part)
                .and_then(|p| p.checked_add(rem))
                .ok_or_else(|| crate::error::PhysicsError::ScoreOverflow {
                    context: "Penalty map update idx2".to_string(),
                })?
                .raw();
            acc.penalty_map[idx3] = Score::from_raw(acc.penalty_map[idx3])
                .checked_add(part)
                .ok_or_else(|| crate::error::PhysicsError::ScoreOverflow {
                    context: "Penalty map update idx3".to_string(),
                })?
                .raw();
            acc.trigram_accum = acc.trigram_accum.checked_add(contribution).ok_or_else(|| {
                crate::error::PhysicsError::ScoreOverflow {
                    context: "Trigram total accumulation".to_string(),
                }
            })?;

            if flow_cost == ctx.penalty_redirect {
                acc.redirect_freq += u64::from(freq);
                acc.redir_penalty_accum = acc
                    .redir_penalty_accum
                    .checked_add(contribution)
                    .ok_or_else(|| crate::error::PhysicsError::ScoreOverflow {
                        context: "Redirect penalty accumulation".to_string(),
                    })?;

                acc.redirs.push(MetricViolation {
                    keys: format!("{}{}{}", u16_to_char(c1), u16_to_char(c2), u16_to_char(c3)),
                    score: contribution,
                    freq: Score::from_raw(freq_i),
                });
            } else if flow_cost < Score::ZERO {
                acc.roll_freq += u64::from(freq);
                acc.roll_penalty_accum = acc
                    .roll_penalty_accum
                    .checked_add(contribution)
                    .ok_or_else(|| crate::error::PhysicsError::ScoreOverflow {
                        context: "Roll penalty accumulation".to_string(),
                    })?;
            }
        }
    }
    Ok(())
}

#[allow(clippy::too_many_lines)]
fn process_bigrams(
    ctx: &EngineContext,
    pm: &PosMap<'_>,
    acc: &mut MetricsAccumulator<'_>,
    char_usage: &mut [u64; 65536],
) -> Result<(), crate::error::PhysicsError> {
    let key_count = ctx.key_count;
    for &(c1, c2, freq) in ctx.all_bigrams.iter() {
        let candidates1 = pm.get(KeyCode::new(c1));
        let candidates2 = pm.get(KeyCode::new(c2));
        if candidates1.is_empty() || candidates2.is_empty() {
            continue;
        }

        let freq_i = i64::from(freq);
        acc.total_bigrams += u64::from(freq);

        let mut min_score = Score::from_scaled_i64(i64::MAX);
        let mut best_pair = (0, 0);

        if candidates1.len() == 1 && candidates2.len() == 1 {
            best_pair = (candidates1[0].as_usize(), candidates2[0].as_usize());
        } else {
            for &p1 in candidates1 {
                for &p2 in candidates2 {
                    let mut cost =
                        ctx.geometry.cost_matrix[p1.as_usize() * key_count + p2.as_usize()];
                    if let Some(&mod_val) = ctx.sequence_modifiers.get(&(c1, c2)) {
                        cost = cost.checked_add(mod_val).ok_or_else(|| {
                            crate::error::PhysicsError::ScoreOverflow {
                                context: format!("Bigram modifier for ({c1}, {c2})"),
                            }
                        })?;
                    }
                    if cost < min_score {
                        min_score = cost;
                        best_pair = (p1.as_usize(), p2.as_usize());
                    }
                }
            }
        }

        let (idx1, idx2) = best_pair;
        acc.heatmap[idx2] += u64::from(freq);
        char_usage[usize::from(c2)] += u64::from(freq);

        if idx1 != idx2 && ctx.geometry.hands[idx1] == ctx.geometry.hands[idx2] {
            let dist = ctx.geometry.dist_matrix[idx1 * key_count + idx2];
            let contrib = dist.checked_mul(freq_i).ok_or_else(|| {
                crate::error::PhysicsError::ScoreOverflow {
                    context: "Bigram distance contribution".to_string(),
                }
            })?;
            acc.dist_accum = acc.dist_accum.checked_add(contrib).ok_or_else(|| {
                crate::error::PhysicsError::ScoreOverflow {
                    context: "Bigram distance accumulation".to_string(),
                }
            })?;

            if ctx.geometry.fingers[idx1] == ctx.geometry.fingers[idx2] {
                acc.sfb_total_freq += u64::from(freq);
                let sfb_cost = ctx.geometry.cost_matrix[idx1 * key_count + idx2];
                let contribution = sfb_cost.checked_mul(freq_i).ok_or_else(|| {
                    crate::error::PhysicsError::ScoreOverflow {
                        context: "SFB contribution".to_string(),
                    }
                })?;
                acc.sfb_penalty_accum = acc
                    .sfb_penalty_accum
                    .checked_add(contribution)
                    .ok_or_else(|| crate::error::PhysicsError::ScoreOverflow {
                        context: "SFB penalty accumulation".to_string(),
                    })?;
                acc.sfbs.push(MetricViolation {
                    keys: format!("{}{}", u16_to_char(c1), u16_to_char(c2)),
                    score: contribution,
                    freq: Score::from_raw(freq_i),
                });
            }
        } else if idx1 != idx2 {
            let dist = ctx.geometry.key_home_distances[idx2];
            let contrib = dist.checked_mul(freq_i).ok_or_else(|| {
                crate::error::PhysicsError::ScoreOverflow {
                    context: "Bigram home distance contribution".to_string(),
                }
            })?;
            acc.dist_accum = acc.dist_accum.checked_add(contrib).ok_or_else(|| {
                crate::error::PhysicsError::ScoreOverflow {
                    context: "Bigram home distance accumulation".to_string(),
                }
            })?;
        }

        let r1 = ctx.geometry.rows[idx1];
        let r2 = ctx.geometry.rows[idx2];
        let f1 = ctx.geometry.fingers[idx1];
        let f2 = ctx.geometry.fingers[idx2];
        if ctx.geometry.hands[idx1] == ctx.geometry.hands[idx2]
            && f1.distance(f2) == 1
            && (i16::from(r1.raw()) - i16::from(r2.raw())).abs() >= 2
            && f1 != FingerIndex::THUMB
            && f2 != FingerIndex::THUMB
        {
            acc.scissor_freq += u64::from(freq);
            let scissor_cost = ctx.geometry.cost_matrix[idx1 * key_count + idx2];
            let contribution = scissor_cost.checked_mul(freq_i).ok_or_else(|| {
                crate::error::PhysicsError::ScoreOverflow {
                    context: "Scissor contribution".to_string(),
                }
            })?;
            acc.scissor_penalty_accum = acc
                .scissor_penalty_accum
                .checked_add(contribution)
                .ok_or_else(|| crate::error::PhysicsError::ScoreOverflow {
                    context: "Scissor penalty accumulation".to_string(),
                })?;
            acc.scissors.push(MetricViolation {
                keys: format!("{}{}", u16_to_char(c1), u16_to_char(c2)),
                score: contribution,
                freq: Score::from_raw(freq_i),
            });
        }

        let mut trans_cost = ctx.geometry.cost_matrix[idx1 * key_count + idx2];
        if let Some(&mod_val) = ctx.sequence_modifiers.get(&(c1, c2)) {
            trans_cost = trans_cost.checked_add(mod_val).ok_or_else(|| {
                crate::error::PhysicsError::ScoreOverflow {
                    context: "Bigram transition modifier".to_string(),
                }
            })?;
        }
        let trans_contrib = trans_cost.checked_mul(freq_i).ok_or_else(|| {
            crate::error::PhysicsError::ScoreOverflow {
                context: "Bigram transition contribution".to_string(),
            }
        })?;
        acc.bigram_accum = acc.bigram_accum.checked_add(trans_contrib).ok_or_else(|| {
            crate::error::PhysicsError::ScoreOverflow {
                context: "Bigram transition accumulation".to_string(),
            }
        })?;

        // Refactor scaling logic to use i128 intermediate arithmetic
        let tc_128 = i128::from(trans_contrib.raw());
        let part_raw =
            i64::try_from(tc_128 / 2).map_err(|_| crate::error::PhysicsError::ScoreOverflow {
                context: "Bigram part overflow".to_string(),
            })?;
        let rem_raw =
            i64::try_from(tc_128 % 2).map_err(|_| crate::error::PhysicsError::ScoreOverflow {
                context: "Bigram remainder overflow".to_string(),
            })?;

        let part = Score::from_raw(part_raw);
        let rem = Score::from_raw(rem_raw);

        acc.penalty_map[idx1] = Score::from_raw(acc.penalty_map[idx1])
            .checked_add(part)
            .ok_or_else(|| crate::error::PhysicsError::ScoreOverflow {
                context: "Bigram penalty map update idx1".to_string(),
            })?
            .raw();
        acc.penalty_map[idx2] = Score::from_raw(acc.penalty_map[idx2])
            .checked_add(part)
            .and_then(|p| p.checked_add(rem))
            .ok_or_else(|| crate::error::PhysicsError::ScoreOverflow {
                context: "Bigram penalty map update idx2".to_string(),
            })?
            .raw();
    }
    Ok(())
}

#[allow(clippy::cast_possible_truncation)]
fn process_monograms(
    ctx: &EngineContext,
    pm: &PosMap<'_>,
    acc: &mut MetricsAccumulator<'_>,
) -> Result<(), crate::error::PhysicsError> {
    for &code in pm.used_keys() {
        let freq = ctx.corpus.char_freqs[code.as_usize()];
        if freq == 0 {
            continue;
        }
        let freq_i = i64::try_from(freq).unwrap_or(i64::MAX);
        let candidates = pm.get(code);

        // Find minimum usage cost across all duplicate keys (Oracle Parity)
        let mut min_c = Score::from_scaled_i64(i64::MAX);
        let mut bp = 0;
        for &p in candidates {
            let c = ctx.geometry.key_costs[p.as_usize()];
            if c < min_c {
                min_c = c;
                bp = p.as_usize();
            }
        }

        if min_c.raw() != i64::MAX {
            let contrib = min_c.checked_mul(freq_i).ok_or_else(|| {
                crate::error::PhysicsError::ScoreOverflow {
                    context: format!("Monogram freq scale for code {code:?}"),
                }
            })?;
            acc.mono_accum = acc.mono_accum.checked_add(contrib).ok_or_else(|| {
                crate::error::PhysicsError::ScoreOverflow {
                    context: format!("Monogram accumulation at code {code:?}"),
                }
            })?;
            acc.heatmap[bp] += freq;
            acc.penalty_map[bp] = Score::from_raw(acc.penalty_map[bp])
                .checked_add(contrib)
                .ok_or_else(|| crate::error::PhysicsError::ScoreOverflow {
                    context: "Monogram penalty map update".to_string(),
                })?
                .raw();
        }
    }
    Ok(())
}

#[allow(clippy::too_many_lines)]
fn finalize_report(
    ctx: &EngineContext,
    mut acc: MetricsAccumulator<'_>,
    report: &mut AnalysisReport,
) -> Result<(), crate::error::PhysicsError> {
    let sort_violations = |v: &mut Vec<MetricViolation>| {
        v.sort_by(|a, b| b.freq.cmp(&a.freq));
        v.truncate(MAX_REPORTED_VIOLATIONS);
    };
    sort_violations(&mut acc.sfbs);
    sort_violations(&mut acc.scissors);
    sort_violations(&mut acc.redirs);

    let total_freq = ctx.corpus.char_freqs.iter().sum::<u64>();
    let total_freq_it = IterationCount::new(usize::try_from(total_freq).unwrap_or(0));
    let score_scale = SCORE_SCALE;

    if total_freq > 0 {
        report.travel_per_key =
            deterministic_normalize(acc.dist_accum, ScalingFactor::new(1), total_freq_it);

        let norm_100k =
            |val: Score| deterministic_normalize(val, ScalingFactor::new(100_000), total_freq_it);
        let norm_pct = |val: u64| {
            deterministic_normalize(
                Score::from_raw(i64::try_from(val).unwrap_or(i64::MAX)),
                ScalingFactor::new(100 * score_scale),
                total_freq_it,
            )
        };

        report.distance = norm_100k(acc.dist_accum);
        report.sfb_total = norm_pct(acc.sfb_total_freq);
        report.scissors = norm_pct(acc.scissor_freq);
        report.redirects = norm_pct(acc.redirect_freq);
        report.rolls = norm_pct(acc.roll_freq);
        report.sfb_penalty = norm_100k(acc.sfb_penalty_accum);
        report.scissor_penalty = norm_100k(acc.scissor_penalty_accum);
        report.redir_penalty = norm_100k(acc.redir_penalty_accum);
        report.roll_penalty = norm_100k(acc.roll_penalty_accum);

        report.heatmap = acc.heatmap[..ctx.key_count]
            .iter()
            .map(|&h| norm_pct(h))
            .collect();
        report.penalty_map = acc.penalty_map[..ctx.key_count]
            .iter()
            .map(|&p| norm_100k(Score::from_raw(p)))
            .collect();

        if acc.total_bigrams > 0 {
            report.sfb_ratio = deterministic_normalize(
                Score::from_raw(i64::try_from(acc.sfb_total_freq).unwrap_or(i64::MAX)),
                ScalingFactor::new(score_scale),
                IterationCount::new(usize::try_from(acc.total_bigrams).unwrap_or(0)),
            );
        }
        if acc.total_load > 0 {
            let left_share = deterministic_normalize(
                Score::from_raw(i64::try_from(acc.left_hand_load).unwrap_or(i64::MAX)),
                ScalingFactor::new(score_scale),
                IterationCount::new(usize::try_from(acc.total_load).unwrap_or(0)),
            );
            let balance = left_share
                .raw()
                .checked_sub(500_000)
                .and_then(|d| d.checked_mul(-2))
                .ok_or_else(|| crate::error::PhysicsError::ScoreOverflow {
                    context: "Hand balance calculation overflow".to_string(),
                })?;
            report.hand_balance = Score::from_raw(balance);
        }

        for v in &mut acc.sfbs {
            v.freq = norm_pct(v.freq.raw().unsigned_abs());
            v.score = norm_100k(v.score);
        }
        for v in &mut acc.scissors {
            v.freq = norm_pct(v.freq.raw().unsigned_abs());
            v.score = norm_100k(v.score);
        }
        for v in &mut acc.redirs {
            v.freq = norm_pct(v.freq.raw().unsigned_abs());
            v.score = norm_100k(v.score);
        }

        // Final Score: Sum components for bit-perfect parity with Oracle
        let raw_total = acc
            .mono_accum
            .checked_add(acc.bigram_accum)
            .and_then(|sum| sum.checked_add(acc.trigram_accum))
            .ok_or_else(|| crate::error::PhysicsError::ScoreOverflow {
                context: "Final report total score accumulation".to_string(),
            })?;
        report.raw_score = raw_total;
        report.score = norm_100k(raw_total);
    }

    report.top_sfbs = acc.sfbs;
    report.top_scissors = acc.scissors;
    report.top_redirs = acc.redirs;

    report
        .metrics
        .set(MetricId::TravelDistance, report.distance);
    report.metrics.set(MetricId::Sfb, report.sfb_total);
    report.metrics.set(MetricId::SfbPenalty, report.sfb_penalty);
    report.metrics.set(MetricId::Scissor, report.scissors);
    report
        .metrics
        .set(MetricId::ScissorPenalty, report.scissor_penalty);
    report.metrics.set(MetricId::Redirect, report.redirects);
    report
        .metrics
        .set(MetricId::RedirectPenalty, report.redir_penalty);
    report
        .metrics
        .set(MetricId::RollPenalty, report.roll_penalty);
    report
        .metrics
        .set(MetricId::HandBalance, report.hand_balance);

    Ok(())
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;
    use crate::kernel::compiler::Compiler;
    use keyforge_model::types::{ColIndex, FingerIndex, HandIndex, KeyCode, KeyIndex, RowIndex};
    use keyforge_model::{Corpus, CostModel, KeyNode, Keyboard, Rubric};
    use std::collections::HashMap;
    use std::sync::Arc;

    #[test]
    fn test_u16_to_char() -> anyhow::Result<()> {
        assert_eq!(u16_to_char(97), "a");
        assert_eq!(u16_to_char(8), "⌫");
        assert_eq!(u16_to_char(9), "⇥");
        assert_eq!(u16_to_char(10), "↵");
        assert_eq!(u16_to_char(32), "␣");
        assert_eq!(u16_to_char(0), "[0x00]");
        assert_eq!(u16_to_char(0xD800), "[0xD800]"); // Invalid surrogate
        Ok(())
    }

    #[test]
    fn test_analyze_layout_branches() -> anyhow::Result<()> {
        let mut keys = vec![
            KeyNode {
                index: KeyIndex(0),
                hand: HandIndex::LEFT,
                finger: FingerIndex::INDEX,
                row: RowIndex::new(0),
                col: ColIndex::new(0),
                is_home: true,
                ..Default::default()
            },
            KeyNode {
                index: KeyIndex(1),
                hand: HandIndex::LEFT,
                finger: FingerIndex::MIDDLE,
                row: RowIndex::new(0),
                col: ColIndex::new(1),
                is_home: true,
                ..Default::default()
            },
            KeyNode {
                index: KeyIndex(2),
                hand: HandIndex::LEFT,
                finger: FingerIndex::RING,
                row: RowIndex::new(0),
                col: ColIndex::new(2),
                is_home: true,
                ..Default::default()
            },
        ];
        // Add a duplicate key for space load sharing
        keys.push(KeyNode {
            index: KeyIndex(3),
            hand: HandIndex::LEFT,
            finger: FingerIndex::INDEX,
            row: RowIndex::new(1),
            col: ColIndex::new(0),
            is_home: false,
            ..Default::default()
        });

        let kb = Keyboard::new(keys, keyforge_model::types::RowIndex::new(0), "test".into())?;
        let mut corpus = Corpus::default();
        let mut freqs = corpus.char_freqs.to_vec();
        freqs[97] = 100; // 'a'
        freqs[98] = 200; // 'b'
        corpus.char_freqs = Arc::from(freqs);
        corpus.bigrams = Arc::from(vec![(97, 98, 50)]);
        corpus.trigrams = Arc::from(vec![(97, 98, 97, 10)]); // Redirect: a -> b -> a (Index -> Middle -> Index)

        let mut fingers = std::collections::HashMap::new();
        let sc = |v: i64| keyforge_model::types::Score::from_scaled_i64(v);

        let mut base_r0 = keyforge_model::cost_model::RowCosts::new();
        base_r0.insert(RowIndex::new(0), sc(1_000_000));
        let mut base_r1 = keyforge_model::cost_model::RowCosts::new();
        base_r1.insert(RowIndex::new(1), sc(2_000_000));

        let mut index_base = base_r0.clone();
        index_base.extend(base_r1);

        let index_zones = keyforge_model::cost_model::FingerReach {
            base: index_base,
            inner: HashMap::default(),
            outer: HashMap::default(),
        };

        fingers.insert(
            "index".to_string(),
            keyforge_model::cost_model::FingerDefinition::Standard(index_zones),
        );

        let other_zones = keyforge_model::cost_model::FingerReach {
            base: base_r0,
            inner: HashMap::default(),
            outer: HashMap::default(),
        };

        fingers.insert(
            "middle".to_string(),
            keyforge_model::cost_model::FingerDefinition::Standard(other_zones.clone()),
        );
        fingers.insert(
            "ring".to_string(),
            keyforge_model::cost_model::FingerDefinition::Standard(other_zones),
        );

        let mut cm = keyforge_model::cost_model::CostModel::default();
        cm.models.insert(
            "model_a_row_staggered".into(),
            keyforge_model::cost_model::ModelDefinition {
                description: "test".into(),
                static_costs: std::collections::HashMap::from([(
                    "universal_hand".to_string(),
                    keyforge_model::cost_model::HandDefinition { fingers },
                )]),
            },
        );

        let ctx = Compiler::compile(&kb, &corpus, &Rubric::default(), &cm)?;
        let layout_keys = vec![
            KeyCode::new(97),
            KeyCode::new(98),
            KeyCode::new(99),
            KeyCode::new(100),
        ];
        let validated = ValidatedLayout::new(&layout_keys, kb.count())?;

        let report = analyze_layout(&ctx, &validated)?;
        assert!(report.score.raw() > 0);
        assert!(report.redirects.raw() > 0);
        Ok(())
    }

    #[test]
    fn test_deterministic_normalize_symmetry() -> anyhow::Result<()> {
        let scale = ScalingFactor::new(10);
        let total_freq = IterationCount::new(100);

        // Input Score(25) * scale(10) / total_freq(100) = 2.5 -> 3
        assert_eq!(
            deterministic_normalize(Score::from_raw(25), scale, total_freq).raw(),
            3
        );

        // -2.5 -> -3
        assert_eq!(
            deterministic_normalize(Score::from_raw(-25), scale, total_freq).raw(),
            -3
        );

        // 2.4 -> 2
        assert_eq!(
            deterministic_normalize(Score::from_raw(24), scale, total_freq).raw(),
            2
        );

        // -2.4 -> -2
        assert_eq!(
            deterministic_normalize(Score::from_raw(-24), scale, total_freq).raw(),
            -2
        );
        Ok(())
    }
}
