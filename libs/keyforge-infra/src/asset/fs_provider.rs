use keyforge_model::error::ForgeError;
use keyforge_model::loader::{AssetLoader, LoaderResult, RawCostData};
use keyforge_model::Corpus;
use keyforge_protocol::config::CorpusSource;
use keyforge_protocol::constants::MAX_INPUT_FILE_SIZE;
use keyforge_protocol::geometry::KeyboardDefinition;
use keyforge_protocol::keycodes::KeycodeRegistry;
use sha2::Digest;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct FsProvider {
    pub root: PathBuf,
}

impl FsProvider {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    fn check_size(&self, path: &Path) -> LoaderResult<()> {
        let meta = std::fs::metadata(path)?;
        if meta.len() > MAX_INPUT_FILE_SIZE {
            return Err(ForgeError::InvalidData(format!(
                "File {:?} exceeds size limit of {} bytes",
                path, MAX_INPUT_FILE_SIZE
            )));
        }
        Ok(())
    }

    fn load_binary<T: serde::de::DeserializeOwned>(&self, path: &Path) -> LoaderResult<T> {
        self.check_size(path)?;
        let file = File::open(path)?;
        let decoder =
            zstd::Decoder::new(file).map_err(|e| ForgeError::Internal(e.to_string()))?;
        rmp_serde::from_read(decoder).map_err(|e| ForgeError::Internal(e.to_string()))
    }

    fn load_json<T: serde::de::DeserializeOwned>(&self, path: &Path) -> LoaderResult<T> {
        self.check_size(path)?;
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).map_err(ForgeError::Serde)
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

    pub fn get_corpus_hash(&self, id: &str) -> LoaderResult<String> {
        let mut hasher = sha2::Sha256::new();
        let files = ["1grams", "2grams", "3grams", "words"];
        let is_system = self.root.join("system/corpora").join(id).exists();
        let base = if is_system {
            self.root.join("system/corpora").join(id)
        } else {
            self.root.join("user/corpora").join(id)
        };
        let ext = if is_system { "mpk.zst" } else { "json" };
        for f in files {
            let path = base.join(format!("{}.{}", f, ext));
            if path.exists() {
                hasher.update(&std::fs::read(&path)?);
            }
        }
        Ok(hex::encode(hasher.finalize()))
    }
}

impl AssetLoader for FsProvider {
    fn load_keyboard(&self, name: &str) -> LoaderResult<KeyboardDefinition> {
        let stem = name.strip_suffix(".json").unwrap_or(name);
        if let Some(p) = self.resolve_system_path("keyboards", stem) {
            return self.load_binary(&p);
        }
        if let Some(p) = self.resolve_user_path("keyboards", stem) {
            return self.load_json(&p);
        }
        Err(ForgeError::NotFound(name.to_string()))
    }

    fn load_corpus(&self, sources: &[CorpusSource]) -> LoaderResult<Corpus> {
        let mut corpus = Corpus::default();
        for src in sources {
            let is_system = self.root.join("system/corpora").join(&src.id).exists();
            let base = if is_system {
                self.root.join("system/corpora").join(&src.id)
            } else {
                self.root.join("user/corpora").join(&src.id)
            };
            let ext = if is_system { "mpk.zst" } else { "json" };

            let load_part = |stem: &str| -> LoaderResult<Vec<serde_json::Value>> {
                let p = base.join(format!("{}.{}", stem, ext));
                if !p.exists() {
                    return Ok(vec![]);
                }
                if is_system {
                    self.load_binary(&p)
                } else {
                    self.load_json(&p)
                }
            };

            for e in load_part("1grams")? {
                if let Some(c) = e["char"].as_str().and_then(|s| s.chars().next()) {
                    if (c as usize) < 256 {
                        corpus.char_freqs[c as usize] +=
                            (e["freq"].as_u64().unwrap_or(0) as f32 * src.weight).round() as u32;
                    }
                }
            }
            for e in load_part("2grams")? {
                let c1 = e["char1"]
                    .as_str()
                    .and_then(|s| s.chars().next())
                    .unwrap_or('\0') as u16;
                let c2 = e["char2"]
                    .as_str()
                    .and_then(|s| s.chars().next())
                    .unwrap_or('\0') as u16;
                corpus.bigrams.push((
                    c1,
                    c2,
                    (e["freq"].as_u64().unwrap_or(0) as f32 * src.weight).round() as u32,
                ));
            }
            for e in load_part("3grams")? {
                let c1 = e["char1"]
                    .as_str()
                    .and_then(|s| s.chars().next())
                    .unwrap_or('\0') as u16;
                let c2 = e["char2"]
                    .as_str()
                    .and_then(|s| s.chars().next())
                    .unwrap_or('\0') as u16;
                let c3 = e["char3"]
                    .as_str()
                    .and_then(|s| s.chars().next())
                    .unwrap_or('\0') as u16;
                corpus.trigrams.push((
                    c1,
                    c2,
                    c3,
                    (e["freq"].as_u64().unwrap_or(0) as f32 * src.weight).round() as u32,
                ));
            }
            for e in load_part("words")? {
                if let Some(w) = e["word"].as_str() {
                    corpus.words.push((
                        w.to_string(),
                        (e["freq"].as_u64().unwrap_or(0) as f32 * src.weight).round() as u32,
                    ));
                }
            }
        }
        Ok(corpus)
    }

    fn load_cost_matrix(&self, filename: &str) -> LoaderResult<RawCostData> {
        let stem = filename.strip_suffix(".json").unwrap_or(filename);
        if let Some(p) = self.resolve_system_path("weights", stem) {
            return self.load_binary(&p);
        }
        if let Some(p) = self.resolve_user_path("weights", stem) {
            #[derive(serde::Deserialize)]
            struct CostEntry {
                from_key: String,
                to_key: String,
                cost_ms: f32,
            }
            #[derive(serde::Deserialize)]
            #[serde(untagged)]
            enum Format {
                Wrapped { entries: Vec<CostEntry> },
                Direct(Vec<CostEntry>),
            }
            let format: Format = self.load_json(&p)?;
            let entries = match format {
                Format::Wrapped { entries } => entries,
                Format::Direct(v) => v,
            };
            return Ok(RawCostData {
                entries: entries
                    .into_iter()
                    .map(|e| (e.from_key, e.to_key, e.cost_ms))
                    .collect(),
            });
        }
        Err(ForgeError::NotFound(filename.to_string()))
    }

    fn load_keycodes(&self, filename: &str) -> LoaderResult<KeycodeRegistry> {
        let stem = filename.strip_suffix(".json").unwrap_or(filename);
        if let Some(p) = self.resolve_system_path("config", stem) {
            let defs = self.load_binary(&p)?;
            return Ok(KeycodeRegistry::new(defs));
        }
        let p = self
            .resolve_user_path("config", stem)
            .ok_or(ForgeError::NotFound(filename.to_string()))?;
        let defs = self.load_json(&p)?;
        Ok(KeycodeRegistry::new(defs))
    }
}
