use super::breaker::CircuitBreaker;
use crate::agent::errors::{AgentError, AgentResult};
use keyforge_infra::HiveClient;
use keyforge_protocol::ResultSubmission;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use tracing::{debug, error, info, warn};

/// Typestate for a WAL entry that has just been loaded from disk.
#[derive(Debug)]
pub struct Unprocessed;
/// Typestate for a WAL entry that has been validated and is ready for submission.
#[derive(Debug)]
pub struct Verified;
/// Typestate for a WAL entry that failed validation and is destined for the dead letter office.
#[derive(Debug)]
pub struct DeadLetter;

/// A reified WAL entry using the Typestate pattern to enforce lifecycle invariants.
#[derive(Debug)]
pub struct WalEntry<State> {
    pub path: PathBuf,
    pub submission: ResultSubmission,
    _state: PhantomData<State>,
}

impl WalEntry<Unprocessed> {
    /// Attempts to load a `WalEntry` from a path.
    pub fn load(path: PathBuf) -> AgentResult<Self> {
        let content =
            std::fs::read_to_string(&path).map_err(|e| AgentError::Resource(e.to_string()))?;
        let submission = serde_json::from_str::<ResultSubmission>(&content)
            .map_err(|_| AgentError::Identity("Corrupt WAL".into()))?;
        Ok(Self {
            path,
            submission,
            _state: PhantomData,
        })
    }

    /// Transitions an unprocessed entry to a verified state.
    #[must_use]
    pub fn verify(self) -> WalEntry<Verified> {
        WalEntry {
            path: self.path,
            submission: self.submission,
            _state: PhantomData,
        }
    }

    /// Transitions an unprocessed entry to a dead-letter state.
    #[must_use]
    pub fn reject(self) -> WalEntry<DeadLetter> {
        WalEntry {
            path: self.path,
            submission: self.submission,
            _state: PhantomData,
        }
    }
}

#[derive(Debug)]
pub struct ResultOutbox {
    _client: HiveClient,
    wal_dir: PathBuf,
    dead_letter_dir: PathBuf,
    _breaker: CircuitBreaker,
}

impl ResultOutbox {
    #[must_use]
    pub fn new(client: HiveClient, data_root: &Path, threshold: u32, cooldown_secs: u64) -> Self {
        let wal_dir = data_root.join("user/agent_wal");
        let dead_letter_dir = data_root.join("user/dead_letter");
        std::fs::create_dir_all(&wal_dir).ok();
        std::fs::create_dir_all(&dead_letter_dir).ok();
        Self {
            _client: client,
            wal_dir,
            dead_letter_dir,
            _breaker: CircuitBreaker::new(threshold, cooldown_secs),
        }
    }

    pub fn save_to_wal(&self, submission: &ResultSubmission) -> AgentResult<()> {
        let path = self.wal_dir.join(format!(
            "result_{}_{}.json",
            submission.job_id, submission.nonce
        ));
        if let Ok(json) = serde_json::to_string(submission) {
            if let Err(e) = std::fs::write(&path, json) {
                error!(
                    "CRITICAL: Failed to write result to WAL at {:?}: {}",
                    path, e
                );
                return Err(AgentError::Resource(e.to_string()));
            }
            info!("Buffered result {} to WAL", submission.job_id);
        }
        Ok(())
    }

    pub fn save_to_dead_letter(
        &self,
        submission: &ResultSubmission,
        reason: &str,
    ) -> AgentResult<()> {
        let path = self
            .dead_letter_dir
            .join(format!("{}.json", submission.nonce));
        let mut map = serde_json::to_value(submission).unwrap_or_default();
        if let Some(obj) = map.as_object_mut() {
            obj.insert(
                "rejection_reason".to_string(),
                serde_json::Value::String(reason.to_string()),
            );
        }
        if let Ok(json) = serde_json::to_string_pretty(&map) {
            std::fs::write(&path, json).ok();
            warn!("Saved rejected result to Dead Letter: {:?}", path);
        }
        Ok(())
    }

    /// Processes pending WAL entries one by one using a callback.
    /// This prevents memory spikes for large backlogs.
    ///
    /// # Errors
    /// Returns `InfraError` if directory reading fails.
    pub fn process_pending<F>(&self, mut handler: F) -> AgentResult<()>
    where
        F: FnMut(WalEntry<Unprocessed>),
    {
        if let Ok(entries) = std::fs::read_dir(&self.wal_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(submission) = serde_json::from_str::<ResultSubmission>(&content) {
                            let entry = WalEntry {
                                path,
                                submission,
                                _state: PhantomData,
                            };
                            handler(entry);
                        } else {
                            warn!("Deleting corrupt WAL file: {:?}", path);
                            let _ = std::fs::remove_file(path);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn commit_verified(&self, entry: WalEntry<Verified>) {
        self.delete(&entry.path);
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn commit_dead_letter(&self, entry: WalEntry<DeadLetter>, reason: &str) {
        let _ = self.save_to_dead_letter(&entry.submission, reason);
        self.delete(&entry.path);
    }

    // Deprecated: keeping for compatibility until all callers use process_pending
    #[must_use]
    pub fn get_pending(&self) -> Vec<(PathBuf, ResultSubmission)> {
        let mut pending = Vec::new();
        let _ = self.process_pending(|entry| pending.push((entry.path, entry.submission)));
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tempfile::tempdir;
    use tokio::fs;

    #[tokio::test]
    async fn test_wal_persistence_on_failure() {
        let dir = tempdir().unwrap();
        let data_root = dir.path().to_path_buf();
        let wal_dir = data_root.join("user/agent_wal");

        let client = HiveClient::new(keyforge_infra::net::client::ClientConfig {
            api_url: "http://localhost:1".into(),
            asset_url: "http://localhost:1".into(),
            ..Default::default()
        })
        .unwrap();
        let outbox = ResultOutbox::new(client, &data_root, 10, 60);

        let submission = ResultSubmission {
            version: 1,
            job_id: "test-job".into(),
            layout: "a b c".into(),
            score: 10.5,
            raw_score: 10_500_000,
            node_id: "test-node".into(),
            timestamp: 123456789,
            nonce: 42,
            signature: "dummy".into(),
        };

        outbox.save_to_wal(&submission).unwrap();

        tokio::time::sleep(Duration::from_millis(200)).await;

        let mut entries = fs::read_dir(wal_dir).await.unwrap();
        let mut found = false;
        while let Ok(Some(entry)) = entries.next_entry().await {
            if entry.path().extension().and_then(|s| s.to_str()) == Some("json") {
                found = true;
                break;
            }
        }
        assert!(
            found,
            "WAL file should have been created for failed submission"
        );
    }
}
