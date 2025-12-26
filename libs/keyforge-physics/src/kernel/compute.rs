use super::EngineContext;
use keyforge_model::{AnalysisReport, MetricViolation};
use keyforge_protocol::constants::SCORE_SCALE;
use tracing::instrument;

/// Scores a layout.
pub fn score_layout(ctx: &EngineContext, layout: &[u16], pos_map: &mut [u8]) -> i64 {
    let mut total_score = 0i64;

    pos_map.fill(255);

    let limit = layout.len().min(ctx.key_count);

    for (i, &code) in layout.iter().enumerate().take(limit) {
        if (code as usize) < pos_map.len() {
            pos_map[code as usize] = i as u8;
        }
    }

    // Bigrams
    for (c1, &p1) in pos_map.iter().enumerate().take(256) {
        if p1 == 255 {
            continue;
        }
        let start = ctx.bigram_starts[c1];
        let end = ctx.bigram_starts[c1 + 1];
        for k in start..end {
            let c2 = ctx.bigram_others[k] as usize;
            let p2 = pos_map[c2];
            if p2 != 255 {
                let idx = (p1 as usize) * ctx.key_count + (p2 as usize);
                if let Some(cost) = ctx.cost_matrix.get(idx) {
                    total_score = total_score
                        .checked_add(cost.saturating_mul(ctx.bigram_freqs[k] as i64))
                        .unwrap_or(i64::MAX);
                }
            }
        }
    }

    // Trigrams
    for (c1, &p1) in pos_map.iter().enumerate().take(256) {
        if p1 == 255 {
            continue;
        }
        let start = ctx.trigram_starts[c1];
        let end = ctx.trigram_starts[c1 + 1];
        for k in start..end {
            let c2 = ctx.trigram_others1[k] as usize;
            let c3 = ctx.trigram_others2[k] as usize;

            let p2 = pos_map[c2];
            let p3 = pos_map[c3];

            if p2 != 255 && p3 != 255 {
                let cost = calculate_flow_cost(ctx, p1 as usize, p2 as usize, p3 as usize);
                if cost != 0 {
                    total_score = total_score
                        .checked_add(cost.saturating_mul(ctx.trigram_freqs[k] as i64))
                        .unwrap_or(i64::MAX);
                }
            }
        }
    }
    total_score
}

pub fn calculate_swap_delta(
    ctx: &EngineContext,
    layout: &[u16],
    pos_map: &[u8],
    idx_a: usize,
    idx_b: usize,
) -> i64 {
    if idx_a >= layout.len() || idx_b >= layout.len() {
        return 0;
    }
    if idx_a >= ctx.key_count || idx_b >= ctx.key_count {
        return 0;
    }

    let code_a = layout[idx_a] as usize;
    let code_b = layout[idx_b] as usize;
    let mut delta = 0i64;

    let get_pos = |c: usize, p_map: &[u8]| -> usize {
        if c == code_a {
            idx_a
        } else if c == code_b {
            idx_b
        } else {
            p_map[c] as usize
        }
    };

    let get_swapped_pos = |c: usize, p_map: &[u8]| -> usize {
        if c == code_a {
            idx_b
        } else if c == code_b {
            idx_a
        } else {
            p_map[c] as usize
        }
    };

    if code_a < 256 {
        delta = delta.saturating_add(calc_key_delta(
            ctx,
            code_a,
            pos_map,
            &get_pos,
            &get_swapped_pos,
        ));
    }
    if code_b < 256 {
        delta = delta.saturating_add(calc_key_delta_skip(
            ctx,
            code_b,
            code_a,
            pos_map,
            &get_pos,
            &get_swapped_pos,
        ));
    }
    delta
}

#[inline(always)]
fn calc_key_delta<F1, F2>(
    ctx: &EngineContext,
    c_target: usize,
    pos_map: &[u8],
    get_old: &F1,
    get_new: &F2,
) -> i64
where
    F1: Fn(usize, &[u8]) -> usize,
    F2: Fn(usize, &[u8]) -> usize,
{
    let mut d = 0i64;
    // Bigrams Start
    let start = ctx.bigram_starts[c_target];
    let end = ctx.bigram_starts[c_target + 1];
    for k in start..end {
        let c_other = ctx.bigram_others[k] as usize;
        if pos_map[c_other] != 255 {
            let p1_old = get_old(c_target, pos_map);
            let p2_old = get_old(c_other, pos_map);
            if p1_old < ctx.key_count && p2_old < ctx.key_count {
                let cost_old = ctx.cost_matrix[p1_old * ctx.key_count + p2_old];
                let p1_new = get_new(c_target, pos_map);
                let p2_new = get_new(c_other, pos_map);
                if p1_new < ctx.key_count && p2_new < ctx.key_count {
                    let cost_new = ctx.cost_matrix[p1_new * ctx.key_count + p2_new];
                    d = d.saturating_add(
                        cost_new
                            .saturating_sub(cost_old)
                            .saturating_mul(ctx.bigram_freqs[k] as i64),
                    );
                }
            }
        }
    }
    // Bigrams End
    let start = ctx.bigram_rev_starts[c_target];
    let end = ctx.bigram_rev_starts[c_target + 1];
    for k in start..end {
        let c_other = ctx.bigram_rev_others[k] as usize;
        if c_other == c_target {
            continue;
        }
        if c_other < 256 && pos_map[c_other] != 255 {
            let p1_old = get_old(c_other, pos_map);
            let p2_old = get_old(c_target, pos_map);
            if p1_old < ctx.key_count && p2_old < ctx.key_count {
                let cost_old = ctx.cost_matrix[p1_old * ctx.key_count + p2_old];
                let p1_new = get_new(c_other, pos_map);
                let p2_new = get_new(c_target, pos_map);
                if p1_new < ctx.key_count && p2_new < ctx.key_count {
                    let cost_new = ctx.cost_matrix[p1_new * ctx.key_count + p2_new];
                    d = d.saturating_add(
                        cost_new
                            .saturating_sub(cost_old)
                            .saturating_mul(ctx.bigram_rev_freqs[k] as i64),
                    );
                }
            }
        }
    }
    // Trigrams Start
    let start = ctx.trigram_starts[c_target];
    let end = ctx.trigram_starts[c_target + 1];
    for k in start..end {
        let c2 = ctx.trigram_others1[k] as usize;
        let c3 = ctx.trigram_others2[k] as usize;
        if pos_map[c2] != 255 && pos_map[c3] != 255 {
            let p1_old = get_old(c_target, pos_map);
            let p2_old = get_old(c2, pos_map);
            let p3_old = get_old(c3, pos_map);
            let cost_old = calculate_flow_cost(ctx, p1_old, p2_old, p3_old);
            let p1_new = get_new(c_target, pos_map);
            let p2_new = get_new(c2, pos_map);
            let p3_new = get_new(c3, pos_map);
            let cost_new = calculate_flow_cost(ctx, p1_new, p2_new, p3_new);
            d = d.saturating_add(
                cost_new
                    .saturating_sub(cost_old)
                    .saturating_mul(ctx.trigram_freqs[k] as i64),
            );
        }
    }
    // Trigrams Mid
    let start = ctx.trigram_mid_starts[c_target];
    let end = ctx.trigram_mid_starts[c_target + 1];
    for k in start..end {
        let c1 = ctx.trigram_mid_others1[k] as usize;
        let c3 = ctx.trigram_mid_others2[k] as usize;
        if c1 == c_target {
            continue;
        }
        if c1 < 256 && pos_map[c1] != 255 && pos_map[c3] != 255 {
            let p1_old = get_old(c1, pos_map);
            let p2_old = get_old(c_target, pos_map);
            let p3_old = get_old(c3, pos_map);
            let cost_old = calculate_flow_cost(ctx, p1_old, p2_old, p3_old);
            let p1_new = get_new(c1, pos_map);
            let p2_new = get_new(c_target, pos_map);
            let p3_new = get_new(c3, pos_map);
            let cost_new = calculate_flow_cost(ctx, p1_new, p2_new, p3_new);
            d = d.saturating_add(
                cost_new
                    .saturating_sub(cost_old)
                    .saturating_mul(ctx.trigram_mid_freqs[k] as i64),
            );
        }
    }
    // Trigrams End
    let start = ctx.trigram_end_starts[c_target];
    let end = ctx.trigram_end_starts[c_target + 1];
    for k in start..end {
        let c1 = ctx.trigram_end_others1[k] as usize;
        let c2 = ctx.trigram_end_others2[k] as usize;
        if c1 == c_target || c2 == c_target {
            continue;
        }
        if c1 < 256 && pos_map[c1] != 255 && pos_map[c2] != 255 {
            let p1_old = get_old(c1, pos_map);
            let p2_old = get_old(c2, pos_map);
            let p3_old = get_old(c_target, pos_map);
            let cost_old = calculate_flow_cost(ctx, p1_old, p2_old, p3_old);
            let p1_new = get_new(c1, pos_map);
            let p2_new = get_new(c2, pos_map);
            let p3_new = get_new(c_target, pos_map);
            let cost_new = calculate_flow_cost(ctx, p1_new, p2_new, p3_new);
            d = d.saturating_add(
                cost_new
                    .saturating_sub(cost_old)
                    .saturating_mul(ctx.trigram_end_freqs[k] as i64),
            );
        }
    }
    d
}

#[inline(always)]
fn calc_key_delta_skip<F1, F2>(
    ctx: &EngineContext,
    c_target: usize,
    c_skip: usize,
    pos_map: &[u8],
    get_old: &F1,
    get_new: &F2,
) -> i64
where
    F1: Fn(usize, &[u8]) -> usize,
    F2: Fn(usize, &[u8]) -> usize,
{
    let mut d = 0i64;
    // Bigrams Start
    let start = ctx.bigram_starts[c_target];
    let end = ctx.bigram_starts[c_target + 1];
    for k in start..end {
        let c_other = ctx.bigram_others[k] as usize;
        if c_other == c_skip {
            continue;
        }
        if pos_map[c_other] != 255 {
            let p1_old = get_old(c_target, pos_map);
            let p2_old = get_old(c_other, pos_map);
            if p1_old < ctx.key_count && p2_old < ctx.key_count {
                let cost_old = ctx.cost_matrix[p1_old * ctx.key_count + p2_old];
                let p1_new = get_new(c_target, pos_map);
                let p2_new = get_new(c_other, pos_map);
                if p1_new < ctx.key_count && p2_new < ctx.key_count {
                    let cost_new = ctx.cost_matrix[p1_new * ctx.key_count + p2_new];
                    d = d.saturating_add(
                        cost_new
                            .saturating_sub(cost_old)
                            .saturating_mul(ctx.bigram_freqs[k] as i64),
                    );
                }
            }
        }
    }
    // Bigrams End
    let start = ctx.bigram_rev_starts[c_target];
    let end = ctx.bigram_rev_starts[c_target + 1];
    for k in start..end {
        let c_other = ctx.bigram_rev_others[k] as usize;
        if c_other == c_target || c_other == c_skip {
            continue;
        }
        if c_other < 256 && pos_map[c_other] != 255 {
            let p1_old = get_old(c_other, pos_map);
            let p2_old = get_old(c_target, pos_map);
            if p1_old < ctx.key_count && p2_old < ctx.key_count {
                let cost_old = ctx.cost_matrix[p1_old * ctx.key_count + p2_old];
                let p1_new = get_new(c_other, pos_map);
                let p2_new = get_new(c_target, pos_map);
                if p1_new < ctx.key_count && p2_new < ctx.key_count {
                    let cost_new = ctx.cost_matrix[p1_new * ctx.key_count + p2_new];
                    d = d.saturating_add(
                        cost_new
                            .saturating_sub(cost_old)
                            .saturating_mul(ctx.bigram_rev_freqs[k] as i64),
                    );
                }
            }
        }
    }
    // Trigrams Start
    let start = ctx.trigram_starts[c_target];
    let end = ctx.trigram_starts[c_target + 1];
    for k in start..end {
        let c2 = ctx.trigram_others1[k] as usize;
        let c3 = ctx.trigram_others2[k] as usize;
        if c2 == c_skip || c3 == c_skip {
            continue;
        }
        if pos_map[c2] != 255 && pos_map[c3] != 255 {
            let p1_old = get_old(c_target, pos_map);
            let p2_old = get_old(c2, pos_map);
            let p3_old = get_old(c3, pos_map);
            let cost_old = calculate_flow_cost(ctx, p1_old, p2_old, p3_old);
            let p1_new = get_new(c_target, pos_map);
            let p2_new = get_new(c2, pos_map);
            let p3_new = get_new(c3, pos_map);
            let cost_new = calculate_flow_cost(ctx, p1_new, p2_new, p3_new);
            d = d.saturating_add(
                cost_new
                    .saturating_sub(cost_old)
                    .saturating_mul(ctx.trigram_freqs[k] as i64),
            );
        }
    }
    // Trigrams Mid
    let start = ctx.trigram_mid_starts[c_target];
    let end = ctx.trigram_mid_starts[c_target + 1];
    for k in start..end {
        let c1 = ctx.trigram_mid_others1[k] as usize;
        let c3 = ctx.trigram_mid_others2[k] as usize;
        if c1 == c_target {
            continue;
        }
        if c1 == c_skip || c3 == c_skip {
            continue;
        }
        if c1 < 256 && pos_map[c1] != 255 && pos_map[c3] != 255 {
            let p1_old = get_old(c1, pos_map);
            let p2_old = get_old(c_target, pos_map);
            let p3_old = get_old(c3, pos_map);
            let cost_old = calculate_flow_cost(ctx, p1_old, p2_old, p3_old);
            let p1_new = get_new(c1, pos_map);
            let p2_new = get_new(c_target, pos_map);
            let p3_new = get_new(c3, pos_map);
            let cost_new = calculate_flow_cost(ctx, p1_new, p2_new, p3_new);
            d = d.saturating_add(
                cost_new
                    .saturating_sub(cost_old)
                    .saturating_mul(ctx.trigram_mid_freqs[k] as i64),
            );
        }
    }
    // Trigrams End
    let start = ctx.trigram_end_starts[c_target];
    let end = ctx.trigram_end_starts[c_target + 1];
    for k in start..end {
        let c1 = ctx.trigram_end_others1[k] as usize;
        let c2 = ctx.trigram_end_others2[k] as usize;
        if c1 == c_target || c2 == c_target {
            continue;
        }
        if c1 == c_skip || c2 == c_skip {
            continue;
        }
        if c1 < 256 && pos_map[c1] != 255 && pos_map[c2] != 255 {
            let p1_old = get_old(c1, pos_map);
            let p2_old = get_old(c2, pos_map);
            let p3_old = get_old(c_target, pos_map);
            let cost_old = calculate_flow_cost(ctx, p1_old, p2_old, p3_old);
            let p1_new = get_new(c1, pos_map);
            let p2_new = get_new(c2, pos_map);
            let p3_new = get_new(c_target, pos_map);
            let cost_new = calculate_flow_cost(ctx, p1_new, p2_new, p3_new);
            d = d.saturating_add(
                cost_new
                    .saturating_sub(cost_old)
                    .saturating_mul(ctx.trigram_end_freqs[k] as i64),
            );
        }
    }
    d
}

#[inline(always)]
fn calculate_flow_cost(ctx: &EngineContext, p1: usize, p2: usize, p3: usize) -> i64 {
    if p1 >= ctx.key_count || p2 >= ctx.key_count || p3 >= ctx.key_count {
        return 0;
    }
    let h1 = ctx.hands[p1];
    let h2 = ctx.hands[p2];
    let h3 = ctx.hands[p3];
    if h1 != h2 || h2 != h3 {
        return 0;
    }
    let f1 = ctx.fingers[p1] as i8;
    let f2 = ctx.fingers[p2] as i8;
    let f3 = ctx.fingers[p3] as i8;
    if f1 == f3 && f1 != f2 {
        return ctx.penalty_redirect;
    }
    let dir1 = f2 - f1;
    let dir2 = f3 - f2;
    if dir1 == 0 || dir2 == 0 {
        return 0;
    }
    if dir1.signum() != dir2.signum() {
        return ctx.penalty_redirect;
    }
    if dir1 < 0 {
        return -ctx.bonus_roll;
    }
    0
}

#[instrument(skip_all)]
pub fn analyze_layout(ctx: &EngineContext, layout: &[u16]) -> AnalysisReport {
    let mut report = AnalysisReport::default();
    let mut pos_map = vec![255u8; 65536];
    let mut heatmap = vec![0.0; ctx.key_count]; // Initialize Heatmap

    let limit = layout.len().min(ctx.key_count);
    for (i, &code) in layout.iter().enumerate().take(limit) {
        pos_map[code as usize] = i as u8;
    }

    let mut total_bigrams = 0.0;
    let mut left_hand_load = 0.0;
    let mut total_load = 0.0;

    let mut sfbs = Vec::new();
    let mut scissors = Vec::new();
    let mut redirs = Vec::new();

    // 1. Monogram Stats (Heatmap & Balance)
    for (c, &p) in pos_map.iter().enumerate().take(256) {
        if p == 255 {
            continue;
        }
        let freq = ctx.char_freqs[c] as f32;
        if freq > 0.0 {
            total_load += freq;
            let idx = p as usize;
            if idx < ctx.key_count {
                heatmap[idx] += freq; // Populate Heatmap
                if ctx.hands[idx] == 0 {
                    left_hand_load += freq;
                }
            }
        }
    }

    // 2. Bigram Stats
    for (c1, &p1) in pos_map.iter().enumerate().take(256) {
        if p1 == 255 {
            continue;
        }
        let start = ctx.bigram_starts[c1];
        let end = ctx.bigram_starts[c1 + 1];

        for k in start..end {
            let c2 = ctx.bigram_others[k] as usize;
            let p2 = pos_map[c2];
            if p2 != 255 {
                let freq = ctx.bigram_freqs[k] as f32;
                total_bigrams += freq;
                let idx1 = p1 as usize;
                let idx2 = p2 as usize;

                if idx1 < ctx.key_count && idx2 < ctx.key_count {
                    // Distance
                    let cost = ctx.cost_matrix[idx1 * ctx.key_count + idx2] as f32;
                    report.distance += cost * freq;

                    // SFB
                    if ctx.fingers[idx1] == ctx.fingers[idx2] && ctx.hands[idx1] == ctx.hands[idx2]
                    {
                        report.sfb_total += freq;
                        sfbs.push(MetricViolation {
                            keys: format!("{} {}", c1 as u8 as char, c2 as u8 as char),
                            score: 1.0,
                            freq,
                        });
                    }

                    // Scissor
                    let f1 = ctx.fingers[idx1] as i8;
                    let f2 = ctx.fingers[idx2] as i8;
                    let r1 = ctx.rows[idx1];
                    let r2 = ctx.rows[idx2];
                    if ctx.hands[idx1] == ctx.hands[idx2]
                        && (f1 - f2).abs() == 1
                        && (r1 - r2).abs() >= 2
                    {
                        report.scissors += freq;
                        scissors.push(MetricViolation {
                            keys: format!("{} {}", c1 as u8 as char, c2 as u8 as char),
                            score: 1.0,
                            freq,
                        });
                    }
                }
            }
        }
    }

    // 3. Trigram Stats (Redirects/Rolls)
    for (c1, &p1) in pos_map.iter().enumerate().take(256) {
        if p1 == 255 {
            continue;
        }
        let start = ctx.trigram_starts[c1];
        let end = ctx.trigram_starts[c1 + 1];
        for k in start..end {
            let c2 = ctx.trigram_others1[k] as usize;
            let c3 = ctx.trigram_others2[k] as usize;
            let p2 = pos_map[c2];
            let p3 = pos_map[c3];

            if p2 != 255 && p3 != 255 {
                let cost = calculate_flow_cost(ctx, p1 as usize, p2 as usize, p3 as usize);
                let freq = ctx.trigram_freqs[k] as f32;

                if cost == ctx.penalty_redirect {
                    report.redirects += freq;
                    redirs.push(MetricViolation {
                        keys: format!(
                            "{}{}{}",
                            c1 as u8 as char, c2 as u8 as char, c3 as u8 as char
                        ),
                        score: 1.0,
                        freq,
                    });
                } else if cost == -ctx.bonus_roll {
                    report.rolls += freq;
                }
            }
        }
    }

    // Finalize
    let sort_violations = |v: &mut Vec<MetricViolation>| {
        v.sort_by(|a, b| b.freq.partial_cmp(&a.freq).unwrap());
        v.truncate(10);
    };

    sort_violations(&mut sfbs);
    sort_violations(&mut scissors);
    sort_violations(&mut redirs);

    report.top_sfbs = sfbs;
    report.top_scissors = scissors;
    report.top_redirs = redirs;
    report.heatmap = heatmap; // Assign Heatmap

    let mut scratch = vec![255u8; 65536];
    report.score = score_layout(ctx, layout, &mut scratch) as f32 / SCORE_SCALE;
    report.distance /= SCORE_SCALE; // Normalize distance

    if total_bigrams > 0.0 {
        report.sfb_ratio = report.sfb_total / total_bigrams;
    }
    if total_load > 0.0 {
        let left_ratio = left_hand_load / total_load;
        report.hand_balance = (left_ratio - 0.5) * -2.0; // -1 (Left) to 1 (Right)
    }
    report
}
