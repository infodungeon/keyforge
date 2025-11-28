use csv;
use std::collections::HashSet;
use std::fs::File;
use std::process;

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

pub fn load_cost_matrix(path: &str, debug: bool) -> RawCostData {
    if debug {
        println!("   Loading Costs from: {}", path);
    }

    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("\n❌ FATAL: Could not open Cost Matrix file at '{}'", path);
            eprintln!("    Error: {}", e);
            process::exit(1);
        }
    };

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
                    if debug {
                        eprintln!("   ⚠️  [Row {}] Skipped: Missing columns", row_idx);
                    }
                    continue;
                }

                let k1 = rec[0].trim().to_string();
                let k2 = rec[1].trim().to_string();

                let c: f32 = match rec[2].trim().parse() {
                    Ok(val) => val,
                    Err(_) => {
                        skipped_count += 1;
                        if debug {
                            eprintln!(
                                "   ⚠️  [Row {}] Skipped: Invalid cost '{}'",
                                row_idx, &rec[2]
                            );
                        }
                        continue;
                    }
                };

                entries.push((k1, k2, c));
            }
            Err(e) => {
                skipped_count += 1;
                if debug {
                    eprintln!("   ⚠️  [Row {}] CSV Parse Error: {}", row_idx, e);
                }
            }
        }
    }

    if skipped_count > 0 {
        eprintln!(
            "   ⚠️  WARNING: Skipped {} invalid rows in Cost Matrix.",
            skipped_count
        );
    }
    if debug {
        println!("   -> Successfully parsed {} entries.", entries.len());
    }

    RawCostData { entries }
}

pub struct RawNgrams {
    pub bigrams: Vec<(u8, u8, f32)>,
    pub trigrams: Vec<(u8, u8, u8, f32)>,
    pub char_freqs: [f32; 256],
}

pub fn load_ngrams(path: &str, valid: &HashSet<u8>, corpus_scale: f32, debug: bool) -> RawNgrams {
    if debug {
        println!("   Loading Ngrams from: {}", path);
    }

    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("\n❌ FATAL: Could not open N-grams file at '{}'", path);
            eprintln!("    Error: {}", e);
            process::exit(1);
        }
    };

    // Use lenient parsing settings
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .quoting(false)
        .flexible(true) // Allow rows with varying column counts
        .from_reader(file);

    let mut bigrams = Vec::new();
    let mut trigrams = Vec::new();
    let mut char_freqs = [0.0; 256];

    let mut lines_read = 0;
    let mut skipped_format = 0;
    let mut skipped_char = 0;

    for result in rdr.records() {
        lines_read += 1;
        match result {
            Ok(rec) => {
                if rec.len() < 2 {
                    skipped_format += 1;
                    continue;
                }

                let s_raw = rec[0].trim();
                if s_raw.is_empty() {
                    skipped_format += 1;
                    continue;
                }
                let s = s_raw.to_ascii_lowercase();

                // Headers like "*/*" will fail parsing here and be skipped safely
                let count_val: f32 = match rec[1].trim().parse() {
                    Ok(v) => v,
                    Err(_) => {
                        skipped_format += 1;
                        continue;
                    }
                };

                let normalized_freq = count_val / corpus_scale;
                let bytes = s.as_bytes();

                // Check char validity
                let all_valid = bytes.iter().all(|b| valid.contains(b));
                if !all_valid {
                    skipped_char += 1;
                    continue;
                }

                let len = s.len();
                if len == 1 {
                    char_freqs[bytes[0] as usize] += normalized_freq;
                } else if len == 2 {
                    bigrams.push((bytes[0], bytes[1], normalized_freq));
                } else if len == 3 {
                    trigrams.push((bytes[0], bytes[1], bytes[2], normalized_freq));
                }
                // Lengths > 3 are silently ignored (expected for mixed n-gram files)
            }
            Err(_) => {
                skipped_format += 1;
            }
        }
    }

    if debug && (skipped_format > 0 || skipped_char > 0) {
        eprintln!(
            "   ⚠️  Skipped {} lines (Format) and {} lines (Invalid Char)",
            skipped_format, skipped_char
        );
    }

    if debug {
        println!(
            "   -> Scanned {} lines. Loaded: {} 2-grams, {} 3-grams.",
            lines_read,
            bigrams.len(),
            trigrams.len()
        );
    }

    RawNgrams {
        bigrams,
        trigrams,
        char_freqs,
    }
}
