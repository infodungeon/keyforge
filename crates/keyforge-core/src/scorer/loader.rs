use crate::error::{KeyForgeError, KfResult};
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use tracing::{debug, info};

pub struct RawCostData {
    pub entries: Vec<(String, String, f32)>,
}

#[derive(Clone)]
pub struct CorpusBundle {
    pub char_freqs: [f32; 256],
    pub bigrams: Vec<(u8, u8, f32)>,
    pub trigrams: Vec<(u8, u8, u8, f32)>,
    pub words: Vec<(String, u64)>,
}

impl Default for CorpusBundle {
    fn default() -> Self {
        Self {
            char_freqs: [0.0; 256],
            bigrams: Vec::new(),
            trigrams: Vec::new(),
            words: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TrigramRef {
    pub other1: u8,
    pub other2: u8,
    pub freq: f32,
    pub role: u8,
}

pub fn load_cost_matrix<P: AsRef<Path>>(path: P) -> KfResult<RawCostData> {
    let file = File::open(path)?;
    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .has_headers(true)
        .from_reader(file);

    let mut entries = Vec::new();

    for result in rdr.records().flatten() {
        if result.len() < 3 {
            continue;
        }
        let k1 = result[0].trim().to_string();
        let k2 = result[1].trim().to_string();
        if let Ok(cost) = result[2].trim().parse::<f32>() {
            if cost.is_finite() && cost >= 0.0 {
                entries.push((k1, k2, cost));
            }
        }
    }
    Ok(RawCostData { entries })
}

fn parse_token(s: &str) -> u8 {
    match s {
        "\\n" => 10,
        "\\\\" => 92,
        _ => s.as_bytes().first().cloned().unwrap_or(0),
    }
}

fn load_single_corpus(root: &Path, scale: f32, limit: usize) -> KfResult<CorpusBundle> {
    let mut bundle = CorpusBundle::default();

    // 1. Monograms
    let p1 = root.join("1grams.csv");
    if p1.exists() {
        let mut rdr = csv::Reader::from_path(p1)?;
        for result in rdr.records().flatten() {
            let char_str = &result[0];
            let count: f32 = result[1].parse().unwrap_or(0.0);
            if !char_str.is_empty() {
                let b = parse_token(char_str);
                if b > 0 {
                    bundle.char_freqs[b as usize] = count / scale;
                }
            }
        }
    }

    // 2. Bigrams
    let p2 = root.join("2grams.csv");
    if p2.exists() {
        let mut rdr = csv::Reader::from_path(p2)?;
        for result in rdr.records().flatten() {
            let c1 = parse_token(&result[0]);
            let c2 = parse_token(&result[1]);
            let count: f32 = result[2].parse().unwrap_or(0.0);
            if c1 > 0 && c2 > 0 {
                bundle.bigrams.push((c1, c2, count / scale));
            }
        }
    }

    // 3. Trigrams
    let p3 = root.join("3grams.csv");
    if p3.exists() {
        let mut rdr = csv::Reader::from_path(p3)?;
        let mut count = 0;
        for result in rdr.records().flatten() {
            if limit > 0 && count >= limit {
                break;
            }
            let c1 = parse_token(&result[0]);
            let c2 = parse_token(&result[1]);
            let c3 = parse_token(&result[2]);
            let val: f32 = result[3].parse().unwrap_or(0.0);
            if c1 > 0 && c2 > 0 && c3 > 0 {
                bundle.trigrams.push((c1, c2, c3, val / scale));
                count += 1;
            }
        }
    }

    // 4. Words
    let p4 = root.join("words.csv");
    if p4.exists() {
        let mut rdr = csv::Reader::from_path(p4)?;
        for result in rdr.records().flatten() {
            let word = result[0].to_string();
            let freq: u64 = result[1].parse().unwrap_or(0);
            bundle.words.push((word, freq));
        }
    }

    Ok(bundle)
}

pub fn load_merged_bundle<P: AsRef<Path>>(
    base_dir: P,
    config_str: &str,
    global_scale: f32,
    max_trigrams: usize,
) -> KfResult<CorpusBundle> {
    let base = base_dir.as_ref();
    let mut final_bundle = CorpusBundle::default();

    let parts: Vec<&str> = config_str.split(',').collect();

    let mut merged_chars = [0.0f32; 256];
    let mut merged_bigrams: HashMap<(u8, u8), f32> = HashMap::new();
    let mut merged_trigrams: HashMap<(u8, u8, u8), f32> = HashMap::new();
    let mut merged_words: HashMap<String, u64> = HashMap::new();

    info!("📚 Loading and Blending Corpora: '{}'", config_str);

    for part in parts {
        let segs: Vec<&str> = part.split(':').collect();
        let name = segs[0].trim();
        if name.is_empty() {
            continue;
        }

        let weight: f32 = if segs.len() > 1 {
            segs[1].parse().unwrap_or(1.0)
        } else {
            1.0
        };

        // Resolve path
        let corpus_path = if name == "." {
            base.to_path_buf() // Special case for current dir (used in tests)
        } else if Path::new(name).exists() {
            Path::new(name).to_path_buf()
        } else {
            base.join(name)
        };

        if !corpus_path.exists() {
            return Err(KeyForgeError::Validation(format!(
                "Corpus not found: {:?}",
                corpus_path
            )));
        }

        let sub = load_single_corpus(&corpus_path, global_scale, max_trigrams)?;

        // Merge logic: weighted sum
        // FIXED: Use iterator instead of manual indexing
        for (i, freq) in sub.char_freqs.iter().enumerate() {
            merged_chars[i] += freq * weight;
        }

        for (c1, c2, f) in sub.bigrams {
            *merged_bigrams.entry((c1, c2)).or_default() += f * weight;
        }

        for (c1, c2, c3, f) in sub.trigrams {
            *merged_trigrams.entry((c1, c2, c3)).or_default() += f * weight;
        }

        for (w, f) in sub.words {
            *merged_words.entry(w).or_default() += (f as f32 * weight) as u64;
        }
    }

    final_bundle.char_freqs = merged_chars;

    for ((c1, c2), f) in merged_bigrams {
        final_bundle.bigrams.push((c1, c2, f));
    }
    final_bundle
        .bigrams
        .sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());

    for ((c1, c2, c3), f) in merged_trigrams {
        final_bundle.trigrams.push((c1, c2, c3, f));
    }
    final_bundle
        .trigrams
        .sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap());
    if max_trigrams > 0 && final_bundle.trigrams.len() > max_trigrams {
        final_bundle.trigrams.truncate(max_trigrams);
    }

    for (w, f) in merged_words {
        final_bundle.words.push((w, f));
    }
    final_bundle.words.sort_by(|a, b| b.1.cmp(&a.1));

    debug!(
        "Blended Corpus Stats: {} chars, {} bigrams, {} trigrams",
        final_bundle.char_freqs.iter().filter(|&&f| f > 0.0).count(),
        final_bundle.bigrams.len(),
        final_bundle.trigrams.len()
    );

    Ok(final_bundle)
}
