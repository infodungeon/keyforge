//! # `OpenBookCorpus` Processor
//!
//! A high-performance utility for downloading and processing the
//! [OpenBookCorpus](https://huggingface.co/datasets/lucadiliello/bookcorpusopen)
//! dataset into N-gram statistics for `KeyForge`.
//!
//! Features:
//! - Parallel processing using `rayon` and `parquet`.
//! - Concurrent shard downloads.
//! - Strict character normalization and validation for keyboard optimization.
//! - Optimized `FastMap` (`FxHash`) aggregation.
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use parquet::file::reader::{FileReader, SerializedFileReader};
use parquet::record::RowAccessor;
use rayon::prelude::*;
use reqwest::Client;
use rustc_hash::FxHashMap;
use serde::Serialize;
use serde_json::ser::{Formatter, Serializer};
use serde_json::Value;
use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

// --- Configuration ---
const DATASET_NAME: &str = "lucadiliello/bookcorpusopen";
const CONCURRENT_DOWNLOADS: usize = 4;

// --- Dynamic Path Logic ---
#[allow(clippy::expect_used)]
fn get_data_dir() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .expect("Could not find parent directory of crate")
        .parent()
        .expect("Could not find workspace root")
        .join("corpora_data")
}

// --- Type Alias for Performance ---
type FastMap<K, V> = FxHashMap<K, V>;

// --- Data Structures for JSON Output ---
#[derive(Serialize)]
struct Char1Stats {
    char: String,
    freq: usize,
}
#[derive(Serialize)]
struct Char2Stats {
    char1: String,
    char2: String,
    freq: usize,
}
#[derive(Serialize)]
struct Char3Stats {
    char1: String,
    char2: String,
    char3: String,
    freq: usize,
}
#[derive(Serialize)]
struct WordStats {
    word: String,
    freq: usize,
}

// Container for aggregating stats
struct CorpusStats {
    c1: FastMap<char, usize>,
    c2: FastMap<(char, char), usize>,
    c3: FastMap<(char, char, char), usize>,
    words: FastMap<String, usize>,
    book_count: usize,
}

impl CorpusStats {
    fn new() -> Self {
        Self {
            c1: FastMap::default(),
            c2: FastMap::default(),
            c3: FastMap::default(),
            words: FastMap::default(),
            book_count: 0,
        }
    }

    fn merge(&mut self, other: CorpusStats) {
        for (k, v) in other.c1 {
            *self.c1.entry(k).or_insert(0) += v;
        }
        for (k, v) in other.c2 {
            *self.c2.entry(k).or_insert(0) += v;
        }
        for (k, v) in other.c3 {
            *self.c3.entry(k).or_insert(0) += v;
        }
        for (k, v) in other.words {
            *self.words.entry(k).or_insert(0) += v;
        }
        self.book_count += other.book_count;
    }
}

// --- Optimized N-Gram Tracker ---
struct NgramTracker {
    p1: Option<char>,
    p2: Option<char>,
}

impl NgramTracker {
    fn new() -> Self {
        Self { p1: None, p2: None }
    }

    #[inline]
    fn feed(&mut self, c: char, stats: &mut CorpusStats) {
        *stats.c1.entry(c).or_insert(0) += 1;
        if let Some(last) = self.p1 {
            *stats.c2.entry((last, c)).or_insert(0) += 1;
            if let Some(second_last) = self.p2 {
                *stats.c3.entry((second_last, last, c)).or_insert(0) += 1;
            }
        }
        self.p2 = self.p1;
        self.p1 = Some(c);
    }

    #[inline]
    fn reset(&mut self) {
        self.p1 = None;
        self.p2 = None;
    }
}

// --- Strict Escape Formatter ---
struct StrictEscapeFormatter {
    is_key: bool,
}

impl StrictEscapeFormatter {
    fn new() -> Self {
        Self { is_key: false }
    }
}

impl Formatter for StrictEscapeFormatter {
    fn begin_object_key<W: ?Sized + Write>(
        &mut self,
        writer: &mut W,
        first: bool,
    ) -> io::Result<()> {
        if !first {
            writer.write_all(b",")?;
        }
        self.is_key = true;
        Ok(())
    }
    fn end_object_key<W: ?Sized + Write>(&mut self, _writer: &mut W) -> io::Result<()> {
        self.is_key = false;
        Ok(())
    }
    fn begin_object_value<W: ?Sized + Write>(&mut self, writer: &mut W) -> io::Result<()> {
        writer.write_all(b":")?;
        Ok(())
    }
    fn write_string_fragment<W: ?Sized + Write>(
        &mut self,
        writer: &mut W,
        fragment: &str,
    ) -> io::Result<()> {
        if self.is_key {
            writer.write_all(fragment.as_bytes())?;
        } else {
            for c in fragment.chars() {
                let c_u32 = c as u32;
                if c_u32 <= 0xFFFF {
                    write!(writer, "\\u{c_u32:04x}")?;
                } else {
                    let c_u32 = c_u32 - 0x10000;
                    let high = 0xD800 + (c_u32 >> 10);
                    let low = 0xDC00 + (c_u32 & 0x3FF);
                    write!(writer, "\\u{high:04x}\\u{low:04x}")?;
                }
            }
        }
        Ok(())
    }
}

// --- Validation Helpers ---

#[inline]
fn is_keyboard_char(c: char) -> bool {
    matches!(c, 'a'..='z' | '0'..='9' | '.' | ',' | '!' | '?' | ';' | ':' | '\'' | '"' |
        '-' | '_' | '+' | '=' | '*' | '/' | '\\' | '|' |
        '(' | ')' | '[' | ']' | '{' | '}' | '<' | '>' |
        '@' | '#' | '$' | '%' | '^' | '&' | '`' | '~')
}

#[inline]
fn has_vowel(s: &str) -> bool {
    s.chars()
        .any(|c| matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'y'))
}

#[inline]
fn has_repeated_chars_3(s: &str) -> bool {
    let mut chars = s.chars();
    if let Some(mut p2) = chars.next() {
        if let Some(mut p1) = chars.next() {
            for c in chars {
                if c == p1 && p1 == p2 {
                    return true;
                }
                p2 = p1;
                p1 = c;
            }
        }
    }
    false
}

#[inline]
fn has_consonant_cluster_7(s: &str) -> bool {
    let mut count = 0;
    for c in s.chars() {
        if matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'y') {
            count = 0;
        } else if c.is_ascii_alphabetic() {
            count += 1;
            if count >= 7 {
                return true;
            }
        } else {
            count = 0;
        }
    }
    false
}

#[inline]
fn is_valid_word(s: &str) -> bool {
    if s.is_empty() || s.len() > 25 {
        return false;
    }
    if !s
        .chars()
        .all(|c| c.is_ascii_alphabetic() || c == '\'' || c == '-')
    {
        return false;
    }
    if !has_vowel(s) {
        return false;
    }
    if has_repeated_chars_3(s) {
        return false;
    }
    if has_consonant_cluster_7(s) {
        return false;
    }
    true
}

#[allow(clippy::expect_used)]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data_dir = get_data_dir();
    if !data_dir.exists() {
        fs::create_dir_all(&data_dir)?;
    }

    println!("Using data directory: {}", data_dir.display());

    // Fetch File List
    let client = Client::new();
    let api_url = format!("https://datasets-server.huggingface.co/parquet?dataset={DATASET_NAME}");
    let resp = client.get(&api_url).send().await?;
    let json: Value = resp.json().await?;
    let files = json["parquet_files"]
        .as_array()
        .ok_or("Invalid API response")?;
    let train_files: Vec<&Value> = files.iter().filter(|f| f["split"] == "train").collect();

    // Download
    let mut local_file_paths = Vec::new();
    let mut download_tasks = Vec::new();
    for (i, file_info) in train_files.iter().enumerate() {
        let url = file_info["url"].as_str().ok_or("Missing URL")?.to_string();
        let filename = data_dir.join(format!("shard_{i}.parquet"));
        let path_str = filename.to_string_lossy().to_string();
        local_file_paths.push(path_str.clone());
        if !filename.exists() || fs::metadata(&filename)?.len() < 1024 {
            download_tasks.push((url, path_str));
        }
    }

    if !download_tasks.is_empty() {
        println!("Downloading {} missing shards...", download_tasks.len());
        let client_arc = Arc::new(client);
        futures_util::stream::iter(download_tasks)
            .map(|(url, path)| {
                let c = client_arc.clone();
                async move { download_file(&c, &url, &path).await }
            })
            .buffer_unordered(CONCURRENT_DOWNLOADS)
            .collect::<Vec<_>>()
            .await;
    }

    // Process
    println!("\nStarting parallel analysis...");
    let final_stats = process_dataset_parallel(&local_file_paths)?;

    println!(
        "\nAnalysis complete. Total books: {}",
        final_stats.book_count
    );
    println!("Saving results...");

    save_json_file(
        &data_dir,
        "1grams.json",
        &final_stats.c1,
        |k, v| Char1Stats {
            char: k.to_string(),
            freq: *v,
        },
        true,
    )?;
    save_json_file(
        &data_dir,
        "2grams.json",
        &final_stats.c2,
        |k, v| Char2Stats {
            char1: k.0.to_string(),
            char2: k.1.to_string(),
            freq: *v,
        },
        true,
    )?;
    save_json_file(
        &data_dir,
        "3grams.json",
        &final_stats.c3,
        |k, v| Char3Stats {
            char1: k.0.to_string(),
            char2: k.1.to_string(),
            char3: k.2.to_string(),
            freq: *v,
        },
        true,
    )?;
    save_json_file(
        &data_dir,
        "words.json",
        &final_stats.words,
        |k, v| WordStats {
            word: k.clone(),
            freq: *v,
        },
        false,
    )?;

    println!("Done.");
    Ok(())
}

async fn download_file(
    client: &Client,
    url: &str,
    path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let res = client.get(url).send().await?;
    let content = res.bytes().await?;
    let mut file = File::create(path)?;
    file.write_all(&content)?;
    Ok(())
}

fn process_token_buffer(
    buffer: &str,
    stats: &mut CorpusStats,
    tracker: &mut NgramTracker,
    last_emitted_was_space: &mut bool,
) {
    let cleaned = buffer.replace("--", " ");
    for segment in cleaned.split_whitespace() {
        if segment.matches('-').count() > 1 {
            for sub in segment.split('-') {
                validate_and_record(sub, stats, tracker, last_emitted_was_space);
            }
        } else {
            validate_and_record(segment, stats, tracker, last_emitted_was_space);
        }
    }
}

fn validate_and_record(
    raw_word: &str,
    stats: &mut CorpusStats,
    tracker: &mut NgramTracker,
    last_emitted_was_space: &mut bool,
) {
    let word = raw_word.trim_matches(|c: char| !c.is_alphabetic());
    if is_valid_word(word) {
        if !*last_emitted_was_space {
            tracker.feed(' ', stats);
        }
        for c in word.chars() {
            tracker.feed(c, stats);
        }
        *last_emitted_was_space = false;
        *stats.words.entry(word.to_string()).or_insert(0) += 1;
    }
}

fn process_dataset_parallel(
    file_paths: &[String],
) -> Result<CorpusStats, Box<dyn std::error::Error>> {
    let pb = ProgressBar::new(file_paths.len() as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len}s ({eta})")? // Corrected from Shards to s
        .progress_chars("#>- ")); // Added space to progress_chars

    let aggregated_stats = file_paths
        .par_iter()
        .fold(CorpusStats::new, |mut stats, path| {
            process_shard(path, &mut stats);
            pb.inc(1);
            stats
        })
        .reduce(CorpusStats::new, |mut a, b| {
            a.merge(b);
            a
        });

    pb.finish_with_message("Analysis complete");
    Ok(aggregated_stats)
}

fn process_shard(path: &str, stats: &mut CorpusStats) {
    if let Ok(file) = File::open(path) {
        if let Ok(reader) = SerializedFileReader::new(file) {
            if let Ok(row_iter) = reader.get_row_iter(None) {
                let mut word_buffer = String::with_capacity(64);

                for row in row_iter.flatten() {
                    stats.book_count += 1;
                    word_buffer.clear();
                    let mut tracker = NgramTracker::new();
                    let mut last_emitted_was_space = true;
                    let mut word_is_tainted = false;

                    if let Ok(text) = row.get_string(0) {
                        for c_raw in text.chars() {
                            let normalized_c = match c_raw {
                                '“' | '”' | '„' => '"',
                                '‘' | '’' | '`' | '´' => '\'',
                                '–' | '—' | '―' => '-',
                                'ﬁ' | 'ﬂ' => 'f',
                                '\u{00ad}' | '\u{009d}' | '\\' | '_' => ' ',
                                _ => c_raw.to_ascii_lowercase(),
                            };

                            if is_keyboard_char(normalized_c) {
                                if normalized_c.is_alphanumeric() 
                                    || normalized_c == '\'' 
                                    || normalized_c == '-'
                                {
                                    if !word_is_tainted {
                                        word_buffer.push(normalized_c);
                                    }
                                }
                                 else {
                                    if !word_buffer.is_empty() {
                                        if word_is_tainted {
                                            tracker.reset();
                                        } else {
                                            process_token_buffer(
                                                &word_buffer,
                                                stats,
                                                &mut tracker,
                                                &mut last_emitted_was_space,
                                            );
                                        }
                                        word_buffer.clear();
                                    }
                                    word_is_tainted = false;
                                    tracker.feed(normalized_c, stats);
                                    last_emitted_was_space = false;
                                }
                            } else if normalized_c == ' ' 
                                || normalized_c == '\n' 
                                || normalized_c == '\t' 
                                || normalized_c == '\r'
                            {
                                if !word_buffer.is_empty() {
                                    if word_is_tainted {
                                        tracker.reset();
                                        last_emitted_was_space = true;
                                    } else {
                                        process_token_buffer(
                                            &word_buffer,
                                            stats,
                                            &mut tracker,
                                            &mut last_emitted_was_space,
                                        );
                                    }
                                    word_buffer.clear();
                                }
                                word_is_tainted = false;

                                if !last_emitted_was_space {
                                    tracker.feed(' ', stats);
                                    last_emitted_was_space = true;
                                }
                                if normalized_c == '\n' {
                                    tracker.reset();
                                }
                            } else {
                                word_is_tainted = true;
                            }
                        }

                        if !word_buffer.is_empty() {
                            if !word_is_tainted {
                                process_token_buffer(
                                    &word_buffer,
                                    stats,
                                    &mut tracker,
                                    &mut last_emitted_was_space,
                                );
                            }
                            word_buffer.clear();
                        }
                    }
                }
            }
        }
    }
}

fn save_json_file<K, V, F, S>(
    dir: &Path,
    filename: &str,
    map: &FastMap<K, V>,
    mapper: F,
    use_strict_escaping: bool,
) -> Result<(), Box<dyn std::error::Error>>
where
    V: Ord + Copy,
    S: Serialize,
    F: Fn(&K, &V) -> S,
{
    let mut vec: Vec<_> = map.iter().collect();
    vec.sort_by(|a, b| b.1.cmp(a.1));
    let json_out: Vec<S> = vec.iter().map(|(k, v)| mapper(k, v)).collect();

    let file_path = dir.join(filename);
    let file = File::create(file_path)?;
    let writer = BufWriter::new(file);

    if use_strict_escaping {
        let mut serializer = Serializer::with_formatter(writer, StrictEscapeFormatter::new());
        json_out.serialize(&mut serializer)?;
    } else {
        serde_json::to_writer(writer, &json_out)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_mock_stats() -> CorpusStats {
        CorpusStats::new()
    }
    fn get_mock_tracker() -> NgramTracker {
        NgramTracker::new()
    }

    #[test]
    fn test_validity() {
        assert!(is_valid_word("hello"));
        assert!(!is_valid_word("123"));
        assert!(!is_valid_word("café"));
    }

    #[test]
    fn test_process_token_buffer_complex() {
        let mut stats = get_mock_stats();
        let mut tracker = get_mock_tracker();
        let mut last_space = true;

        // Test 1: Double dash split (--) AND Multi-hyphen split (word-word-word)
        // "please--state-of-the-art" -> "please" (ok), "state", "of", "the", "art"
        process_token_buffer(
            "please--state-of-the-art",
            &mut stats,
            &mut tracker,
            &mut last_space,
        );

        assert_eq!(*stats.words.get("please").unwrap(), 1);
        assert_eq!(*stats.words.get("state").unwrap(), 1);
        assert_eq!(*stats.words.get("of").unwrap(), 1);
        assert_eq!(*stats.words.get("the").unwrap(), 1);
        assert_eq!(*stats.words.get("art").unwrap(), 1);

        // Ensure the compound wasn't kept
        assert!(stats.words.get("please--state-of-the-art").is_none());
        assert!(stats.words.get("state-of-the-art").is_none());

        // Test 2: Single Hyphen Preservation
        let mut stats2 = get_mock_stats();
        let mut tracker2 = get_mock_tracker();
        let mut last_space2 = true;

        process_token_buffer("well-known", &mut stats2, &mut tracker2, &mut last_space2);
        assert_eq!(*stats2.words.get("well-known").unwrap(), 1);
    }
}