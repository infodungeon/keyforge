// libs/keyforge-physics/src/kernel/compute/delta/mod.rs

mod bigram;
mod monogram;
mod trigram;

#[cfg(test)]
mod tests;

pub(crate) use super::state::PosMap;
use crate::error::PhysicsError;
use crate::kernel::{types::ValidatedLayout, EngineContext};

/// Calculates the score delta for swapping two keys in a layout.
pub(crate) fn calculate_swap_delta(
    ctx: &EngineContext,
    layout: &ValidatedLayout<'_>,
    pos_map: &PosMap<'_>,
    idx_a: usize,
    idx_b: usize,
) -> Result<i64, PhysicsError> {
    let layout_slice = layout.as_slice();
    if idx_a >= layout_slice.len() {
        return Err(PhysicsError::InvalidInput {
            message: format!("idx_a {idx_a} out of bounds ({})", layout_slice.len()),
        });
    }
    if idx_b >= layout_slice.len() {
        return Err(PhysicsError::InvalidInput {
            message: format!("idx_b {idx_b} out of bounds ({})", layout_slice.len()),
        });
    }

    let code_a = layout_slice[idx_a];
    let code_b = layout_slice[idx_b];
    if code_a == code_b {
        return Ok(0);
    }

    let mut delta = 0i64;

    // 1. Monograms
    delta += monogram::calculate_monogram_delta(ctx, pos_map, code_a, code_b, idx_a, idx_b);

    // 2. Bigrams
    delta += bigram::calculate_bigram_delta(ctx, pos_map, code_a, code_b, idx_a, idx_b);

    // 3. Trigrams
    delta += trigram::calculate_trigram_delta(ctx, layout, pos_map, idx_a.into(), idx_b.into());

    Ok(delta)
}

/// Shared helper for coordinate mapping during swap simulations.
#[inline]
pub(crate) fn get_p_effective(p: usize, idx_a: usize, idx_b: usize) -> usize {
    if p == idx_a {
        idx_b
    } else if p == idx_b {
        idx_a
    } else {
        p
    }
}
