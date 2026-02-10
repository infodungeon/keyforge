// apps/keyforge-agent/src/agent/network/outbox.rs

use crate::agent::errors::{AgentError, AgentResult};
use keyforge_boundary::SafePath;
use keyforge_protocol::ResultSubmission;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct ResultOutbox {
    wal_dir: SafePath,
    dead_letter_dir: SafePath,
}

impl ResultOutbox {
    #[must_use]
    pub fn new(
        _client: keyforge_infra::HiveClient,
        data_dir: &SafePath,
        _threshold: u32,
        _cooldown: u64,
    ) -> Self {
        let wal_dir = SafePath::from_trusted_root_path(data_dir.as_path().join("wal"));
        let dead_letter_dir =
            SafePath::from_trusted_root_path(data_dir.as_path().join("dead_letter"));

        // Ensure directories exist. Errors are logged but not fatal during initialization.
        if let Err(e) = std::fs::create_dir_all(wal_dir.as_path()) {
            tracing::error!("Failed to create WAL directory: {}", e);
        }
        if let Err(e) = std::fs::create_dir_all(dead_letter_dir.as_path()) {
            tracing::error!("Failed to create Dead Letter directory: {}", e);
        }

        Self {
            wal_dir,
            dead_letter_dir,
        }
    }

    pub fn save_to_wal(&self, submission: &ResultSubmission) -> AgentResult<()> {
        let rel = SafePath::try_from_str(&format!("{}.json", submission.nonce))?;
        let path = self.wal_dir.join_trusted(&rel);
        let json = serde_json::to_string(submission)?;
        keyforge_infra::fs::io::atomic_write(&path, json).map_err(AgentError::from)
    }

    /// # Errors
    /// Returns `AgentError` if serialization or write fails.
    pub fn save_to_dead_letter(
        &self,
        submission: &ResultSubmission,
        reason: &str,
    ) -> AgentResult<()> {
        let rel = SafePath::try_from_str(&format!("{}.json", submission.nonce))?;
        let path = self.dead_letter_dir.join_trusted(&rel);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| AgentError::Internal(e.to_string()))?
            .as_secs();

        let payload = serde_json::json!({
            "submission": submission,
            "reason": reason,
            "timestamp": timestamp
        });
        let json = serde_json::to_string_pretty(&payload)?;
        keyforge_infra::fs::io::atomic_write(&path, json).map_err(AgentError::from)
    }

    pub fn get_pending(&self) -> HashMap<PathBuf, ResultSubmission> {
        let mut map = HashMap::new();
        if let Ok(entries) = std::fs::read_dir(self.wal_dir.as_path()) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    let safe_path = SafePath::from_trusted_root_path(path.clone());
                    if let Ok(content) =
                        keyforge_infra::fs::io::read_to_string_limited(&safe_path, 10 * 1024 * 1024)
                    {
                        if let Ok(sub) = serde_json::from_str::<ResultSubmission>(&content) {
                            map.insert(path, sub);
                        }
                    }
                }
            }
        }
        map
    }

    pub fn delete(&self, path: &Path) {
        let _ = std::fs::remove_file(path);
    }
}
