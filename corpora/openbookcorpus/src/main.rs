use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use parquet::file::reader::{FileReader, SerializedFileReader};
use parquet::record::RowAccessor;
use rayon::prelude::*;
use reqwest::Client;
use rustc_hash::FxHashMap;
use serde::Serialize;
use serde_json::Value;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

// --- Configuration ---
const DATASET_NAME: &str = "lucadiliello/bookcorpusopen";
const CONCURRENT_DOWNLOADS: usize = 4;

// --- Dynamic Path Logic ---
// Returns the path to keyforge/corpora_data based on the crate location
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
struct Char1Stats { char: String, freq: usize }
#[derive(Serialize)]
struct Char2Stats { char1: String, char2: String, freq: usize }
#[derive(Serialize)]
struct Char3Stats { char1: String, char2: String, char3: String, freq: usize }
#[derive(Serialize)]
struct WordStats { word: String, freq: usize }

// Container for aggregating stats
struct CorpusStats {
    c1: FastMap<char, usize>,
    c2: FastMap<(char, char), usize>,
    c3: FastMap<(char, char, char), usize>,
    words: FastMap<String, usize>,
    book_count: usize, // <--- NEW FIELD
}

impl CorpusStats {
    fn new() -> Self {
        Self {
            c1: FastMap::default(),
            c2: FastMap::default(),
            c3: FastMap::default(),
            words: FastMap::default(),
            book_count: 0, // <--- INIT
        }
    }

    fn merge(&mut self, other: CorpusStats) {
        for (k, v) in other.c1 { *self.c1.entry(k).or_insert(0) += v; }
        for (k, v) in other.c2 { *self.c2.entry(k).or_insert(0) += v; }
        for (k, v) in other.c3 { *self.c3.entry(k).or_insert(0) += v; }
        for (k, v) in other.words { *self.words.entry(k).or_insert(0) += v; }
        self.book_count += other.book_count; // <--- MERGE COUNT
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

    #[inline(always)]
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

    #[inline(always)]
    fn reset(&mut self) {
        self.p1 = None;
        self.p2 = None;
    }
}

// --- Helper Functions ---
#[inline(always)]
fn is_keyboard_char(c: char) -> bool {
    match c {
        'a'..='z' | '0'..='9' => true,
        '.' | ',' | '!' | '?' | ';' | ':' | '\'' | '"' |
        '-' | '_' | '+' | '=' | '*' | '/' | '\\' | '|' |
        '(' | ')' | '[' | ']' | '{' | '}' | '<' | '>' |
        '@' | '#' | '$' | '%' | '^' | '&' | '`' | '~' => true,
        _ => false,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Setup Directory Path
    let data_dir = get_data_dir();
    
    if !data_dir.exists() {
        fs::create_dir_all(&data_dir)?;
    }
    
    println!("Using data directory: {:?}", data_dir);

    // 2. Get File List
    println!("Fetching file list for {}...", DATASET_NAME);
    let client = Client::new();
    let api_url = format!("https://datasets-server.huggingface.co/parquet?dataset={}", DATASET_NAME);
    
    let resp = client.get(&api_url).send().await?;
    if !resp.status().is_success() {
        println!("Failed to fetch file list. Status: {}", resp.status());
        return Ok(());
    }

    let json: Value = resp.json().await?;
    let files = json["parquet_files"].as_array().ok_or("Invalid API response format")?;
    let train_files: Vec<&Value> = files.iter().filter(|f| f["split"] == "train").collect();

    if train_files.is_empty() {
        println!("No training files found.");
        return Ok(());
    }
    println!("Found {} shard(s).", train_files.len());

    // 3. Download Shards
    let mut local_file_paths = Vec::new();
    let mut download_tasks = Vec::new();

    for (i, file_info) in train_files.iter().enumerate() {
        let url = file_info["url"].as_str().ok_or("Missing URL")?.to_string();
        
        let filename_path = data_dir.join(format!("shard_{}.parquet", i));
        let filename_str = filename_path.to_string_lossy().to_string();
        
        local_file_paths.push(filename_str.clone());

        if !filename_path.exists() || fs::metadata(&filename_path)?.len() < 1024 {
            download_tasks.push((url, filename_str));
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

    // 4. Analyze All Books
    println!("\nStarting parallel analysis (Optimized: Zero-Copy Stream + FxHash)...");
    let final_stats = process_dataset_parallel(&local_file_paths)?;

    // Print Book Count
    println!("\nAnalysis complete.");
    println!("Total books processed: {}", final_stats.book_count);

    // 5. Save Results
    println!("Saving results to JSON...");
    save_json_file("1grams.json", &final_stats.c1, |k, v| Char1Stats { char: k.to_string(), freq: *v })?;
    save_json_file("2grams.json", &final_stats.c2, |k, v| Char2Stats { char1: k.0.to_string(), char2: k.1.to_string(), freq: *v })?;
    save_json_file("3grams.json", &final_stats.c3, |k, v| Char3Stats { char1: k.0.to_string(), char2: k.1.to_string(), char3: k.2.to_string(), freq: *v })?;
    save_json_file("words.json", &final_stats.words, |k, v| WordStats { word: k.clone(), freq: *v })?;

    println!("Done.");
    Ok(())
}

async fn download_file(client: &Client, url: &str, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let res = client.get(url).send().await?;
    let content = res.bytes().await?;
    let mut file = File::create(path)?;
    file.write_all(&content)?;
    println!("Downloaded: {}", path);
    Ok(())
}

fn process_dataset_parallel(file_paths: &[String]) -> Result<CorpusStats, Box<dyn std::error::Error>> {
    let pb = ProgressBar::new(file_paths.len() as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} Shards ({eta})")?
        .progress_chars("#>-"));

    let aggregated_stats = file_paths.par_iter()
        .fold(CorpusStats::new, |mut stats, path| {
            if let Ok(file) = File::open(path) {
                if let Ok(reader) = SerializedFileReader::new(file) {
                    if let Ok(row_iter) = reader.get_row_iter(None) {
                        
                        let mut tracker = NgramTracker::new();
                        let mut word_buffer = String::with_capacity(32);

                        for row_result in row_iter {
                            if let Ok(row) = row_result {
                                // Increment book count for every row found
                                stats.book_count += 1;

                                if let Ok(text) = row.get_string(0) {
                                    
                                    let mut word_is_tainted = false;
                                    let mut last_emitted_was_space = false; 

                                    for c_raw in text.chars() {
                                        let mut process_normalized_char = |n_char: char| {
                                            for c in n_char.to_lowercase() {
                                                if is_keyboard_char(c) {
                                                    word_buffer.push(c);
                                                    last_emitted_was_space = false;
                                                } 
                                                else if c == ' ' || c == '\t' {
                                                    if !word_buffer.is_empty() {
                                                        if word_is_tainted {
                                                            tracker.reset();
                                                        } else {
                                                            for wc in word_buffer.chars() {
                                                                tracker.feed(wc, &mut stats);
                                                            }
                                                            *stats.words.entry(word_buffer.clone()).or_insert(0) += 1;
                                                        }
                                                        word_buffer.clear();
                                                        word_is_tainted = false;
                                                        last_emitted_was_space = false;
                                                    }
                                                    if !last_emitted_was_space {
                                                        tracker.feed(' ', &mut stats);
                                                        last_emitted_was_space = true;
                                                    }
                                                } 
                                                else if c == '\n' || c == '\r' {
                                                    if !word_buffer.is_empty() {
                                                        if word_is_tainted {
                                                            tracker.reset();
                                                        } else {
                                                            for wc in word_buffer.chars() {
                                                                tracker.feed(wc, &mut stats);
                                                            }
                                                            *stats.words.entry(word_buffer.clone()).or_insert(0) += 1;
                                                        }
                                                        word_buffer.clear();
                                                        word_is_tainted = false;
                                                    }
                                                    if c == '\n' {
                                                        tracker.feed('\n', &mut stats);
                                                        last_emitted_was_space = false;
                                                    }
                                                } 
                                                else {
                                                    word_is_tainted = true;
                                                }
                                            }
                                        };

                                        match c_raw {
                                            '“' | '”' | '„' => process_normalized_char('"'),
                                            '‘' | '’' | '`' | '´' => process_normalized_char('\''),
                                            '–' | '—' | '―' => process_normalized_char('-'),
                                            'ﬁ' => { process_normalized_char('f'); process_normalized_char('i'); },
                                            'ﬂ' => { process_normalized_char('f'); process_normalized_char('l'); },
                                            'ﬀ' => { process_normalized_char('f'); process_normalized_char('f'); },
                                            'ﬃ' => { process_normalized_char('f'); process_normalized_char('f'); process_normalized_char('i'); },
                                            'ﬄ' => { process_normalized_char('f'); process_normalized_char('f'); process_normalized_char('l'); },
                                            'æ' => { process_normalized_char('a'); process_normalized_char('e'); },
                                            'œ' => { process_normalized_char('o'); process_normalized_char('e'); },
                                            '\u{00ad}' | '\u{009d}' | '\\' | '_' => {}, 
                                            _ => process_normalized_char(c_raw),
                                        }
                                    }

                                    if !word_buffer.is_empty() {
                                        if !word_is_tainted {
                                            for wc in word_buffer.chars() {
                                                tracker.feed(wc, &mut stats);
                                            }
                                            *stats.words.entry(word_buffer.clone()).or_insert(0) += 1;
                                        }
                                        word_buffer.clear();
                                    }
                                    tracker.reset();
                                }
                            }
                        }
                    }
                }
            }
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

fn save_json_file<K, V, F, S>(filename: &str, map: &FastMap<K, V>, mapper: F) -> Result<(), Box<dyn std::error::Error>>
where 
    V: Ord + Copy,
    S: Serialize,
    F: Fn(&K, &V) -> S
{
    let mut vec: Vec<_> = map.iter().collect();
    vec.sort_by(|a, b| b.1.cmp(a.1)); 
    let json_out: Vec<S> = vec.iter().map(|(k, v)| mapper(k, v)).collect();
    let file = File::create(filename)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer(writer, &json_out)?;
    Ok(())
}