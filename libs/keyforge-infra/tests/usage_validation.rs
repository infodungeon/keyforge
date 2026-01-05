// Copyright (c) 2025 KeyForge Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use keyforge_infra::FsProvider;
use keyforge_core::loader::AssetLoader;
use keyforge_model::config::CorpusSource;
use keyforge_model::constants::CORPUS_TOKEN_MAP;
use std::path::{Path, PathBuf};
use std::fs::File;
use walkdir::WalkDir;

// Helper to resolve tokens manually (Shadow Logic)
fn resolve_token(token: &str) -> Option<char> {
    for (key, val) in CORPUS_TOKEN_MAP {
        if token == *key { return Some(*val); }
    }
    if token.chars().count() == 1 {
        token.chars().next().map(|c| c.to_ascii_lowercase())
    } else {
        None
    }
}

async fn validate_corpus(root: &Path, id: &str) -> Vec<String> {
    let mut errors = Vec::new();
    let corpus_path = root.join("system/corpora").join(id);

    // 1. Load via System
    let provider = FsProvider::new(root.to_path_buf());
    let sources = vec![CorpusSource {
        id: id.to_string(),
        weight: 1.0,
        hash: None,
    }];
    
    let system_corpus = match provider.load_corpus(&sources).await {
        Ok(c) => c,
        Err(e) => {
            errors.push(format!("[{}] Failed to load: {}", id, e));
            return errors;
        }
    };

    // 2. Load Raw 1-grams
    let p1 = corpus_path.join("1grams.mpk.zst");
    if !p1.exists() {
        errors.push(format!("[{}] 1grams.mpk.zst missing", id));
        return errors;
    }
    
    let f1 = File::open(&p1).unwrap();
    let raw_1grams: Vec<serde_json::Value> = match rmp_serde::from_read(zstd::Decoder::new(f1).unwrap()) {
        Ok(v) => v,
        Err(e) => {
            errors.push(format!("[{}] Failed to decode 1grams: {}", id, e));
            return errors;
        }
    };

    let mut raw_char_freqs = vec![0u32; 65536];
    for item in raw_1grams {
        if let Some(c) = item["char"].as_str().and_then(resolve_token) {
            let c_val = c as u32;
            if c_val < 65536 {
                let freq = item["freq"].as_u64().unwrap_or(0) as u32;
                raw_char_freqs[c_val as usize] += freq;
            }
            // Ignore chars > 65535 as they cannot be represented in the current Corpus model
        }
    }

    // 3. Compare 1-grams
    for i in 0..65536 {
        // Skip synthetic injection targets for exact match check
        if i == '\n' as usize || i == '\x08' as usize { continue; }
        
        let sys = system_corpus.char_freqs[i];
        let raw = raw_char_freqs[i];
        
        if sys != raw {
            errors.push(format!("[{}] Char mismatch at {} ('{}'): System={}, Raw={}", 
                id, i, std::char::from_u32(i as u32).unwrap_or('?'), sys, raw));
        }
    }

    errors
}

#[tokio::test]
async fn test_all_corpora_completeness() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir.parent().unwrap().parent().unwrap().join("data");
    let corpora_root = root.join("system/corpora");

    let mut all_errors = Vec::new();
    let mut checked_count = 0;

    let walker = WalkDir::new(&corpora_root).min_depth(2).max_depth(3);
    
    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        if entry.file_name().to_str() == Some("1grams.mpk.zst") {
            let parent = entry.path().parent().unwrap();
            let id = parent.strip_prefix(&corpora_root).unwrap().to_string_lossy().replace('\\', "/");
            
            println!("Checking: {}", id);
            let errs = validate_corpus(&root, &id).await;
            if !errs.is_empty() {
                all_errors.extend(errs);
            }
            checked_count += 1;
        }
    }

    println!("\n--- VALIDATION SUMMARY ---");
    println!("Checked {} corpora.", checked_count);
    
    if !all_errors.is_empty() {
        println!("{} Failures Found:", all_errors.len());
        // Group errors by corpus ID for readability
        let mut current_id = "";
        for e in &all_errors {
            let id_end = e.find(']').unwrap_or(0);
            let id = &e[1..id_end];
            if id != current_id {
                println!("\n--- {} ---", id);
                current_id = id;
            }
            println!("  {}", &e[id_end+2..]);
        }
        panic!("Corpus validation failed.");
    } else {
        println!("✅ All corpora validated successfully.");
    }
}

#[tokio::test]
async fn test_inspect_corpus_content() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir.parent().unwrap().parent().unwrap().join("data");
    let corpora_root = root.join("system/corpora");

    let walker = WalkDir::new(&corpora_root).min_depth(2).max_depth(3);
    
    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        if entry.file_name().to_str() == Some("1grams.mpk.zst") {
            let parent = entry.path().parent().unwrap();
            let id = parent.strip_prefix(&corpora_root).unwrap().to_string_lossy().replace('\\', "/");
            
            println!("\n=== CORPUS: {} ===", id);
            
            let f = File::open(entry.path()).unwrap();
            let raw: Vec<serde_json::Value> = rmp_serde::from_read(zstd::Decoder::new(f).unwrap()).unwrap();
            
            let mut chars = Vec::new();
            for item in raw {
                if let Some(c) = item["char"].as_str().and_then(resolve_token) {
                    chars.push(c);
                }
            }
            chars.sort();
            chars.dedup();

            let mut count = 0;
            for c in chars {
                // Print printable chars directly, others as hex code
                if c.is_control() {
                    print!("[{:02X}] ", c as u32);
                } else {
                    print!("{} ", c);
                }
                count += 1;
                if count % 20 == 0 {
                    println!();
                }
            }
            println!();
        }
    }
}
