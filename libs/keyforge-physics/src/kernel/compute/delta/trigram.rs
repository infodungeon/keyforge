use crate::kernel::compute::PosMap;
use crate::kernel::mechanics::calculate_flow_cost;
use crate::kernel::types::{KeyCode, KeyIndex, ValidatedLayout};
use crate::kernel::EngineContext;

/// Calculates the change in score when two keys are swapped, considering trigrams.
#[allow(clippy::too_many_lines)]
#[must_use]
pub fn calculate_trigram_delta(
    ctx: &EngineContext,
    layout: &ValidatedLayout<'_>,
    pm: &PosMap<'_>,
    idx_a: usize,
    idx_b: usize,
) -> i64 {
    let mut delta = 0i64;

    let keys = layout.as_slice();
    let code_a = keys[idx_a];
    let code_b = keys[idx_b];

    if code_a == KeyCode::EMPTY && code_b == KeyCode::EMPTY {
        return 0;
    }

    let pos_a = KeyIndex::new(u16::try_from(idx_a).unwrap_or(0));
    let pos_b = KeyIndex::new(u16::try_from(idx_b).unwrap_or(0));

    let ca = code_a.as_usize();
    let cb = code_b.as_usize();

    // 2. Trigrams involving A
    let start_a = ctx.corpus.trigram_starts[ca];
    let end_a = ctx.corpus.trigram_starts[ca + 1];

    for i in start_a..end_a {
        let c2 = ctx.corpus.trigram_others1[i];
        let c3 = ctx.corpus.trigram_others2[i];
        let freq = ctx.corpus.trigram_freqs[i];

        let mut min_old = i64::MAX;
        let mut min_new = i64::MAX;

        for &p1 in pm.get(code_a) {
            let p1_new = swap_pos(p1, pos_a, pos_b);
            for &p2 in pm.get(c2) {
                let p2_new = swap_pos(p2, pos_a, pos_b);
                for &p3 in pm.get(c3) {
                    let cost = calculate_flow_cost_at(ctx, p1, p2, p3);
                    if cost < min_old {
                        min_old = cost;
                    }

                    // New position for A
                    let p3_new = swap_pos(p3, pos_a, pos_b);

                    let cost_new = calculate_flow_cost_at(ctx, p1_new, p2_new, p3_new);
                    if cost_new < min_new {
                        min_new = cost_new;
                    }
                }
            }
        }

        if min_old != i64::MAX && min_new != i64::MAX {
            delta += (min_new - min_old) * i64::from(freq);
        }
    }

    // 3. Trigrams involving B
    let start_b = ctx.corpus.trigram_starts[cb];
    let end_b = ctx.corpus.trigram_starts[cb + 1];

    for i in start_b..end_b {
        let c2 = ctx.corpus.trigram_others1[i];
        let c3 = ctx.corpus.trigram_others2[i];
        let freq = ctx.corpus.trigram_freqs[i];

        // Skip if already processed in A (would be redundant)
        if c2 == code_a || c3 == code_a {
            continue;
        }

        let mut min_old = i64::MAX;
        let mut min_new = i64::MAX;

        for &p1 in pm.get(code_b) {
            let p1_new = swap_pos(p1, pos_a, pos_b);
            for &p2 in pm.get(c2) {
                let p2_new = swap_pos(p2, pos_a, pos_b);
                for &p3 in pm.get(c3) {
                    let cost = calculate_flow_cost_at(ctx, p1, p2, p3);
                    if cost < min_old {
                        min_old = cost;
                    }

                    // New position for B
                    let p3_new = swap_pos(p3, pos_a, pos_b);

                    let cost_new = calculate_flow_cost_at(ctx, p1_new, p2_new, p3_new);
                    if cost_new < min_new {
                        min_new = cost_new;
                    }
                }
            }
        }

        if min_old != i64::MAX && min_new != i64::MAX {
            delta += (min_new - min_old) * i64::from(freq);
        }
    }

    delta
}

#[inline]
fn swap_pos(p: KeyIndex, pos_a: KeyIndex, pos_b: KeyIndex) -> KeyIndex {
    if p == pos_a {
        pos_b
    } else if p == pos_b {
        pos_a
    } else {
        p
    }
}

#[inline]
fn calculate_flow_cost_at(ctx: &EngineContext, p1: KeyIndex, p2: KeyIndex, p3: KeyIndex) -> i64 {
    let idx1 = p1.as_usize();
    let idx2 = p2.as_usize();
    let idx3 = p3.as_usize();
    calculate_flow_cost(
        ctx.geometry.hands[idx1],
        ctx.geometry.hands[idx2],
        ctx.geometry.hands[idx3],
        ctx.geometry.fingers[idx1],
        ctx.geometry.fingers[idx2],
        ctx.geometry.fingers[idx3],
        ctx.penalty_redirect,
        ctx.bonus_roll,
        ctx.bonus_roll_out,
    )
    .raw()
}
