// ===== keyforge/crates/keyforge-core/src/scorer/loader.rs =====
use crate::error::{KeyForgeError, KfResult};
use std::fs::File;
use std::path::Path;
use tracing::{debug, info};

pub struct RawCostData {
    pub entries: Vec<(String, String, f32)>,
}

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

pub fn load_corpus_bundle<P: AsRef<Path>>(
    dir: P,
    corpus_scale: f32,
    max_trigrams: usize,
) -> KfResult<CorpusBundle> {
    let root = dir.as_ref();
    let mut bundle = CorpusBundle::default();

    info!("📦 Loading Corpus Bundle from: {:?}", root);

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
                    bundle.char_freqs[b as usize] = count / corpus_scale;
                }
            }
        }
    } else {
        return Err(KeyForgeError::Validation(format!(
            "Missing 1grams.csv in {:?}",
            root
        )));
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
                bundle.bigrams.push((c1, c2, count / corpus_scale));
            }
        }
    }

    // 3. Trigrams
    let p3 = root.join("3grams.csv");
    if p3.exists() {
        let mut rdr = csv::Reader::from_path(p3)?;
        let mut count = 0;
        for result in rdr.records().flatten() {
            if max_trigrams > 0 && count >= max_trigrams {
                break;
            }

            let c1 = parse_token(&result[0]);
            let c2 = parse_token(&result[1]);
            let c3 = parse_token(&result[2]);
            let val: f32 = result[3].parse().unwrap_or(0.0);

            if c1 > 0 && c2 > 0 && c3 > 0 {
                bundle.trigrams.push((c1, c2, c3, val / corpus_scale));
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

    debug!(
        "Loaded Corpus: {} unique chars, {} bigrams, {} trigrams, {} words",
        bundle.char_freqs.iter().filter(|&&f| f > 0.0).count(),
        bundle.bigrams.len(),
        bundle.trigrams.len(),
        bundle.words.len()
    );

    Ok(bundle)
}
