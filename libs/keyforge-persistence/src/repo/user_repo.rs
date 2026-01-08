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

use keyforge_infra::error::{InfraError, InfraResult};
use keyforge_infra::fs::io::atomic_write;
use keyforge_infra::util::common::{generate_cost_profile, sanitize_filename};
use keyforge_model::geometry::KeyboardDefinition;
use keyforge_protocol::{BiometricSample, UserStatsStore};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// A persistent store for user-created layouts, organized by keyboard ID.
#[derive(Serialize, Deserialize, Default)]
pub struct UserLayoutStore {
    layouts: HashMap<String, HashMap<String, String>>,
}

/// A repository for managing user-specific persistent data on the local file system.
///
/// This handles the storage of custom layouts, biometric samples for personalization,
/// and custom keyboard definitions.
pub struct UserRepo {
    root: PathBuf,
}

impl UserRepo {
    /// Creates a new `UserRepo` instance using the specified root directory as the data store.
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
    pub fn save_layout(&self, kb_id: &str, name: &str, layout: &str) -> InfraResult<()> {
        let mut store = self.load_layout_store();
        let kb_entry = store.layouts.entry(kb_id.to_string()).or_default();
        kb_entry.insert(name.to_string(), layout.to_string());
        self.save_layout_store(&store)
    }

    /// Deletes a previously saved user layout.
    pub fn delete_layout(&self, kb_id: &str, name: &str) -> InfraResult<()> {
        let mut store = self.load_layout_store();
        if let Some(kb_layouts) = store.layouts.get_mut(kb_id) {
            kb_layouts.remove(name);
            self.save_layout_store(&store)?;
        }
        Ok(())
    }

    /// Returns all saved layouts for a specific keyboard.
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

    /// Appends biometric samples to the local audit log for future profile generation.
    ///
    /// Returns a message indicating the number of samples recorded.
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

    /// Retrieves all accumulated biometric samples.
    pub fn get_biometrics(&self) -> Vec<BiometricSample> {
        self.load_stats_store().biometrics
    }

    /// Deletes all accumulated biometric samples from disk.
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
    /// Returns an error if there are fewer than 300 samples collected.
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

    /// Saves a custom keyboard definition to the user's local inventory.
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
