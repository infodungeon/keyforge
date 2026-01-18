use std::path::PathBuf;
use tracing::{info, warn, error, debug};
use keyforge_infra::HiveClient;
use keyforge_protocol::ResultSubmission;
use crate::agent::errors::{AgentResult, AgentError};
use super::breaker::CircuitBreaker;

#[derive(Debug)]
pub struct ResultOutbox {
    _client: HiveClient,
    wal_dir: PathBuf,
    dead_letter_dir: PathBuf,
    _breaker: CircuitBreaker,
}

impl ResultOutbox {
    #[must_use] 
    pub fn new(client: HiveClient, data_root: PathBuf, threshold: u32, cooldown_secs: u64) -> Self {
        let wal_dir = data_root.join("user/agent_wal");
        let dead_letter_dir = data_root.join("user/dead_letter");
        std::fs::create_dir_all(&wal_dir).ok();
        std::fs::create_dir_all(&dead_letter_dir).ok();
        Self { 
            _client: client, 
            wal_dir, 
            dead_letter_dir, 
            _breaker: CircuitBreaker::new(threshold, cooldown_secs) 
        }
    }

    pub fn save_to_wal(&self, submission: &ResultSubmission) -> AgentResult<()> {
        let path = self.wal_dir.join(format!("{}.json", submission.nonce));
        if let Ok(json) = serde_json::to_string(submission) {
             if let Err(e) = std::fs::write(&path, json) {
                 error!("CRITICAL: Failed to write result to WAL at {:?}: {}", path, e);
                 return Err(AgentError::Resource(e.to_string()));
             }
             info!("Buffered result {} to WAL", submission.job_id);
        }
        Ok(())
    }

    pub fn save_to_dead_letter(&self, submission: &ResultSubmission, reason: &str) -> AgentResult<()> {
        let path = self.dead_letter_dir.join(format!("{}.json", submission.nonce));
        let mut map = serde_json::to_value(submission).unwrap_or_default();
        if let Some(obj) = map.as_object_mut() {
            obj.insert("rejection_reason".to_string(), serde_json::Value::String(reason.to_string()));
        }
        if let Ok(json) = serde_json::to_string_pretty(&map) {
             std::fs::write(&path, json).ok();
             warn!("Saved rejected result to Dead Letter: {:?}", path);
        }
        Ok(())
    }

    pub fn get_pending(&self) -> Vec<(PathBuf, ResultSubmission)> {
        let mut pending = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&self.wal_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(sub) = serde_json::from_str::<ResultSubmission>(&content) {
                            pending.push((path, sub));
                        } else {
                            warn!("Deleting corrupt WAL file: {:?}", path);
                            let _ = std::fs::remove_file(path);
                        }
                    }
                }
            }
        }
        pending
    }

    pub fn delete(&self, path: &PathBuf) {
        if let Err(e) = std::fs::remove_file(path) {
            warn!("Failed to delete WAL file {:?}: {}", path, e);
        } else {
            debug!("Removed WAL file {:?}", path);
        }
    }
}
