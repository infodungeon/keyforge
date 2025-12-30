use crate::error::{InfraError, InfraResult};
use crate::fs::io::atomic_write;
use crate::util::common::{generate_cost_profile, sanitize_filename};
use keyforge_model::geometry::KeyboardDefinition;
use keyforge_protocol::{BiometricSample, UserStatsStore};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Default)]
pub struct UserLayoutStore {
    layouts: HashMap<String, HashMap<String, String>>,
}

pub struct UserRepo {
    root: PathBuf,
}

impl UserRepo {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    // --- USER LAYOUTS ---

    fn load_layout_store(&self) -> UserLayoutStore {
        let path = self.root.join("user/user_layouts.json");
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(store) = serde_json::from_str(&content) {
                    return store;
                }
            }
        }
        UserLayoutStore::default()
    }

    fn save_layout_store(&self, store: &UserLayoutStore) -> InfraResult<()> {
        let path = self.root.join("user/user_layouts.json");
        let json = serde_json::to_string_pretty(store).map_err(InfraError::Serde)?;
        atomic_write(path, json)?;
        Ok(())
    }

    pub fn save_layout(&self, kb_id: &str, name: &str, layout: &str) -> InfraResult<()> {
        let mut store = self.load_layout_store();
        let kb_entry = store.layouts.entry(kb_id.to_string()).or_default();
        kb_entry.insert(name.to_string(), layout.to_string());
        self.save_layout_store(&store)
    }

    pub fn delete_layout(&self, kb_id: &str, name: &str) -> InfraResult<()> {
        let mut store = self.load_layout_store();
        if let Some(kb_layouts) = store.layouts.get_mut(kb_id) {
            kb_layouts.remove(name);
            self.save_layout_store(&store)?;
        }
        Ok(())
    }

    pub fn get_layouts(&self, kb_id: &str) -> HashMap<String, String> {
        let store = self.load_layout_store();
        store.layouts.get(kb_id).cloned().unwrap_or_default()
    }

    // --- BIOMETRICS (JSONL) ---

    fn load_stats_store(&self) -> UserStatsStore {
        let path = self.root.join("user/user_stats.jsonl");
        let mut store = UserStatsStore::default();

        if path.exists() {
            if let Ok(file) = fs::File::open(&path) {
                let reader = BufReader::new(file);
                for line in reader.lines().map_while(Result::ok) {
                    if let Ok(sample) = serde_json::from_str::<BiometricSample>(&line) {
                        store.biometrics.push(sample);
                    }
                }
            }
        }
        store.total_keystrokes = store.biometrics.len() as u64;
        store.sessions = 1;
        store
    }

    pub fn record_biometrics(&self, samples: Vec<BiometricSample>) -> InfraResult<String> {
        let path = self.root.join("user/user_stats.jsonl");

        // Ensure parent dir exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(InfraError::Io)?;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(InfraError::Io)?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut count = 0;
        for mut s in samples {
            if s.timestamp == 0 {
                s.timestamp = now;
            }
            let json = serde_json::to_string(&s).map_err(InfraError::Serde)?;
            writeln!(file, "{}", json).map_err(InfraError::Io)?;
            count += 1;
        }

        Ok(format!("Appended {} samples to log.", count))
    }

    pub fn get_biometrics(&self) -> Vec<BiometricSample> {
        self.load_stats_store().biometrics
    }

    pub fn reset_biometrics(&self) -> InfraResult<()> {
        let path = self.root.join("user/user_stats.jsonl");
        if path.exists() {
            fs::remove_file(path).map_err(InfraError::Io)?;
        }
        Ok(())
    }

    pub fn generate_profile(&self) -> InfraResult<String> {
        let store = self.load_stats_store();
        if store.biometrics.len() < 300 {
            return Err(InfraError::Config(format!(
                "Insufficient data. {}/300 samples collected.",
                store.biometrics.len()
            )));
        }

        let profile_content = generate_cost_profile(&store);
        let output_path = self.root.join("user/personal_cost.json");
        atomic_write(output_path, profile_content)?;

        Ok(format!(
            "Profile generated from {} samples.",
            store.biometrics.len()
        ))
    }

    // --- KEYBOARDS ---

    pub fn save_keyboard_definition(
        &self,
        filename: &str,
        def: &KeyboardDefinition,
    ) -> InfraResult<()> {
        let kb_dir = self.root.join("user/keyboards");
        if !kb_dir.exists() {
            fs::create_dir_all(&kb_dir).map_err(InfraError::Io)?;
        }

        let safe_name = sanitize_filename(filename);
        let path = kb_dir.join(format!("{}.json", safe_name));
        let json = serde_json::to_string_pretty(def).map_err(InfraError::Serde)?;

        atomic_write(path, json)?;
        Ok(())
    }
}
