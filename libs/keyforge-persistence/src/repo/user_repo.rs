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

use crate::error::{PersistenceError, PersistenceResult};
use fs2::FileExt;
use keyforge_infra::error::{InfraError, InfraResult};
use keyforge_infra::fs::io::atomic_write;
use keyforge_infra::util::common::sanitize_filename;
use keyforge_model::geometry::KeyboardDefinition;
use keyforge_protocol::{BiometricSample, UserStatsStore};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// A persistent store for user-created layouts, organized by keyboard ID.
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct UserLayoutStore {
    pub layouts: HashMap<String, HashMap<String, String>>,
}

/// Common interface for user data management across different backends.
#[async_trait::async_trait]
pub trait UserRepository: Send + Sync {
    /// Retrieves a user profile by ID.
    ///
    /// # Errors
    /// Returns `PersistenceError` if the backend operation fails.
    async fn get_profile(
        &self,
        user_id: Uuid,
    ) -> PersistenceResult<Option<keyforge_model::user::UserProfile>>;

    /// Saves or updates a user profile.
    ///
    /// # Errors
    /// Returns `PersistenceError` if the backend operation fails.
    async fn save_profile(
        &self,
        profile: &keyforge_model::user::UserProfile,
    ) -> PersistenceResult<()>;
}

/// A repository for managing user-specific persistent data on the local file system.
#[derive(Debug)]
pub struct UserRepo {
    root: PathBuf,
}

#[async_trait::async_trait]
impl UserRepository for UserRepo {
    /// # Errors
    /// Always returns an error as this is not yet implemented for the filesystem backend.
    async fn get_profile(
        &self,
        _user_id: Uuid,
    ) -> PersistenceResult<Option<keyforge_model::user::UserProfile>> {
        // Legacy filesystem-based profiles are handled differently for now.
        // This is a bridge for local mode.
        Err(PersistenceError::Adapter(
            "Filesystem user profiles not yet fully implemented via trait".into(),
        ))
    }

    /// # Errors
    /// Always returns an error as this is not yet implemented for the filesystem backend.
    async fn save_profile(
        &self,
        _profile: &keyforge_model::user::UserProfile,
    ) -> PersistenceResult<()> {
        Err(PersistenceError::Adapter(
            "Filesystem user profiles not yet fully implemented via trait".into(),
        ))
    }
}

/// `PostgreSQL` implementation of the user repository.
#[derive(Debug, Clone)]
pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    /// Creates a new `PgUserRepository`.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UserRepository for PgUserRepository {
    /// # Errors
    /// Returns `PersistenceError::Database` if the database query fails.
    async fn get_profile(
        &self,
        user_id: Uuid,
    ) -> PersistenceResult<Option<keyforge_model::user::UserProfile>> {
        let row = sqlx::query!(
            r#"
            SELECT id, name, biometric_status, preferences
            FROM user_profiles
            WHERE id = $1
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| PersistenceError::Database(e.to_string()))?;

        if let Some(r) = row {
            let preferences: keyforge_model::user::UserPreferences =
                serde_json::from_value(r.preferences)?;
            let biometric_status = match r.biometric_status.as_str() {
                "collecting" => keyforge_model::user::UserBiometricStatus::Collecting,
                "ready" => keyforge_model::user::UserBiometricStatus::Ready,
                "active" => keyforge_model::user::UserBiometricStatus::Active,
                _ => keyforge_model::user::UserBiometricStatus::Empty,
            };

            Ok(Some(keyforge_model::user::UserProfile {
                id: r.id.into(),
                name: r.name,
                preferences,
                biometric_status,
            }))
        } else {
            Ok(None)
        }
    }

    /// # Errors
    /// Returns `PersistenceError::Database` if the insertion fails.
    async fn save_profile(
        &self,
        profile: &keyforge_model::user::UserProfile,
    ) -> PersistenceResult<()> {
        let prefs = serde_json::to_value(&profile.preferences)?;
        let status = format!("{:?}", profile.biometric_status).to_lowercase();

        sqlx::query!(
            r#"
            INSERT INTO user_profiles (id, name, biometric_status, preferences)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (id) DO UPDATE SET
                name = EXCLUDED.name,
                biometric_status = EXCLUDED.biometric_status,
                preferences = EXCLUDED.preferences,
                updated_at = CURRENT_TIMESTAMP
            "#,
            profile.id.as_uuid(),
            profile.name,
            status,
            prefs
        )
        .execute(&self.pool)
        .await
        .map_err(|e| PersistenceError::Database(e.to_string()))?;

        Ok(())
    }
}

impl UserRepo {
    /// Creates a new `UserRepo` instance using the specified root directory as the data store.
    #[must_use]
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    // --- USER LAYOUTS ---

    /// Checks if the legacy monolithic `user_layouts.json` file exists.
    /// If so, it migrates the data to individual files and deletes the legacy file.
    fn migrate_legacy_store_if_needed(&self) -> InfraResult<()> {
        let legacy_path = self.root.join("user/user_layouts.json");
        if !legacy_path.exists() {
            return Ok(());
        }

        tracing::info!("Migrating legacy user layouts to individual files...");

        if let Ok(content) = fs::read_to_string(&legacy_path) {
            if let Ok(store) = serde_json::from_str::<UserLayoutStore>(&content) {
                for (kb_id, layouts) in store.layouts {
                    for (name, layout_data) in layouts {
                        self.save_layout_internal(&kb_id, &name, &layout_data)?;
                    }
                }
            }
        }

        // Rename legacy file to .bak instead of deleting immediately, for safety
        let backup_path = self.root.join("user/user_layouts.json.bak");
        fs::rename(legacy_path, backup_path).map_err(InfraError::Io)?;

        Ok(())
    }

    fn get_layout_dir(&self, kb_id: &str) -> PathBuf {
        self.root.join("user/layouts").join(kb_id)
    }

    fn save_layout_internal(&self, kb_id: &str, name: &str, layout: &str) -> InfraResult<()> {
        let dir = self.get_layout_dir(kb_id);
        if !dir.exists() {
            fs::create_dir_all(&dir).map_err(InfraError::Io)?;
        }

        let safe_name = sanitize_filename(name);
        let path = dir.join(format!("{safe_name}.json"));

        let data = serde_json::json!({
            "name": name,
            "layout": layout,
            "updated_at": SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
        });

        let json = serde_json::to_string_pretty(&data).map_err(InfraError::Serde)?;
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
        self.migrate_legacy_store_if_needed()?;
        self.save_layout_internal(kb_id, name, layout)
    }

    /// Deletes a previously saved user layout.
    ///
    /// # Errors
    ///
    /// Returns `InfraError` if the layout cannot be deleted.
    pub fn delete_layout(&self, kb_id: &str, name: &str) -> InfraResult<()> {
        self.migrate_legacy_store_if_needed()?;

        let dir = self.get_layout_dir(kb_id);
        let safe_name = sanitize_filename(name);
        let path = dir.join(format!("{safe_name}.json"));

        if path.exists() {
            fs::remove_file(path).map_err(InfraError::Io)?;
        }
        Ok(())
    }

    /// Returns all saved layouts for a specific keyboard.
    #[must_use]
    pub fn get_layouts(&self, kb_id: &str) -> HashMap<String, String> {
        let _ = self.migrate_legacy_store_if_needed();

        let mut layouts = HashMap::new();
        let dir = self.get_layout_dir(kb_id);

        if !dir.exists() {
            return layouts;
        }

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "json") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                            if let (Some(name), Some(layout)) = (
                                json.get("name").and_then(|v| v.as_str()),
                                json.get("layout").and_then(|v| v.as_str()),
                            ) {
                                layouts.insert(name.to_string(), layout.to_string());
                            }
                        }
                    }
                }
            }
        }
        layouts
    }

    // --- BIOMETRICS (JSONL) ---

    fn load_stats_store(&self) -> UserStatsStore {
        let mut store = UserStatsStore::default();
        let _ = self.load_stats_streaming(|sample| {
            if store.biometrics.len() < keyforge_protocol::constants::MAX_BIOMETRIC_SAMPLES {
                store.biometrics.push(sample);
            }
        });
        store.total_keystrokes = store.biometrics.len() as u64;
        store.sessions = 1;
        store
    }

    /// Iterates through stored biometric samples and applies the provided function.
    ///
    /// # Errors
    /// Returns `InfraError` if the stats file cannot be read.
    pub fn load_stats_streaming<F>(&self, mut f: F) -> InfraResult<usize>
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
        if samples.is_empty() {
            return Ok("No samples to record.".to_string());
        }

        // 1. Prepare buffer in memory
        let mut buffer = Vec::new();
        for s in samples {
            serde_json::to_writer(&mut buffer, &s).map_err(InfraError::Serde)?;
            buffer.push(b'\n');
        }

        let path = self.root.join("user/user_stats.jsonl");
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(InfraError::Io)?;
        }

        // 2. Critical Section: Lock and Append
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(InfraError::Io)?;

        file.lock_exclusive().map_err(InfraError::Io)?;
        file.write_all(&buffer).map_err(InfraError::Io)?;
        // Auto-unlock on drop

        #[allow(clippy::naive_bytecount)]
        Ok(format!(
            "Appended {} samples to log.",
            buffer.iter().filter(|&&b| b == b'\n').count()
        ))
    }

    /// Retrieves all accumulated biometric samples.
    #[must_use]
    pub fn get_biometrics(&self) -> Vec<BiometricSample> {
        self.load_stats_store().biometrics.0
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

    /// Saves a cost model to the user's personal profile.
    ///
    /// # Errors
    /// Returns `InfraError` if saving fails.
    pub fn save_personal_cost_model(&self, model: &keyforge_model::CostModel) -> InfraResult<()> {
        let output_path = self.root.join("user/personal_cost.json");
        let json = serde_json::to_string_pretty(model).map_err(InfraError::Serde)?;
        atomic_write(output_path, json)?;
        Ok(())
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
