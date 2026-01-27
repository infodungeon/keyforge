// apps/keyforge-agent/src/agent/network/outbox.rs

use crate::agent::errors::{AgentError, AgentResult};
use keyforge_protocol::ResultSubmission;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct ResultOutbox {
    wal_dir: PathBuf,
    dead_letter_dir: PathBuf,
}

impl ResultOutbox {
    #[must_use]
    pub fn new(
        _client: keyforge_infra::HiveClient,
        data_dir: &Path,
        _threshold: u32,
        _cooldown: u64,
    ) -> Self {
        let wal_dir = data_dir.join("wal");
        let dead_letter_dir = data_dir.join("dead_letter");
        std::fs::create_dir_all(&wal_dir).ok();
        std::fs::create_dir_all(&dead_letter_dir).ok();

        Self {
            wal_dir,
            dead_letter_dir,
        }
    }

    pub fn save_to_wal(&self, submission: &ResultSubmission) -> AgentResult<()> {
        let path = self.wal_dir.join(format!("{}.json", submission.nonce));
        let json = serde_json::to_string(submission)?;
        std::fs::write(path, json).map_err(AgentError::from)
    }

    /// # Errors
    /// Returns `AgentError` if serialization or write fails.
    pub fn save_to_dead_letter(
        &self,
        submission: &ResultSubmission,
        reason: &str,
    ) -> AgentResult<()> {
        let path = self
            .dead_letter_dir
            .join(format!("{}.json", submission.nonce));
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
        std::fs::write(path, json).map_err(AgentError::from)
    }

    pub fn get_pending(&self) -> HashMap<PathBuf, ResultSubmission> {
        let mut map = HashMap::new();
        if let Ok(entries) = std::fs::read_dir(&self.wal_dir) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
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
