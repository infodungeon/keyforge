#![allow(unsafe_code)]
use super::{EngineCapabilities, ScoringEngine};
use crate::kernel::compute::{flow::calculate_flow_cost, PhysicsScratch, PosMap};
use crate::kernel::{
    types::ValidatedLayout,
    EngineContext,
};
use crate::PhysicsError;
use keyforge_model::{AnalysisReport, Layout, Score, SwapSuggestion};

/// Configuration for the Intel-specific scoring engine.
#[derive(Debug, Clone, Copy)]
pub struct IntelEngineConfig {
    pub l1d_size_bytes: usize,
    pub l2_size_bytes: usize,
    pub l3_size_bytes: usize,
    /// If true, use prefetching intrinsics.
    pub use_prefetch: bool,
}

impl Default for IntelEngineConfig {
    fn default() -> Self {
        Self {
            l1d_size_bytes: 32 * 1024,
            l2_size_bytes: 256 * 1024,
            l3_size_bytes: 8 * 1024 * 1024,
            use_prefetch: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct IntelScoringEngine {
    pub(crate) ctx: EngineContext,
    config: IntelEngineConfig,
}

impl IntelScoringEngine {
    pub fn new(ctx: EngineContext, config: Option<IntelEngineConfig>) -> Self {
        Self { 
            ctx,
            config: config.unwrap_or_default(),
        }
    }

    fn score_internal(&self, layout: &Layout, force_scalar: bool) -> Result<Score, PhysicsError> {
        let mut scratch = Box::new(PhysicsScratch::new());
        let validated = ValidatedLayout::new(&layout.keys, self.ctx.key_count)?;

        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
             if is_x86_feature_detected!("avx2") && !force_scalar {
                 unsafe { Ok(Score(score_layout_avx2(&self.ctx, &validated, &mut scratch, &self.config)?)) }
             } else {
                 Ok(Score(score_layout_scalar(&self.ctx, &validated, &mut scratch)?))
             }
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        {
             Ok(Score(score_layout_scalar(&self.ctx, &validated, &mut scratch)?))
        }
    }
}

impl ScoringEngine for IntelScoringEngine {
    fn name(&self) -> &'static str {
        "Intel Comet Lake (AVX2 Optimized)"
    }

    fn capabilities(&self) -> EngineCapabilities {
        EngineCapabilities {
            is_exact: false,
            supports_avx2: true,
            supports_blocking: true, // Future work in Phase 3
        }
    }

    fn key_count(&self) -> usize {
        self.ctx.key_count
    }

    fn score(&self, layout: &Layout) -> Result<Score, PhysicsError> {
        self.score_internal(layout, false)
    }

    fn score_detailed(&self, layout: &Layout) -> Result<(i64, i64, i64), PhysicsError> {
        let mut scratch = Box::new(PhysicsScratch::new());
        let validated = ValidatedLayout::new(&layout.keys, self.ctx.key_count)?;
        let layout_slice = validated.as_slice();
        let pm = PosMap::from_scratch(
            layout_slice,
            self.ctx.key_count,
            scratch.starts.as_mut_slice(),
            scratch.counts.as_mut_slice(),
            scratch.indices.as_mut_slice(),
            scratch.current_offsets.as_mut_slice(),
            &mut scratch.used_keys,
        );

        let mono = score_monograms(&self.ctx, &pm)?.0;
        let bigram = score_bigrams(&self.ctx, &pm)?.0;
        let trigram = score_trigrams(&self.ctx, &pm)?.0;
        Ok((mono, bigram, trigram))
    }

    fn calculate_swap_delta(
        &self,
        layout: &Layout,
        _pos_map: &[u16],
        idx_a: usize,
        idx_b: usize,
    ) -> Result<i64, PhysicsError> {
        let mut scratch = Box::new(PhysicsScratch::new());
        let validated = ValidatedLayout::new(&layout.keys, self.ctx.key_count)?;

        let pm = PosMap::from_scratch(
            validated.as_slice(),
            self.ctx.key_count,
            scratch.starts.as_mut_slice(),
            scratch.counts.as_mut_slice(),
            scratch.indices.as_mut_slice(),
            scratch.current_offsets.as_mut_slice(),
            &mut scratch.used_keys,
        );

        Ok(crate::kernel::compute::calculate_swap_delta(&self.ctx, &validated, &pm, idx_a, idx_b))
    }

    fn analyze(&self, layout: &Layout) -> Result<AnalysisReport, PhysicsError> {
        let validated = ValidatedLayout::new(&layout.keys, self.ctx.key_count)?;
        Ok(crate::kernel::compute::analyze_layout(&self.ctx, &validated))
    }

    fn suggest_improvements(&self, layout: &Layout, include_thumbs: bool) -> Vec<SwapSuggestion> {
        crate::analysis::heuristics::suggest_swaps(&self.ctx, layout, include_thumbs)
    }

    fn context(&self) -> &EngineContext {
        &self.ctx
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Intent: Verify IntelEngineConfig default values are sensible.
    /// Expected: Default config has prefetch enabled and L1D cache size of 32KB.
    #[test]
    fn test_config_defaults() {
        let config = IntelEngineConfig::default();
        assert!(config.use_prefetch);
        assert_eq!(config.l1d_size_bytes, 32 * 1024);
    }
}

// -----------------------------------------------------------------------------
// Scalar Fallback (Copy of Generic)
// -----------------------------------------------------------------------------

#[allow(clippy::cast_possible_wrap)]
fn score_layout_scalar(
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
        .ok_or_else(|| PhysicsError::ScoreOverflow { context: "Intel scalar total score accumulation".into() })?;

    scratch.clear_used();
    Ok(total.0)
}

// -----------------------------------------------------------------------------
// AVX2 Optimized
// -----------------------------------------------------------------------------

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx2")]
#[allow(clippy::cast_possible_wrap)]
unsafe fn score_layout_avx2(
    ctx: &EngineContext,
    layout: &ValidatedLayout<'_>,
    scratch: &mut PhysicsScratch,
    _config: &IntelEngineConfig,
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
        .ok_or_else(|| PhysicsError::ScoreOverflow { context: "Intel AVX2 total score accumulation".into() })?;

    scratch.clear_used();
    Ok(total.0)
}

// Reuse helper functions. Ideally these would also be target_feature optimized,
// but they are called from within the target_feature function so they might get inlined and vectorized.
// To be sure, we should mark them inline always.

#[inline(always)]
fn score_monograms(ctx: &EngineContext, pm: &PosMap<'_>) -> Result<Score, PhysicsError> {
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
            let contrib = min_cost.checked_mul(freq as i64).ok_or_else(|| PhysicsError::ScoreOverflow { context: format!("Intel Monogram freq scale for code {}", code) })?;
            total = total.checked_add(contrib).ok_or_else(|| PhysicsError::ScoreOverflow { context: format!("Intel Monogram total accumulation at code {}", code) })?;
        }
    }
    Ok(total)
}

#[inline(always)]
fn score_bigrams(ctx: &EngineContext, pm: &PosMap<'_>) -> Result<Score, PhysicsError> {
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
            // This inner loop is the hot path for optimization
            for &p1 in candidates1 {
                for &p2 in candidates2 {
                    let idx = (p1 as usize) * ctx.key_count + (p2 as usize);
                    
                    let mut cost = ctx.cost_matrix[idx];

                    if let Some(&mod_val) = ctx.sequence_modifiers.get(&(code1, c2.0)) {
                        cost = cost.checked_add(mod_val).ok_or_else(|| PhysicsError::ScoreOverflow { context: format!("Intel Bigram modifier for ({}, {})", code1, c2.0) })?;
                    }

                    if cost < min_cost {
                        min_cost = cost;
                    }
                }
            }
            
            if min_cost.0 != i64::MAX {
                let freq = i64::from(ctx.bigram_freqs[k]);
                let contrib = min_cost.checked_mul(freq).ok_or_else(|| PhysicsError::ScoreOverflow { context: format!("Intel Bigram freq scale for ({}, {})", code1, c2.0) })?;
                total = total.checked_add(contrib).ok_or_else(|| PhysicsError::ScoreOverflow { context: format!("Intel Bigram total accumulation at ({}, {})", code1, c2.0) })?;
            }
        }
    }
    Ok(total)
}

#[inline(always)]
fn score_trigrams(ctx: &EngineContext, pm: &PosMap<'_>) -> Result<Score, PhysicsError> {
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

            if min_cost.0 != i64::MAX && min_cost.0 != 0 {
                let freq = i64::from(ctx.trigram_freqs[k]);
                let contrib = min_cost.checked_mul(freq).ok_or_else(|| PhysicsError::ScoreOverflow { context: format!("Intel Trigram freq scale for sequence starting with {}", code1) })?;
                total = total.checked_add(contrib).ok_or_else(|| PhysicsError::ScoreOverflow { context: format!("Intel Trigram total accumulation for sequence starting with {}", code1) })?;
            }
        }
    }
    Ok(total)
}