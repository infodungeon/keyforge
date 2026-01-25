// libs/keyforge-physics/src/kernel/compute/delta/trigram.rs

use super::get_flow_delta;
use super::PosMap;
use crate::kernel::EngineContext;

#[allow(clippy::cast_possible_truncation)]
pub(crate) fn calculate_trigram_delta(
    ctx: &EngineContext,
    pos_map: &PosMap<'_>,
    code_a: crate::kernel::types::KeyCode,
    code_b: crate::kernel::types::KeyCode,
    idx_a: usize,
    idx_b: usize,
) -> i64 {
    enum TrigramPos {
        Start,
        Mid,
        End,
    }

    let mut total_delta = 0i64;
    if ctx.corpus.trigram_freqs.is_empty() {
        return 0;
    }

    let mut seen = std::collections::HashSet::new();
    let ca = code_a.0 as usize;
    let cb = code_b.0 as usize;

    let mut process_range = |starts: &[usize],
                             others1: &[crate::kernel::types::KeyCode],
                             others2: &[crate::kernel::types::KeyCode],
                             freqs: &[u32],
                             char_idx: usize,
                             pos: TrigramPos| {
        if let (Some(&s), Some(&e)) = (starts.get(char_idx), starts.get(char_idx + 1)) {
            for k in s..e {
                let (c1, c2, c3) = match pos {
                    TrigramPos::Start => (
                        crate::kernel::types::KeyCode(char_idx as u16),
                        others1[k],
                        others2[k],
                    ),
                    TrigramPos::Mid => (
                        others1[k],
                        crate::kernel::types::KeyCode(char_idx as u16),
                        others2[k],
                    ),
                    TrigramPos::End => (
                        others1[k],
                        others2[k],
                        crate::kernel::types::KeyCode(char_idx as u16),
                    ),
                };
                // Triple u16 -> u64 for fast hash
                let key = (u64::from(c1.0) << 32) | (u64::from(c2.0) << 16) | u64::from(c3.0);
                if seen.insert(key) {
                    total_delta += get_flow_delta(ctx, pos_map, c1, c2, c3, idx_a, idx_b)
                        * i64::from(freqs[k]);
                }
            }
        }
    };

    // 1. Starts
    process_range(
        &ctx.corpus.trigram_starts,
        &ctx.corpus.trigram_others1,
        &ctx.corpus.trigram_others2,
        &ctx.corpus.trigram_freqs,
        ca,
        TrigramPos::Start,
    );
    process_range(
        &ctx.corpus.trigram_starts,
        &ctx.corpus.trigram_others1,
        &ctx.corpus.trigram_others2,
        &ctx.corpus.trigram_freqs,
        cb,
        TrigramPos::Start,
    );

    // 2. Mids
    process_range(
        &ctx.corpus.trigram_mid_starts,
        &ctx.corpus.trigram_mid_others1,
        &ctx.corpus.trigram_mid_others2,
        &ctx.corpus.trigram_mid_freqs,
        ca,
        TrigramPos::Mid,
    );
    process_range(
        &ctx.corpus.trigram_mid_starts,
        &ctx.corpus.trigram_mid_others1,
        &ctx.corpus.trigram_mid_others2,
        &ctx.corpus.trigram_mid_freqs,
        cb,
        TrigramPos::Mid,
    );

    // 3. Ends
    process_range(
        &ctx.corpus.trigram_end_starts,
        &ctx.corpus.trigram_end_others1,
        &ctx.corpus.trigram_end_others2,
        &ctx.corpus.trigram_end_freqs,
        ca,
        TrigramPos::End,
    );
    process_range(
        &ctx.corpus.trigram_end_starts,
        &ctx.corpus.trigram_end_others1,
        &ctx.corpus.trigram_end_others2,
        &ctx.corpus.trigram_end_freqs,
        cb,
        TrigramPos::End,
    );

    total_delta
}
