use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::fs;
use tracing::{error, warn};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PersistedRecord {
    pub job_id: String,
    pub layout: String,
    pub score: f32,
    pub node_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WalEntry {
    pub checksum: u32,
    pub record: PersistedRecord,
}

#[derive(Debug)]
pub struct DeadLetterQueue {
    path: PathBuf,
}

impl DeadLetterQueue {
    #[must_use] 
    pub fn new(data_path: &Path) -> Self {
        Self { path: data_path.join("user/dlq") }
    }

    pub async fn push(&self, record: &PersistedRecord, reason: &str) {
        if let Err(e) = fs::create_dir_all(&self.path).await {
            error!("Failed to create DLQ dir: {}", e);
            return;
        }
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis();
        let filename = format!("{}_{}.json", timestamp, Uuid::new_v4());
        let file_path = self.path.join(filename);
        let payload = serde_json::json!({ "error": reason, "record": record });
        if let Err(e) = fs::write(&file_path, payload.to_string()).await {
            error!("CRITICAL: Failed to write to DLQ: {}", e);
        } else {
            warn!("Moved record to DLQ: {:?}", file_path);
        }
    }
}
