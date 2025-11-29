use csv;
use std::collections::HashSet;
use std::fs::File;

#[derive(Debug, Clone)]
pub struct TrigramRef {
    pub other1: u8,
    pub other2: u8,
    pub freq: f32,
    pub role: u8,
}

pub struct RawCostData {
    pub entries: Vec<(String, String, f32)>,
}

pub struct RawNgrams {
    pub bigrams: Vec<(u8, u8, f32)>,
    pub trigrams: Vec<(u8, u8, u8, f32)>,
    pub char_freqs: [f32; 256],
}

pub fn load_cost_matrix(path: &str, debug: bool) -> Result<RawCostData, String> {
    if debug {
        println!("   Loading Costs from: {}", path);
    }

    let file = File::open(path)
        .map_err(|e| format!("❌ Could not open Cost Matrix at '{}': {}", path, e))?;

    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .has_headers(true)
        .from_reader(file);

    let mut entries = Vec::new();
    let mut skipped_count = 0;
    let mut row_idx = 0;

    for result in rdr.records() {
        row_idx += 1;
        match result {
            Ok(rec) => {
                if rec.len() < 3 {
                    skipped_count += 1;
                    continue;
                }

                let k1 = rec[0].trim().to_string();
                let k2 = rec[1].trim().to_string();

                let c: f32 = match rec[2].trim().parse() {
                    Ok(val) => val,
                    Err(_) => {
                        skipped_count += 1;
                        continue;
                    }
                };

                entries.push((k1, k2, c));
            }
            Err(e) => {
                if debug {
                    eprintln!("   ⚠️  [Row {}] CSV Parse Error: {}", row_idx, e);
                }
            }
        }
    }

    if debug && skipped_count > 0 {
        println!(
            "   ⚠️  Skipped {} invalid rows in Cost Matrix.",
            skipped_count
        );
    }

    Ok(RawCostData { entries })
}

pub fn load_ngrams(
    path: &str,
    valid: &HashSet<u8>,
    corpus_scale: f32,
    debug: bool,
) -> Result<RawNgrams, String> {
    if debug {
        println!("   Loading Ngrams from: {}", path);
    }

    let file = File::open(path)
        .map_err(|e| format!("❌ Could not open N-grams file at '{}': {}", path, e))?;

    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .quoting(false)
        .flexible(true)
        .from_reader(file);

    let mut bigrams = Vec::new();
    let mut trigrams = Vec::new();
    let mut char_freqs = [0.0; 256];
    let mut lines_read = 0;

    for result in rdr.records() {
        lines_read += 1;
        if let Ok(rec) = result {
            if rec.len() < 2 {
                continue;
            }

            let s_raw = rec[0].trim();
            if s_raw.is_empty() {
                continue;
            }
            let s = s_raw.to_ascii_lowercase();

            let count_val: f32 = match rec[1].trim().parse() {
                Ok(v) => v,
                Err(_) => continue,
            };

            let normalized_freq = count_val / corpus_scale;
            let bytes = s.as_bytes();

            if !bytes.iter().all(|b| valid.contains(b)) {
                continue;
            }

            match s.len() {
                1 => char_freqs[bytes[0] as usize] += normalized_freq,
                2 => bigrams.push((bytes[0], bytes[1], normalized_freq)),
                3 => trigrams.push((bytes[0], bytes[1], bytes[2], normalized_freq)),
                _ => {}
            }
        }
    }

    if debug {
        println!(
            "   -> Scanned {} lines. Loaded: {} 2-grams, {} 3-grams.",
            lines_read,
            bigrams.len(),
            trigrams.len()
        );
    }

    Ok(RawNgrams {
        bigrams,
        trigrams,
        char_freqs,
    })
}
