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
use keyforge_model::error::ForgeError;
use keyforge_core::loader::{AssetLoader, LoaderResult, RawCostData};
use keyforge_model::Corpus;
use keyforge_model::constants::{STD_CORPUS_ERROR_RATE, STD_CORPUS_BACKSPACE_FACTOR, STD_CORPUS_SENTENCE_RATIO};
use keyforge_model::config::CorpusSource;
use keyforge_model::constants::{MAX_INPUT_FILE_SIZE, CORPUS_TOKEN_MAP};
use keyforge_model::geometry::KeyboardDefinition;
use keyforge_model::keycodes::KeycodeRegistry;
use sha2::Digest;
use keyforge_model::validator::Validator;
use std::fs::File;

use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct FsProvider {
    pub root: PathBuf,
}

impl FsProvider {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    async fn check_size(&self, path: &Path) -> LoaderResult<()> {
        let meta = tokio::fs::metadata(path).await?;
        if meta.len() > MAX_INPUT_FILE_SIZE {
            return Err(ForgeError::InvalidData(format!(
                "File {:?} exceeds size limit of {} bytes",
                path, MAX_INPUT_FILE_SIZE
            )));
        }
        Ok(())
    }

    async fn load_binary<T: serde::de::DeserializeOwned + Send + 'static>(&self, path: &Path) -> LoaderResult<T> {
        self.check_size(path).await?;
        let path = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let file = File::open(&path)?;
            let decoder =
                zstd::Decoder::new(file).map_err(|e| ForgeError::Internal(e.to_string()))?;
            rmp_serde::from_read(decoder).map_err(|e| ForgeError::Internal(e.to_string()))
        })
        .await
        .map_err(|e| ForgeError::Internal(e.to_string()))?
    }

    async fn load_json<T: serde::de::DeserializeOwned + Send + 'static>(&self, path: &Path) -> LoaderResult<T> {
        self.check_size(path).await?;
        let content = tokio::fs::read_to_string(path).await?;
        tokio::task::spawn_blocking(move || {
            serde_json::from_str(&content).map_err(ForgeError::Serde)
        })
        .await
        .map_err(|e| ForgeError::Internal(e.to_string()))?
    }

    fn resolve_system_path(&self, category: &str, stem: &str) -> Option<PathBuf> {
        // Map categories to new structure
        let sub = match category {
            "keyboards" => "keyboards/models",
            "weights" => "weights",
            "config" => "config",
            "keymap_extras" => "keymap_extras",
            _ => category,
        };

        let p = self
            .root
            .join("system")
            .join(sub)
            .join(format!("{}.mpk.zst", stem));

        if p.exists() {
            return Some(p);
        }

        // Fallback for direct mapping if needed (e.g. if we passed "keyboards/models" as category)
        let p_direct = self
            .root
            .join("system")
            .join(category)
            .join(format!("{}.mpk.zst", stem));
        if p_direct.exists() {
            return Some(p_direct);
        }

        None
    }

    fn resolve_user_path(&self, category: &str, stem: &str) -> Option<PathBuf> {
        let p = self
            .root
            .join("user")
            .join(category)
            .join(format!("{}.json", stem));
        p.exists().then_some(p)
    }

    pub async fn get_corpus_hash(&self, id: &str) -> LoaderResult<String> {
        let files = ["1grams", "2grams", "3grams", "words"];
        let is_system = self.root.join("system/corpora").join(id).exists();
        let base = if is_system {
            self.root.join("system/corpora").join(id)
        } else {
            self.root.join("user/corpora").join(id)
        };
        let ext = if is_system { "mpk.zst" } else { "json" };
        
        let mut hasher = sha2::Sha256::new();
        for f in files {
            let path = base.join(format!("{}.{}", f, ext));
            if path.exists() {
                let content = tokio::fs::read(&path).await?;
                hasher.update(&content);
            }
        }
        Ok(hex::encode(hasher.finalize()))
    }
}

#[derive(serde::Deserialize)]
struct CostEntry {
    #[serde(alias = "from")]
    from_key: String,
    #[serde(alias = "to")]
    to_key: String,
    #[serde(alias = "cost")]
    cost_ms: f32,
}

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum CostFormat {
    Wrapped { entries: Vec<CostEntry> },
    Direct(Vec<CostEntry>),
}

/// Resolves a corpus token string to a character.
/// Handles special tokens like "SPACE", "ENTER", etc. using the shared map.
/// Normalizes single characters to Lowercase to match KeycodeRegistry normalization.
fn resolve_corpus_char(token: &str) -> Option<char> {
    for (key, val) in CORPUS_TOKEN_MAP {
        if token == *key {
            return Some(*val);
        }
    }
    // Fallback: If it's a single char, use it, normalizing to lowercase
    if token.chars().count() == 1 {
        token.chars().next().map(|c| c.to_ascii_lowercase())
    } else {
        None
    }
}

/// Injects synthetic data (Enter, Backspace) for standard prose corpora.
fn inject_synthetic_data(corpus: &mut Corpus, is_std: bool) {
    if !is_std { return; }

    // 1. Calculate Totals
    let total_chars: u64 = corpus.char_freqs.iter().sum();
    let sentence_count: u64 = 
        corpus.char_freqs['.' as usize] + 
        corpus.char_freqs['?' as usize] + 
        corpus.char_freqs['!' as usize];

    if total_chars == 0 { return; }

    // 2. Calculate Injection Volumes
    let enter_count = (sentence_count as f32 / STD_CORPUS_SENTENCE_RATIO).round() as u64;
    let bksp_count = (total_chars as f32 * STD_CORPUS_ERROR_RATE * STD_CORPUS_BACKSPACE_FACTOR).round() as u64;

    // 3. Inject 1-grams
    corpus.char_freqs['\n' as usize] += enter_count;
    corpus.char_freqs['\x08' as usize] += bksp_count;

    // 4. Inject 2-grams (Bigrams)
    // Strategy: Distribute transitions proportionally to character frequency
    
    // A. Backspace Injection (Random Error Model)
    // X -> BKSP (Typo) and BKSP -> X (Correction)
    // We distribute the total backspaces across all existing characters based on their frequency
    if bksp_count > 0 {
        let mut new_bigrams = Vec::new();
        for (char_code, &freq) in corpus.char_freqs.iter().enumerate() {
            if freq > 0 && char_code != '\x08' as usize && char_code != '\n' as usize {
                let ratio = freq as f32 / total_chars as f32;
                let share = (bksp_count as f32 * ratio).round() as u32;
                if share > 0 {
                    // Char -> Backspace
                    new_bigrams.push((char_code as u16, '\x08' as u16, share));
                    // Backspace -> Char
                    new_bigrams.push(('\x08' as u16, char_code as u16, share));
                }
            }
        }
        corpus.bigrams.extend(new_bigrams);
    }

    // B. Enter Injection (Sentence Boundary Model)
    // Punctuation -> Enter
    if enter_count > 0 {
        let puncts = ['.', '?', '!'];
        let total_punct = sentence_count.max(1);
        
        for p in puncts {
            let p_freq = corpus.char_freqs[p as usize];
            if p_freq > 0 {
                let ratio = p_freq as f32 / total_punct as f32;
                let share = (enter_count as f32 * ratio).round() as u32;
                if share > 0 {
                    corpus.bigrams.push((p as u16, '\n' as u16, share));
                }
            }
        }
    }

    // 5. Inject 3-grams (Trigrams)
    // Strategy: Distribute transitions proportionally to existing Bigrams
    
    // A. Backspace Trigrams
    // (A, B, BKSP) -> User typed A, B, then deleted B
    if bksp_count > 0 {
        let total_bigrams: u64 = corpus.bigrams.iter().map(|(_, _, f)| *f as u64).sum();
        if total_bigrams > 0 {
            let mut new_trigrams = Vec::new();
            for (a, b, freq) in &corpus.bigrams {
                // Skip if already involves special keys to avoid recursion/noise
                if *a == '\x08' as u16 || *b == '\x08' as u16 || *a == '\n' as u16 || *b == '\n' as u16 {
                    continue;
                }
                
                let ratio = *freq as f32 / total_bigrams as f32;
                let share = (bksp_count as f32 * ratio).round() as u32;
                
                if share > 0 {
                    // (A, B, BKSP)
                    new_trigrams.push((*a, *b, '\x08' as u16, share));
                }
            }
            corpus.trigrams.extend(new_trigrams);
        }
    }

    // B. Enter Trigrams
    // (A, Punct, Enter) -> End of sentence
    if enter_count > 0 {
        let puncts = ['.', '?', '!'];
        let mut new_trigrams = Vec::new();
        
        // Filter bigrams ending in punctuation
        let punct_bigrams: Vec<_> = corpus.bigrams.iter()
            .filter(|(_, b, _)| puncts.contains(&(*b as u8 as char)))
            .collect();
            
        let total_punct_bigrams: u64 = punct_bigrams.iter().map(|(_, _, f)| *f as u64).sum();
        
        if total_punct_bigrams > 0 {
            for (a, b, freq) in punct_bigrams {
                let ratio = *freq as f32 / total_punct_bigrams as f32;
                let share = (enter_count as f32 * ratio).round() as u32;
                
                if share > 0 {
                    // (A, Punct, Enter)
                    new_trigrams.push((*a, *b, '\n' as u16, share));
                }
            }
            corpus.trigrams.extend(new_trigrams);
        }
    }

    // SORTING: Essential for Physics Engine lookup tables
    corpus.bigrams.sort_unstable_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
    corpus.trigrams.sort_unstable_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));
}

#[async_trait::async_trait]
impl AssetLoader for FsProvider {
    async fn load_keyboard(&self, name: &str) -> LoaderResult<KeyboardDefinition> {
        let stem = name.strip_suffix(".json").unwrap_or(name);
        if let Some(p) = self.resolve_system_path("keyboards", stem) {
            let kb: KeyboardDefinition = self.load_binary(&p).await?;
            kb.validate().map_err(|e| ForgeError::InvalidData(format!("Invalid system keyboard '{}': {}", name, e)))?;
            return Ok(kb);
        }
        if let Some(p) = self.resolve_user_path("keyboards", stem) {
            let kb: KeyboardDefinition = self.load_json(&p).await?;
            kb.validate().map_err(|e| ForgeError::InvalidData(format!("Invalid user keyboard '{}': {}", name, e)))?;
            return Ok(kb);
        }
        Err(ForgeError::NotFound(name.to_string()))
    }

    async fn load_corpus(&self, sources: &[CorpusSource]) -> LoaderResult<Corpus> {
        let mut corpus = Corpus::default();
        for src in sources {
            let is_system = self.root.join("system/corpora").join(&src.id).exists();
            let base = if is_system {
                self.root.join("system/corpora").join(&src.id)
            } else {
                self.root.join("user/corpora").join(&src.id)
            };
            let ext = if is_system { "mpk.zst" } else { "json" };

            let mut segments = Vec::new();
            for stem in ["1grams", "2grams", "3grams", "words"] {
                let p = base.join(format!("{}.{}", stem, ext));
                if p.exists() {
                    let part: Vec<serde_json::Value> = if is_system {
                        self.load_binary(&p).await?
                    } else {
                        self.load_json(&p).await?
                    };
                    segments.push((stem, part));
                }
            }

            for (stem, part) in segments {
                match stem {
                    "1grams" => {
                        for e in part {
                            if let Some(c) = e["char"].as_str().and_then(resolve_corpus_char) {
                                if (c as usize) < 65536 {
                                    corpus.char_freqs[c as usize] +=
                                        (e["freq"].as_u64().unwrap_or(0) as f32 * src.weight).round() as u64;
                                }
                            }
                        }
                    }
                    "2grams" => {
                        for e in part {
                            let c1 = e["char1"].as_str().and_then(resolve_corpus_char).unwrap_or('\0') as u16;
                            let c2 = e["char2"].as_str().and_then(resolve_corpus_char).unwrap_or('\0') as u16;
                            corpus.bigrams.push((c1, c2, (e["freq"].as_u64().unwrap_or(0) as f32 * src.weight).round() as u32));
                        }
                    }
                    "3grams" => {
                        for e in part {
                            let c1 = e["char1"].as_str().and_then(resolve_corpus_char).unwrap_or('\0') as u16;
                            let c2 = e["char2"].as_str().and_then(resolve_corpus_char).unwrap_or('\0') as u16;
                            let c3 = e["char3"].as_str().and_then(resolve_corpus_char).unwrap_or('\0') as u16;
                            corpus.trigrams.push((c1, c2, c3, (e["freq"].as_u64().unwrap_or(0) as f32 * src.weight).round() as u32));
                        }
                    }
                    "words" => {
                        for e in part {
                            if let Some(w) = e["word"].as_str() {
                                corpus.words.push((w.to_string(), (e["freq"].as_u64().unwrap_or(0) as f32 * src.weight).round() as u32));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        let is_std = sources.iter().any(|s| s.id.contains("_std"));
        inject_synthetic_data(&mut corpus, is_std);

        corpus.validate().map_err(|e| ForgeError::InvalidData(format!("Invalid corpus: {}", e)))?;
        Ok(corpus)
    }

    async fn load_cost_matrix(&self, filename: &str) -> LoaderResult<RawCostData> {
        let stem = filename.strip_suffix(".json").unwrap_or(filename);
        if let Some(p) = self.resolve_system_path("weights", stem) {
            return self.load_binary(&p).await;
        }
        if let Some(p) = self.resolve_user_path("weights", stem) {
            let format: CostFormat = self.load_json(&p).await?;
            let entries = match format {
                CostFormat::Wrapped { entries } => entries,
                CostFormat::Direct(v) => v,
            };
            return Ok(RawCostData {
                entries: entries
                    .into_iter()
                    .map(|e| keyforge_core::loader::CostEntry {
                        from: e.from_key,
                        to: e.to_key,
                        cost: e.cost_ms,
                    })
                    .collect(),
            });
        }
        Err(ForgeError::NotFound(filename.to_string()))
    }

    async fn load_keycodes(&self, filename: &str) -> LoaderResult<KeycodeRegistry> {
        let stem = filename.strip_suffix(".json").unwrap_or(filename);
        if let Some(p) = self.resolve_system_path("config", stem) {
            let defs = self.load_binary(&p).await?;
            let reg = KeycodeRegistry::new(defs);
            reg.validate().map_err(|e| ForgeError::InvalidData(format!("Invalid system keycodes: {}", e)))?;
            return Ok(reg);
        }
        let p = self
            .resolve_user_path("config", stem)
            .ok_or(ForgeError::NotFound(filename.to_string()))?;
        let defs = self.load_json(&p).await?;
        let reg = KeycodeRegistry::new(defs);
        reg.validate().map_err(|e| ForgeError::InvalidData(format!("Invalid user keycodes: {}", e)))?;
        Ok(reg)
    }
}