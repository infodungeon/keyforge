// ===== keyforge/crates/keyforge-core/src/corpus.rs =====
use std::collections::HashMap;

/// Generates a TSV string containing Monograms, Bigrams, and Trigrams from raw text.
///
/// # Arguments
/// * `content` - The raw source text.
/// * `top_n` - Limit for bigrams/trigrams (e.g., 3000 to keep file size small).
pub fn generate_ngrams(content: &str, top_n: usize) -> String {
    let mut monograms: HashMap<char, usize> = HashMap::new();
    let mut bigrams: HashMap<String, usize> = HashMap::new();
    let mut trigrams: HashMap<String, usize> = HashMap::new();

    // 1. Normalize & Filter
    // We only care about characters that usually appear on a keyboard's base layer
    // plus common punctuation. Lowercase everything.
    let valid_chars = "abcdefghijklmnopqrstuvwxyz.,;'[]-!?:\"()";

    let clean_text: Vec<char> = content
        .to_lowercase()
        .chars()
        .filter(|c| valid_chars.contains(*c) || c.is_whitespace())
        .collect();

    let filtered: Vec<char> = clean_text
        .into_iter()
        .filter(|c| !c.is_whitespace())
        .collect();

    // 2. Sliding Window Analysis
    for i in 0..filtered.len() {
        // Monogram
        let c = filtered[i];
        *monograms.entry(c).or_default() += 1;

        // Bigram
        if i + 1 < filtered.len() {
            let s: String = filtered[i..i + 2].iter().collect();
            *bigrams.entry(s).or_default() += 1;
        }

        // Trigram
        if i + 2 < filtered.len() {
            let s: String = filtered[i..i + 3].iter().collect();
            *trigrams.entry(s).or_default() += 1;
        }
    }

    // 3. Format Output
    let mut output = String::new();

    // Append Monograms (All)
    for (c, count) in monograms {
        output.push_str(&format!("{}\t{}\n", c, count));
    }

    // Helper to sort and append top N items
    fn append_ngrams(output: &mut String, map: HashMap<String, usize>, limit: usize) {
        let mut entries: Vec<_> = map.into_iter().collect();
        // Sort DESC by count
        entries.sort_by(|a, b| b.1.cmp(&a.1));

        for (k, v) in entries.into_iter().take(limit) {
            output.push_str(&format!("{}\t{}\n", k, v));
        }
    }

    append_ngrams(&mut output, bigrams, top_n);
    append_ngrams(&mut output, trigrams, top_n);

    output
}