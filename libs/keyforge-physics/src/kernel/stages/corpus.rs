use super::CompilationStage;
use crate::error::PhysicsError;
use crate::kernel::types::KeyCode;
use keyforge_model::constants::MAX_KEYCODE_SPACE;
use keyforge_model::{Corpus, Rubric};
use std::sync::Arc;

/// Intermediate state containing flattened and pruned corpus data.
#[derive(Debug)]
pub(crate) struct CorpusOutput {
    pub char_freqs: Arc<[u64]>,
    pub bigram_starts: Vec<usize>,
    pub bigram_others: Vec<KeyCode>,
    pub bigram_freqs: Vec<u32>,
    pub bigram_rev_starts: Vec<usize>,
    pub bigram_rev_others: Vec<KeyCode>,
    pub bigram_rev_freqs: Vec<u32>,
    pub trigram_starts: Vec<usize>,
    pub trigram_others1: Vec<KeyCode>,
    pub trigram_others2: Vec<KeyCode>,
    pub trigram_freqs: Vec<u32>,
}

/// Stage 3: Corpus Flattening & Pruning.
#[derive(Debug)]
pub(crate) struct CorpusStage<'a> {
    pub corpus: &'a Corpus,
    pub rubric: &'a Rubric,
}

impl CompilationStage for CorpusStage<'_> {
    type Input = ();
    type Output = CorpusOutput;

    fn execute(&self, (): Self::Input) -> Result<Self::Output, PhysicsError> {
        let (bigram_starts, bigram_others, bigram_freqs) = flatten_bigrams(&self.corpus.bigrams);
        let (bigram_rev_starts, bigram_rev_others, bigram_rev_freqs) =
            flatten_bigrams_rev(&self.corpus.bigrams);

        let pruned_trigrams = prune_trigrams(
            &self.corpus.trigrams,
            self.rubric.trigram_coverage(),
            self.rubric.trigram_limit(),
        );

        let (trigram_starts, trigram_others1, trigram_others2, trigram_freqs) =
            flatten_trigrams_start(&pruned_trigrams);

        Ok(CorpusOutput {
            char_freqs: self.corpus.char_freqs.clone(),
            bigram_starts,
            bigram_others,
            bigram_freqs,
            bigram_rev_starts,
            bigram_rev_others,
            bigram_rev_freqs,
            trigram_starts,
            trigram_others1,
            trigram_others2,
            trigram_freqs,
        })
    }
}

fn flatten_bigrams(source: &[(u16, u16, u32)]) -> (Vec<usize>, Vec<KeyCode>, Vec<u32>) {
    let mut sorted = source.to_vec();
    sorted.sort_unstable_by_key(|&(c1, _, _)| c1);
    let mut starts = vec![0; MAX_KEYCODE_SPACE + 1];
    let mut others = Vec::with_capacity(source.len());
    let mut freqs = Vec::with_capacity(source.len());

    let mut current_char = 0usize;
    let mut current_offset = 0usize;

    for &(c1, c2, freq) in &sorted {
        let c1 = c1 as usize;
        while current_char <= c1 {
            starts[current_char] = current_offset;
            current_char += 1;
        }
        others.push(KeyCode::new(c2));
        freqs.push(freq);
        current_offset += 1;
    }

    while current_char <= MAX_KEYCODE_SPACE {
        starts[current_char] = current_offset;
        current_char += 1;
    }

    (starts, others, freqs)
}

fn flatten_bigrams_rev(source: &[(u16, u16, u32)]) -> (Vec<usize>, Vec<KeyCode>, Vec<u32>) {
    let mut sorted = source.to_vec();
    sorted.sort_unstable_by_key(|&(_, c2, _)| c2);

    let mut starts = vec![0; MAX_KEYCODE_SPACE + 1];
    let mut others = Vec::with_capacity(source.len());
    let mut freqs = Vec::with_capacity(source.len());

    let mut current_char = 0usize;
    let mut current_offset = 0usize;

    for &(c1, c2, freq) in &sorted {
        let c2 = c2 as usize;
        while current_char <= c2 {
            starts[current_char] = current_offset;
            current_char += 1;
        }
        others.push(KeyCode::new(c1));
        freqs.push(freq);
        current_offset += 1;
    }

    while current_char <= MAX_KEYCODE_SPACE {
        starts[current_char] = current_offset;
        current_char += 1;
    }

    (starts, others, freqs)
}

#[allow(clippy::cast_sign_loss, clippy::cast_precision_loss)]
fn prune_trigrams(
    source: &[(u16, u16, u16, u32)],
    coverage: Score,
    limit: usize,
) -> Vec<(u16, u16, u16, u32)> {
    if source.is_empty() {
        return vec![];
    }

    let mut source = source.to_vec();

    source.sort_unstable_by(|a, b| {
        b.3.cmp(&a.3)
            .then_with(|| a.0.cmp(&b.0))
            .then_with(|| a.1.cmp(&b.1))
            .then_with(|| a.2.cmp(&b.2))
    });
    let total_freq: u64 = source.iter().map(|x| u64::from(x.3)).sum();
    // Deterministic coverage calculation using Score (basis: SCORE_SCALE)
    let coverage_scaled = coverage.raw() as u64; 
    #[allow(clippy::cast_possible_truncation)]
    let target = (u128::from(total_freq) * u128::from(coverage_scaled) / 1_000_000) as u64;
    let mut acc = 0;
    let mut cutoff = source.len();
    for (i, item) in source.iter().enumerate() {
        acc += u64::from(item.3);
        if acc >= target {
            cutoff = i + 1;
            break;
        }
    }
    if cutoff > limit {
        cutoff = limit;
    }
    source.truncate(cutoff);
    source
}

fn flatten_trigrams_start(
    source: &[(u16, u16, u16, u32)],
) -> (Vec<usize>, Vec<KeyCode>, Vec<KeyCode>, Vec<u32>) {
    let mut sorted = source.to_vec();
    sorted.sort_unstable_by_key(|&(c1, _, _, _)| c1);
    let mut starts = vec![0; MAX_KEYCODE_SPACE + 1];
    let mut o1 = Vec::with_capacity(source.len());
    let mut o2 = Vec::with_capacity(source.len());
    let mut freqs = Vec::with_capacity(source.len());

    let mut current_char = 0usize;
    let mut current_offset = 0usize;

    for &(c1, c2, c3, freq) in &sorted {
        let c1 = c1 as usize;
        while current_char <= c1 {
            starts[current_char] = current_offset;
            current_char += 1;
        }
        o1.push(KeyCode::new(c2));
        o2.push(KeyCode::new(c3));
        freqs.push(freq);
        current_offset += 1;
    }

    while current_char <= MAX_KEYCODE_SPACE {
        starts[current_char] = current_offset;
        current_char += 1;
    }

    (starts, o1, o2, freqs)
}
