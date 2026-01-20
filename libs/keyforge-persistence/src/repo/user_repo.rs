// libs/keyforge-persistence/src/repo/user_repo.rs

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

use fs2::FileExt;
use keyforge_infra::error::{InfraError, InfraResult};
use keyforge_infra::fs::io::atomic_write;
use keyforge_infra::util::common::{sanitize_filename, StreamingProfileBuilder};
use keyforge_model::constants::MIN_BIOMETRIC_SAMPLES;
use keyforge_model::geometry::KeyboardDefinition;
use keyforge_protocol::{BiometricSample, UserStatsStore};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// A persistent store for user-created layouts, organized by keyboard ID.
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct UserLayoutStore {
    layouts: HashMap<String, HashMap<String, String>>,
}

/// A repository for managing user-specific persistent data on the local file system.
///
/// This handles the storage of custom layouts, biometric samples for personalization,
/// and custom keyboard definitions.
#[derive(Debug)]
pub struct UserRepo {
    root: PathBuf,
}

impl UserRepo {
    /// Creates a new `UserRepo` instance using the specified root directory as the data store.
    #[must_use]
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

    /// Saves a user layout for a specific keyboard.
    ///
    /// # Arguments
    /// * `kb_id` - The unique identifier for the keyboard (e.g., "corne").
    /// * `name` - The descriptive name for the layout (e.g., "My Dvorak").
    /// * `layout` - The string representation of the layout.
    ///
    /// # Errors
    ///
    /// Returns `InfraError` if saving fails.
    pub fn save_layout(&self, kb_id: &str, name: &str, layout: &str) -> InfraResult<()> {
        let mut store = self.load_layout_store();
        let kb_entry = store.layouts.entry(kb_id.to_string()).or_default();
        kb_entry.insert(name.to_string(), layout.to_string());
        self.save_layout_store(&store)
    }

    /// Deletes a previously saved user layout.
    ///
    /// # Errors
    ///
    /// Returns `InfraError` if the layout cannot be deleted.
    pub fn delete_layout(&self, kb_id: &str, name: &str) -> InfraResult<()> {
        let mut store = self.load_layout_store();
        if let Some(kb_layouts) = store.layouts.get_mut(kb_id) {
            kb_layouts.remove(name);
            self.save_layout_store(&store)?;
        }
        Ok(())
    }

    /// Returns all saved layouts for a specific keyboard.
    #[must_use]
    pub fn get_layouts(&self, kb_id: &str) -> HashMap<String, String> {
        let store = self.load_layout_store();
        store.layouts.get(kb_id).cloned().unwrap_or_default()
    }

    // --- BIOMETRICS (JSONL) ---

    fn load_stats_store(&self) -> UserStatsStore {
        let mut store = UserStatsStore::default();
        let _ = self.load_stats_streaming(|sample| {
            if store.biometrics.len() < 100_000 {
                store.biometrics.push(sample);
            }
        });
        store.total_keystrokes = store.biometrics.len() as u64;
        store.sessions = 1;
        store
    }

    fn load_stats_streaming<F>(&self, mut f: F) -> InfraResult<usize>
    where
        F: FnMut(BiometricSample),
    {
        let path = self.root.join("user/user_stats.jsonl");
        let mut count = 0;
        if path.exists() {
            let file = fs::File::open(&path).map_err(InfraError::Io)?;
            let reader = BufReader::new(file);
            for line in reader.lines() {
                let line = line.map_err(InfraError::Io)?;
                if let Ok(sample) = serde_json::from_str::<BiometricSample>(&line) {
                    f(sample);
                    count += 1;
                }
            }
        }
        Ok(count)
    }

    /// Appends biometric samples to the local audit log for future profile generation.
    ///
    /// Returns a message indicating the number of samples recorded.
    ///
    /// # Errors
    ///
    /// Returns `InfraError` if the audit log cannot be updated or locked.
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

        // Acquire exclusive lock before appending
        file.lock_exclusive().map_err(InfraError::Io)?;

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
            writeln!(file, "{json}").map_err(InfraError::Io)?;
            count += 1;
        }

        Ok(format!("Appended {count} samples to log."))
    }

    /// Retrieves all accumulated biometric samples.
    #[must_use]
    pub fn get_biometrics(&self) -> Vec<BiometricSample> {
        self.load_stats_store().biometrics
    }

    /// Deletes all accumulated biometric samples from disk.
    ///
    /// # Errors
    ///
    /// Returns `InfraError::Io` if the file cannot be removed.
    pub fn reset_biometrics(&self) -> InfraResult<()> {
        let path = self.root.join("user/user_stats.jsonl");
        if path.exists() {
            fs::remove_file(path).map_err(InfraError::Io)?;
        }
        Ok(())
    }

    /// Generates a personalized cost profile based on collected biometric data.
    ///
    /// # Errors
    /// Returns an error if there are fewer than `MIN_BIOMETRIC_SAMPLES` collected.
    pub fn generate_profile(&self) -> InfraResult<String> {
        let mut builder = StreamingProfileBuilder::new();
        let count = self.load_stats_streaming(|sample| {
            builder.add_sample(&sample);
        })?;

        if count < MIN_BIOMETRIC_SAMPLES {
            return Err(InfraError::Config(format!(
                "Insufficient data. {count}/{MIN_BIOMETRIC_SAMPLES} samples collected."
            )));
        }

        let profile_content = builder.generate();
        let output_path = self.root.join("user/personal_cost.json");
        atomic_write(output_path, profile_content)?;

        Ok(format!("Profile generated from {count} samples."))
    }

    // --- KEYBOARDS ---

    /// Saves a custom keyboard definition to the user's local inventory.
    ///
    /// # Errors
    ///
    /// Returns `InfraError` if saving fails.
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
        let path = kb_dir.join(format!("{safe_name}.json"));
        let json = serde_json::to_string_pretty(def).map_err(InfraError::Serde)?;

        atomic_write(path, json)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_user_repo_layouts() {
        let dir = tempdir().unwrap();
        let repo = UserRepo::new(dir.path().to_path_buf());
        
        repo.save_layout("kb1", "name1", "layout1").unwrap();
        let layouts = repo.get_layouts("kb1");
        assert_eq!(layouts.get("name1").unwrap(), "layout1");
        
        repo.delete_layout("kb1", "name1").unwrap();
        assert!(repo.get_layouts("kb1").is_empty());
    }

    #[test]
    fn test_user_repo_biometrics() {
        let dir = tempdir().unwrap();
        let repo = UserRepo::new(dir.path().to_path_buf());
        
        let sample = BiometricSample { bigram: "th".into(), ms: 100.0, timestamp: 0 };
        repo.record_biometrics(vec![sample]).unwrap();
        
        let biometrics = repo.get_biometrics();
        assert_eq!(biometrics.len(), 1);
        assert_eq!(biometrics[0].bigram, "th");
        
        repo.reset_biometrics().unwrap();
        assert!(repo.get_biometrics().is_empty());
    }

    #[test]
    fn test_user_repo_profile_generation() {
        let dir = tempdir().unwrap();
        let repo = UserRepo::new(dir.path().to_path_buf());
        
        // 1. Fail: Insufficient data
        assert!(repo.generate_profile().is_err());
        
        // 2. Success: Fill data
        let samples = (0..MIN_BIOMETRIC_SAMPLES)
            .map(|_| BiometricSample { bigram: "th".into(), ms: 100.0, timestamp: 0 })
            .collect();
        repo.record_biometrics(samples).unwrap();
        assert!(repo.generate_profile().is_ok());
    }

    #[test]
    fn test_user_repo_keyboard_definition() {
        let dir = tempdir().unwrap();
        let repo = UserRepo::new(dir.path().to_path_buf());
        
        let def = KeyboardDefinition::default();
        repo.save_keyboard_definition("test_kb", &def).unwrap();
        assert!(dir.path().join("user/keyboards/test_kb.json").exists());
    }

    #[test]
    fn test_user_repo_corruption_handling() {
        let dir = tempdir().unwrap();
        let repo = UserRepo::new(dir.path().to_path_buf());
        let path = dir.path().join("user/user_layouts.json");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        
        // 1. Broken JSON in layout store
        fs::write(&path, "{ invalid json }").unwrap();
        let store = repo.load_layout_store();
        assert!(store.layouts.is_empty(), "Should return default on corruption");

        // 2. Broken line in biometric JSONL
        let stats_path = dir.path().join("user/user_stats.jsonl");
        let line1 = "{\"bigram\":\"th\",\"ms\":100.0,\"timestamp\":123}";
        let line2 = "{\"bigram\":\"he\",\"ms\":150.0,\"timestamp\":124}";
        fs::write(&stats_path, format!("{}\n{{broken line}}\n{}", line1, line2)).unwrap();
        let biometrics = repo.get_biometrics();
        assert_eq!(biometrics.len(), 2, "Should skip broken lines in JSONL");
    }

    #[test]
    fn test_user_repo_save_keyboard_sanitization() {
        let dir = tempdir().unwrap();
        let repo = UserRepo::new(dir.path().to_path_buf());
        let def = KeyboardDefinition::default();
        
        // Use a "dirty" filename
        repo.save_keyboard_definition("../../../etc/passwd", &def).unwrap();
        
        // Verify it was sanitized (actual filename depends on sanitize_filename implementation)
        // Usually it replaces / with _
        let exists = fs::read_dir(dir.path().join("user/keyboards")).unwrap()
            .any(|e| e.unwrap().file_name().to_str().unwrap().contains("passwd"));
        assert!(exists);
    }
}
