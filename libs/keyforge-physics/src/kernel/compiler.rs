use super::mechanics::calculate_pair_cost;
use super::EngineContext;
use keyforge_model::{Corpus, Keyboard, Rubric};
use keyforge_protocol::constants::SCORE_SCALE;
use tracing::instrument;

pub struct Compiler;

impl Compiler {
    #[instrument(skip_all)]
    pub fn compile(
        kb: &Keyboard,
        corpus: &Corpus,
        rubric: &Rubric,
        overrides: &[(usize, usize, f32)],
    ) -> EngineContext {
        let key_count = kb.count();

        let mut hands = Vec::with_capacity(key_count);
        let mut fingers = Vec::with_capacity(key_count);
        let mut rows = Vec::with_capacity(key_count);
        let mut cols = Vec::with_capacity(key_count);

        for k in &kb.keys {
            hands.push(k.hand);
            fingers.push(k.finger);
            rows.push(k.row);
            cols.push(k.col);
        }

        let mut cost_matrix = vec![0i64; key_count * key_count];

        for i in 0..key_count {
            for j in 0..key_count {
                let cost = calculate_pair_cost(kb, rubric, i, j);
                cost_matrix[i * key_count + j] = safe_float_to_int(cost);
            }
        }

        for &(i, j, cost) in overrides {
            if i < key_count && j < key_count {
                cost_matrix[i * key_count + j] = safe_float_to_int(cost);
            }
        }

        // --- Bigrams ---
        let (bigram_starts, bigram_others, bigram_freqs) = flatten_bigrams(&corpus.bigrams);
        let (bigram_rev_starts, bigram_rev_others, bigram_rev_freqs) =
            flatten_bigrams_rev(&corpus.bigrams);

        // --- Trigrams (Adaptive Pruning) ---
        let pruned_trigrams = prune_trigrams(
            corpus.trigrams.clone(),
            rubric.trigram_coverage,
            rubric.trigram_limit,
        );

        let (trigram_starts, trigram_others1, trigram_others2, trigram_freqs) =
            flatten_trigrams_start(&pruned_trigrams);

        let (trigram_mid_starts, trigram_mid_others1, trigram_mid_others2, trigram_mid_freqs) =
            flatten_trigrams_mid(&pruned_trigrams);

        let (trigram_end_starts, trigram_end_others1, trigram_end_others2, trigram_end_freqs) =
            flatten_trigrams_end(&pruned_trigrams);

        // Direct copy from fixed array
        let char_freqs = corpus.char_freqs;

        EngineContext {
            key_count,
            hands,
            fingers,
            rows,
            cols,
            cost_matrix,
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

            penalty_redirect: safe_float_to_int(rubric.redirect),
            penalty_skip: 0,
            bonus_roll: safe_float_to_int(rubric.roll_bonus),
        }
    }
}

fn safe_float_to_int(val: f32) -> i64 {
    if val.is_nan() {
        return 0;
    }
    if val.is_infinite() {
        return if val.is_sign_positive() {
            i64::MAX
        } else {
            i64::MIN
        };
    }

    // P2 FIX: High Precision Scaling
    let scaled = val * SCORE_SCALE;

    if scaled >= i64::MAX as f32 {
        return i64::MAX;
    }
    if scaled <= i64::MIN as f32 {
        return i64::MIN;
    }

    scaled as i64
}

fn flatten_bigrams(source: &[(u16, u16, u32)]) -> (Vec<usize>, Vec<u16>, Vec<u32>) {
    let mut buckets = vec![Vec::new(); 256];
    for &(c1, c2, freq) in source {
        if (c1 as usize) < 256 {
            buckets[c1 as usize].push((c2, freq));
        }
    }
    flatten_buckets(buckets)
}

fn flatten_bigrams_rev(source: &[(u16, u16, u32)]) -> (Vec<usize>, Vec<u16>, Vec<u32>) {
    let mut buckets = vec![Vec::new(); 256];
    for &(c1, c2, freq) in source {
        if (c2 as usize) < 256 {
            buckets[c2 as usize].push((c1, freq));
        }
    }
    flatten_buckets(buckets)
}

fn prune_trigrams(
    mut source: Vec<(u16, u16, u16, u32)>,
    coverage: f32,
    limit: usize,
) -> Vec<(u16, u16, u16, u32)> {
    source.sort_unstable_by(|a, b| {
        b.3.cmp(&a.3)
            .then_with(|| a.0.cmp(&b.0))
            .then_with(|| a.1.cmp(&b.1))
            .then_with(|| a.2.cmp(&b.2))
    });

    let total_freq: u64 = source.iter().map(|x| x.3 as u64).sum();
    let target = (total_freq as f64 * coverage as f64) as u64;
    let mut acc = 0;
    let mut cutoff = source.len();

    for (i, item) in source.iter().enumerate() {
        acc += item.3 as u64;
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
) -> (Vec<usize>, Vec<u16>, Vec<u16>, Vec<u32>) {
    let mut buckets = vec![Vec::new(); 256];
    for &(c1, c2, c3, freq) in source {
        if (c1 as usize) < 256 {
            buckets[c1 as usize].push((c2, c3, freq));
        }
    }
    flatten_buckets_tri(buckets)
}

fn flatten_trigrams_mid(
    source: &[(u16, u16, u16, u32)],
) -> (Vec<usize>, Vec<u16>, Vec<u16>, Vec<u32>) {
    let mut buckets = vec![Vec::new(); 256];
    for &(c1, c2, c3, freq) in source {
        if (c2 as usize) < 256 {
            buckets[c2 as usize].push((c1, c3, freq));
        }
    }
    flatten_buckets_tri(buckets)
}

fn flatten_trigrams_end(
    source: &[(u16, u16, u16, u32)],
) -> (Vec<usize>, Vec<u16>, Vec<u16>, Vec<u32>) {
    let mut buckets = vec![Vec::new(); 256];
    for &(c1, c2, c3, freq) in source {
        if (c3 as usize) < 256 {
            buckets[c3 as usize].push((c1, c2, freq));
        }
    }
    flatten_buckets_tri(buckets)
}

fn flatten_buckets(buckets: Vec<Vec<(u16, u32)>>) -> (Vec<usize>, Vec<u16>, Vec<u32>) {
    let mut starts = vec![0; 257];
    let mut others = Vec::new();
    let mut freqs = Vec::new();
    let mut offset = 0;

    for i in 0..256 {
        starts[i] = offset;
        for (o, f) in &buckets[i] {
            others.push(*o);
            freqs.push(*f);
        }
        offset += buckets[i].len();
    }
    starts[256] = offset;
    (starts, others, freqs)
}

fn flatten_buckets_tri(
    buckets: Vec<Vec<(u16, u16, u32)>>,
) -> (Vec<usize>, Vec<u16>, Vec<u16>, Vec<u32>) {
    let mut starts = vec![0; 257];
    let mut o1 = Vec::new();
    let mut o2 = Vec::new();
    let mut freqs = Vec::new();
    let mut offset = 0;

    for i in 0..256 {
        starts[i] = offset;
        for (a, b, f) in &buckets[i] {
            o1.push(*a);
            o2.push(*b);
            freqs.push(*f);
        }
        offset += buckets[i].len();
    }
    starts[256] = offset;
    (starts, o1, o2, freqs)
}
