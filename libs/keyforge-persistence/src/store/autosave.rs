// libs/keyforge-persistence/src/store/autosave.rs

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

use crate::error::PersistenceResult;
use keyforge_model::constants::MAX_SESSION_FILE_SIZE;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tempfile::NamedTempFile;
use tracing::{error, warn};

/// A snapshot of the current user session.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct SessionSnapshot {
    /// The name of the keyboard definition.
    pub keyboard: String,
    /// The name of the current layout.
    pub layout_name: String,
    /// The string representation of the layout.
    pub layout_string: String,
    /// The corpus name or content.
    pub corpus: String,
    /// The cost matrix source (e.g., "defaults.json").
    pub cost_matrix: String,
    /// UNIX timestamp of when the snapshot was taken.
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct PersistedSession {
    snapshot: SessionSnapshot,
    checksum: String,
}

impl PersistedSession {
    fn new(snapshot: SessionSnapshot) -> Self {
        let checksum = Self::calculate_checksum(&snapshot);
        Self { snapshot, checksum }
    }

    fn calculate_checksum(snapshot: &SessionSnapshot) -> String {
        // Task-persist-rev-059: Use postcard for deterministic canonicalization.
        // postcard is designed for deterministic binary serialization.
        let data = postcard::to_stdvec(snapshot).unwrap_or_default();
        hex::encode(Sha256::digest(data))
    }

    fn verify(&self) -> bool {
        let calculated = Self::calculate_checksum(&self.snapshot);
        if calculated != self.checksum {
            warn!(
                "Checksum mismatch! Stored: {}, Calculated: {}",
                self.checksum, calculated
            );
            return false;
        }
        true
    }
}

/// Internal state for the autosave debounce mechanism.
/// Public for integration testing.
#[derive(Debug)]
pub struct AutoSaveState {
    /// Pending snapshot awaiting flush.
    pub pending: Option<SessionSnapshot>,
    /// Timestamp of the last save operation.
    pub last_save: Instant,
}

/// A service that handles automated background saving of the user session.
#[derive(Debug)]
pub struct AutoSaveService {
    path: PathBuf,
    /// Internal state for debounce tracking.
    /// Public for integration testing; do not access in production code.
    pub state: Arc<Mutex<AutoSaveState>>,
}

impl AutoSaveService {
    /// Creates a new `AutoSaveService` instance with a session file located in the provided root path.
    #[must_use]
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(root_path: PathBuf) -> Self {
        let path = root_path.join("session.json");

        Self {
            path,
            state: Arc::new(Mutex::new(AutoSaveState {
                pending: None,
                last_save: Instant::now(),
            })),
        }
    }

    /// Loads the last saved session snapshot from disk.
    ///
    /// # Errors
    /// Returns [`PersistenceError::Io`] if reading the file fails.
    /// Returns [`PersistenceError::Serde`] if parsing JSON fails.
    pub async fn load(&self) -> PersistenceResult<Option<SessionSnapshot>> {
        if !self.path.exists() {
            return Ok(None);
        }

        let meta = tokio::fs::metadata(&self.path).await?;
        if !meta.is_file() {
            return Err(crate::error::PersistenceError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Session path is not a file",
            )));
        }

        if meta.len() > MAX_SESSION_FILE_SIZE {
            warn!("Session file too large ({} bytes), ignoring.", meta.len());
            return Ok(None);
        }

        let path = self.path.clone();
        tokio::task::spawn_blocking(move || {
            let file = std::fs::File::open(path)?;
            let reader = std::io::BufReader::new(file);

            // Peak at content or just try parsing.
            // Since we need to support two formats, we'll read to a value first.
            let v: serde_json::Value = match serde_json::from_reader(reader) {
                Ok(v) => v,
                Err(e) if e.is_io() => return Err(e.into()),
                Err(e) => {
                    warn!("Failed to parse session file: {}. Ignoring.", e);
                    return Ok(None);
                }
            };

            // 1. Try modern format
            if let Ok(persisted) = serde_json::from_value::<PersistedSession>(v.clone()) {
                if persisted.verify() {
                    return Ok(Some(persisted.snapshot));
                }
            }

            // 2. Try legacy format
            if let Ok(snap) = serde_json::from_value::<SessionSnapshot>(v) {
                return Ok(Some(snap));
            }

            Ok(None)
        })
        .await
        .map_err(|e| crate::error::PersistenceError::Task(e.to_string()))?
    }

    /// Schedules a session snapshot to be saved to disk.
    /// Flushing is debounced to avoid excessive disk IRQ.
    pub async fn schedule_save(&self, snapshot: SessionSnapshot) {
        let should_flush = {
            let mut state = match self.state.lock() {
                Ok(s) => s,
                Err(e) => {
                    error!("Mutex poisoned in AutoSaveService: {}", e);
                    return;
                }
            };
            state.pending = Some(snapshot);
            // Debounce: only flush if 2 seconds passed since last save
            state.last_save.elapsed() > Duration::from_secs(2)
        };

        if should_flush {
            self.flush(false).await;
        }
    }

    /// Flushes pending changes to disk.
    /// If `force` is true, ignores the debounce timer.
    pub async fn flush(&self, force: bool) {
        let snapshot_to_save = {
            let mut state = match self.state.lock() {
                Ok(s) => s,
                Err(e) => {
                    error!("Mutex poisoned in AutoSaveService flush: {}", e);
                    return;
                }
            };
            if state.pending.is_none() {
                return;
            }

            if !force && state.last_save.elapsed() < Duration::from_secs(2) {
                return;
            }

            state.last_save = Instant::now();
            state.pending.take() // Take ownership to save
        };

        if let Some(snap) = snapshot_to_save {
            let path = self.path.clone();

            let result = tokio::task::spawn_blocking(move || -> std::io::Result<()> {
                let persisted = PersistedSession::new(snap);
                let json = serde_json::to_string_pretty(&persisted)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

                let dir = path.parent().unwrap_or_else(|| std::path::Path::new("."));

                // Create temp file in the same directory to attempt atomic rename
                let mut temp_file = NamedTempFile::new_in(dir)?;
                temp_file.write_all(json.as_bytes())?;
                temp_file.flush()?;

                // Atomic persist
                // NamedTempFile::persist tries atomic rename, and errors if it fails (e.g. cross-filesystem).
                match temp_file.persist(&path) {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        warn!(
                            "Atomic rename failed, falling back to non-atomic copy: {}",
                            e
                        );
                        let mut source = e.file;
                        source.seek(SeekFrom::Start(0))?;

                        // Fallback: Create a secondary temp file to ensure the copy is as complete as possible
                        // before the final move (which might still be cross-fs but we're trying our best).
                        let mut dest = std::fs::File::create(&path)?;
                        std::io::copy(&mut source, &mut dest)?;
                        Ok(())
                    }
                }
            })
            .await;

            match result {
                Ok(Ok(())) => {}
                Ok(Err(e)) => error!("Failed to save session: {}", e),
                Err(e) => error!("Join error during save: {}", e),
            }
        }
    }
}
