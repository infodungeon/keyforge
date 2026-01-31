use crate::kernel::types::{KeyCode, KeyIndex, Score, ValidatedLayout};
use crate::kernel::EngineContext;
use crate::PhysicsError;
use keyforge_model::{Keyboard, Layout, Rubric};

/// A bit-perfect reference implementation of the scoring logic.
///
/// This implementation is intended to be used as a "Truth Oracle" to verify the
/// correctness of optimized SIMD/WASM engines. It is deliberately simple
/// and unoptimized, prioritizing clarity and direct adherence to the physical model.
#[derive(Debug, Clone)]
pub struct DeterministicScorer {
    ctx: EngineContext,
}

impl DeterministicScorer {
    /// Creates a new `DeterministicScorer` from a compiled engine context.
    #[must_use]
    pub fn new(ctx: EngineContext) -> Self {
        Self { ctx }
    }

    /// Scores a layout using the reference algorithm.
    ///
    /// # Errors
    ///
    /// Returns an error if the layout is invalid for the keyboard.
    pub fn score(
        &self,
        keyboard: &Keyboard,
        corpus: &keyforge_model::Corpus,
        layout: &[KeyCode],
    ) -> Result<i64, PhysicsError> {
        let (mono, bigram, trigram) = self.score_detailed(keyboard, corpus, layout)?;
        Ok(mono + bigram + trigram)
    }

    /// Scores a layout and returns detailed components (monograms, bigrams, trigrams).
    ///
    /// # Errors
    ///
    /// Returns an error if the layout is invalid for the keyboard.
    pub fn score_detailed(
        &self,
        keyboard: &Keyboard,
        corpus: &keyforge_model::Corpus,
        layout: &[KeyCode],
    ) -> Result<(i64, i64, i64), PhysicsError> {
        let key_count = keyboard.keys().len();
        let _validated = ValidatedLayout::new(layout, key_count)?;

        // Component 1: Monograms
        let mut monogram_score = 0i64;
        for (code_val, &freq) in corpus.char_freqs.iter().enumerate() {
            if freq == 0 {
                continue;
            }
            let code = KeyCode::new(code_val.try_into().unwrap_or_default());
            let indices = find_indices(layout, code);
            if indices.is_empty() {
                continue;
            }

            // Find minimum usage cost across all duplicate keys
            let mut min_cost = i64::MAX;
            for &idx in &indices {
                let cost = self.ctx.geometry.key_costs[idx.raw() as usize].raw();
                if cost < min_cost {
                    min_cost = cost;
                }
            }

            if min_cost != i64::MAX {
                monogram_score += min_cost * (i64::try_from(freq).unwrap_or(0));
            }
        }

        // Component 2: Bigrams
        let mut bigram_score = 0i64;
        for (c1, c2, freq) in &*corpus.bigrams {
            let freq = i64::from(*freq);
            let indices1 = find_indices(layout, KeyCode::new(*c1));
            let indices2 = find_indices(layout, KeyCode::new(*c2));

            if !indices1.is_empty() && !indices2.is_empty() {
                let mut min_cost = i64::MAX;
                for &idx1 in &indices1 {
                    for &idx2 in &indices2 {
                        let base_cost = self.ctx.geometry.cost_matrix
                            [(idx1.raw() as usize) * key_count + (idx2.raw() as usize)]
                            .raw();

                        // Apply sequence modifier if any
                        let mut final_cost = base_cost;
                        if let Some(&mod_val) = self.ctx.sequence_modifiers.get(&(*c1, *c2)) {
                            final_cost = final_cost.saturating_add(mod_val.raw());
                        }

                        if final_cost < min_cost {
                            min_cost = final_cost;
                        }
                    }
                }

                if min_cost != i64::MAX {
                    bigram_score += min_cost * freq;
                }
            }
        }

        // Component 3: Trigrams
        let mut trigram_score = 0i64;
        for (c1, c2, c3, freq) in &*corpus.trigrams {
            let freq = i64::from(*freq);
            let indices1 = find_indices(layout, KeyCode::new(*c1));
            let indices2 = find_indices(layout, KeyCode::new(*c2));
            let indices3 = find_indices(layout, KeyCode::new(*c3));

            if !indices1.is_empty() && !indices2.is_empty() && !indices3.is_empty() {
                let mut min_cost = i64::MAX;
                for &idx1 in &indices1 {
                    for &idx2 in &indices2 {
                        for &idx3 in &indices3 {
                            let cost = calculate_trigram_cost(&self.ctx, idx1, idx2, idx3);
                            if cost < min_cost {
                                min_cost = cost;
                            }
                        }
                    }
                }

                if min_cost != i64::MAX {
                    trigram_score += min_cost * freq;
                }
            }
        }

        Ok((monogram_score, bigram_score, trigram_score))
    }
}

fn find_indices(layout: &[KeyCode], code: KeyCode) -> Vec<KeyIndex> {
    layout
        .iter()
        .enumerate()
        .filter(|(_, &c)| c == code)
        .map(|(i, _)| KeyIndex::new(u16::try_from(i).unwrap_or(0)))
        .collect()
}

fn calculate_trigram_cost(
    ctx: &EngineContext,
    idx1: KeyIndex,
    idx2: KeyIndex,
    idx3: KeyIndex,
) -> i64 {
    let h1 = ctx.geometry.hands[idx1.raw() as usize];
    let h2 = ctx.geometry.hands[idx2.raw() as usize];
    let h3 = ctx.geometry.hands[idx3.raw() as usize];

    let f1 = ctx.geometry.fingers[idx1.raw() as usize];
    let f2 = ctx.geometry.fingers[idx2.raw() as usize];
    let f3 = ctx.geometry.fingers[idx3.raw() as usize];

    // Simple reference trigram logic (Redirects)
    if h1 == h3 && h1 != h2 {
        // Change hand and back (Redirect)
        return ctx.penalty_redirect.raw();
    }

    // Rolls
    if h1 == h2 && h2 == h3 {
        let d1 = i16::from(f2.raw()) - i16::from(f1.raw());
        let d2 = i16::from(f3.raw()) - i16::from(f2.raw());

        if d1.signum() == d2.signum() && d1 != 0 && d2 != 0 {
            // All same direction
            if d1 > 0 {
                // Inward
                return -ctx.bonus_roll.raw();
            }
            // Outward
            return -ctx.bonus_roll_out.raw();
        }
    }

    0
}

/// Verification result for a single layout.
#[derive(Debug, Clone)]
pub struct VerificationResult {
    /// True if the scores match perfectly.
    pub match_perfect: bool,
    /// The reference score from the Oracle.
    pub reference_score: Score,
    /// The score being verified.
    pub actual_score: Score,
    /// Monogram component of reference score.
    pub ref_mono: Score,
    /// Bigram component of reference score.
    pub ref_bigram: Score,
    /// Trigram component of reference score.
    pub ref_trigram: Score,
}

/// Verifies an engine implementation against the reference scorer.
///
/// # Errors
///
/// Returns an error if scoring fails.
pub fn verify_engine(
    engine: &dyn crate::engines::ScoringEngine,
    keyboard: &Keyboard,
    corpus: &keyforge_model::Corpus,
    layout: &Layout,
    _rubric: &Rubric,
    _cost_model: &keyforge_model::CostModel,
) -> Result<VerificationResult, PhysicsError> {
    let reference = DeterministicScorer::new(engine.context().clone());
    let (ref_mono, ref_bigram, ref_trigram) =
        reference.score_detailed(keyboard, corpus, layout.keys())?;
    let ref_total = ref_mono + ref_bigram + ref_trigram;

    let actual = engine.score(layout)?;

    Ok(VerificationResult {
        match_perfect: actual.raw() == ref_total,
        reference_score: Score::from_scaled_i64(ref_total),
        actual_score: actual,
        ref_mono: Score::from_scaled_i64(ref_mono),
        ref_bigram: Score::from_scaled_i64(ref_bigram),
        ref_trigram: Score::from_scaled_i64(ref_trigram),
    })
}
