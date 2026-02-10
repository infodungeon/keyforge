use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{error, warn};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PersistedRecord {
    pub job_id: String,
    pub layout: String,
    pub score: f32,
    pub raw_score: i64,
    pub node_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WalEntry {
    pub checksum: u32,
    pub record: PersistedRecord,
}

#[derive(Debug, Clone)]
pub struct DeadLetterQueue {
    path: keyforge_boundary::SafePath,
}

impl DeadLetterQueue {
    /// Creates a new `DeadLetterQueue` rooted at the provided data path.
    #[must_use]
    pub fn new(data_path: &Path) -> Self {
        Self {
            path: keyforge_boundary::SafePath::from_trusted_root_path(data_path.join("user/dlq")),
        }
    }

    pub async fn push(&self, record: &PersistedRecord, reason: &str) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let filename = format!("{}_{}.json", timestamp, Uuid::new_v4());
        let rel = match keyforge_boundary::SafePath::try_from_str(&filename) {
            Ok(r) => r,
            Err(e) => {
                error!("Failed to create safe filename for DLQ: {}", e);
                return;
            }
        };
        let file_path = self.path.join_trusted(&rel);
        let payload = serde_json::json!({ "error": reason, "record": record });

        // Task-sec-027: Use atomic_write for persistence
        let payload_str = payload.to_string();
        let path_clone = file_path.clone();

        let res = tokio::task::spawn_blocking(move || {
            keyforge_infra::fs::io::atomic_write(&path_clone, payload_str)
        })
        .await;

        if let Err(e) =
            res.unwrap_or_else(|e| Err(keyforge_infra::error::InfraError::Internal(e.to_string())))
        {
            error!("CRITICAL: Failed to write to DLQ: {}", e);
        } else {
            warn!("Moved record to DLQ: {:?}", file_path.as_path());
        }
    }
}
