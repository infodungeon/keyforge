use super::CompilationStage;
use crate::error::PhysicsError;
use crate::kernel::types::KeyCode;
use keyforge_model::constants::MAX_KEYCODE_SPACE;
use keyforge_model::{Corpus, Rubric};

/// Intermediate state containing flattened and pruned corpus data.
#[derive(Debug)]
pub struct CorpusOutput {
    pub char_freqs: Vec<u64>,
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
    pub trigram_mid_starts: Vec<usize>,
    pub trigram_mid_others1: Vec<KeyCode>,
    pub trigram_mid_others2: Vec<KeyCode>,
    pub trigram_mid_freqs: Vec<u32>,
    pub trigram_end_starts: Vec<usize>,
    pub trigram_end_others1: Vec<KeyCode>,
    pub trigram_end_others2: Vec<KeyCode>,
    pub trigram_end_freqs: Vec<u32>,
}

/// Stage 3: Corpus Flattening & Pruning.
#[derive(Debug)]
pub struct CorpusStage<'a> {
    pub corpus: &'a Corpus,
    pub rubric: &'a Rubric,
}

impl CompilationStage for CorpusStage<'_> {
    type Input = ();
    type Output = CorpusOutput;

    fn execute(&self, (): Self::Input) -> Result<Self::Output, PhysicsError> {
        // Merge bigram duplicates to ensure consistency
        let mut bigrams = self.corpus.bigrams.clone();
        bigrams.sort_unstable_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
        let mut merged_bigrams = Vec::with_capacity(bigrams.len());
        if !bigrams.is_empty() {
            let mut current = bigrams[0];
            for next in bigrams.into_iter().skip(1) {
                if next.0 == current.0 && next.1 == current.1 {
                    current.2 = current.2.saturating_add(next.2);
                } else {
                    merged_bigrams.push(current);
                    current = next;
                }
            }
            merged_bigrams.push(current);
        }

        let (bigram_starts, bigram_others, bigram_freqs) = flatten_bigrams(&merged_bigrams);
        let (bigram_rev_starts, bigram_rev_others, bigram_rev_freqs) =
            flatten_bigrams_rev(&merged_bigrams);

        let pruned_trigrams = prune_trigrams(
            self.corpus.trigrams.clone(),
            self.rubric.trigram_coverage,
            self.rubric.trigram_limit,
        );

        let (trigram_starts, trigram_others1, trigram_others2, trigram_freqs) =
            flatten_trigrams_start(&pruned_trigrams);
        let (trigram_mid_starts, trigram_mid_others1, trigram_mid_others2, trigram_mid_freqs) =
            flatten_trigrams_mid(&pruned_trigrams);
        let (trigram_end_starts, trigram_end_others1, trigram_end_others2, trigram_end_freqs) =
            flatten_trigrams_end(&pruned_trigrams);

        let char_freqs = self.corpus.char_freqs.clone();

        Ok(CorpusOutput {
            char_freqs,
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
            trigram_mid_starts,
            trigram_mid_others1,
            trigram_mid_others2,
            trigram_mid_freqs,
            trigram_end_starts,
            trigram_end_others1,
            trigram_end_others2,
            trigram_end_freqs,
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
        others.push(KeyCode(c2));
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
        others.push(KeyCode(c1));
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
    mut source: Vec<(u16, u16, u16, u32)>,
    coverage: f32,
    limit: usize,
) -> Vec<(u16, u16, u16, u32)> {
    if source.is_empty() {
        return source;
    }

    // First, merge duplicates to ensure consistent scoring between full and delta passes.
    // Task-phys-rev-044: Seen set in delta calculation requires unique trigram keys.
    source.sort_unstable_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));
    let mut merged = Vec::with_capacity(source.len());
    if !source.is_empty() {
        let mut current = source[0];
        for next in source.into_iter().skip(1) {
            if next.0 == current.0 && next.1 == current.1 && next.2 == current.2 {
                current.3 = current.3.saturating_add(next.3);
            } else {
                merged.push(current);
                current = next;
            }
        }
        merged.push(current);
    }
    let mut source = merged;

    source.sort_unstable_by(|a, b| {
        b.3.cmp(&a.3)
            .then_with(|| a.0.cmp(&b.0))
            .then_with(|| a.1.cmp(&b.1))
            .then_with(|| a.2.cmp(&b.2))
    });
    let total_freq: u64 = source.iter().map(|x| u64::from(x.3)).sum();
    #[allow(clippy::cast_possible_truncation)]
    let target = (total_freq as f64 * f64::from(coverage)) as u64;
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
        o1.push(KeyCode(c2));
        o2.push(KeyCode(c3));
        freqs.push(freq);
        current_offset += 1;
    }

    while current_char <= MAX_KEYCODE_SPACE {
        starts[current_char] = current_offset;
        current_char += 1;
    }

    (starts, o1, o2, freqs)
}

fn flatten_trigrams_mid(
    source: &[(u16, u16, u16, u32)],
) -> (Vec<usize>, Vec<KeyCode>, Vec<KeyCode>, Vec<u32>) {
    let mut sorted = source.to_vec();
    sorted.sort_unstable_by_key(|&(_, c2, _, _)| c2);

    let mut starts = vec![0; MAX_KEYCODE_SPACE + 1];
    let mut o1 = Vec::with_capacity(source.len());
    let mut o2 = Vec::with_capacity(source.len());
    let mut freqs = Vec::with_capacity(source.len());

    let mut current_char = 0usize;
    let mut current_offset = 0usize;

    for &(c1, c2, c3, freq) in &sorted {
        let c2 = c2 as usize;
        while current_char <= c2 {
            starts[current_char] = current_offset;
            current_char += 1;
        }
        o1.push(KeyCode(c1));
        o2.push(KeyCode(c3));
        freqs.push(freq);
        current_offset += 1;
    }

    while current_char <= MAX_KEYCODE_SPACE {
        starts[current_char] = current_offset;
        current_char += 1;
    }

    (starts, o1, o2, freqs)
}

fn flatten_trigrams_end(
    source: &[(u16, u16, u16, u32)],
) -> (Vec<usize>, Vec<KeyCode>, Vec<KeyCode>, Vec<u32>) {
    let mut sorted = source.to_vec();
    sorted.sort_unstable_by_key(|&(_, _, c3, _)| c3);

    let mut starts = vec![0; MAX_KEYCODE_SPACE + 1];
    let mut o1 = Vec::with_capacity(source.len());
    let mut o2 = Vec::with_capacity(source.len());
    let mut freqs = Vec::with_capacity(source.len());

    let mut current_char = 0usize;
    let mut current_offset = 0usize;

    for &(c1, c2, c3, freq) in &sorted {
        let c3 = c3 as usize;
        while current_char <= c3 {
            starts[current_char] = current_offset;
            current_char += 1;
        }
        o1.push(KeyCode(c1));
        o2.push(KeyCode(c2));
        freqs.push(freq);
        current_offset += 1;
    }

    while current_char <= MAX_KEYCODE_SPACE {
        starts[current_char] = current_offset;
        current_char += 1;
    }

    (starts, o1, o2, freqs)
}
