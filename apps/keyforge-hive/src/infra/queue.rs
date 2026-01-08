// apps/keyforge-hive/src/infra/queue.rs

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


use crate::infra::repositories::ResultRepository;
use keyforge_infra::ValkeyProvider;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use tokio::sync::{mpsc, oneshot};
use tokio::time::sleep;
use tracing::{error, info, warn};
use uuid::Uuid;
use crate::config::HiveConfig;

#[derive(Serialize, Deserialize, Clone)]
struct PersistedRecord {
    job_id: String,
    layout: String,
    score: f32,
    node_id: String,
}

#[derive(Serialize, Deserialize)]
struct WalEntry {
    checksum: u32,
    record: PersistedRecord,
}

/// Events that can be submitted to the background write queue for persistence.
pub enum DbEvent {
    /// A new optimization result to be persisted.
    Result {
        /// The ID of the job that produced this result.
        job_id: String,
        /// The serialized layout string.
        layout: String,
        /// The optimization score.
        score: f32,
        /// The identifier of the node that performed the work.
        node_id: String,
        /// Optional acknowledgement channel to signal completion.
        ack: Option<oneshot::Sender<()>>,
    },
    /// A signal to shut down the queue gracefully.
    Shutdown(oneshot::Sender<()>),
}

enum InternalEvent {
    Item {
        id: Uuid,
        data: PersistedRecord,
        ack: Option<oneshot::Sender<()>>,
    },
    Shutdown(oneshot::Sender<()>),
}

struct DeadLetterQueue {
    path: PathBuf,
}

impl DeadLetterQueue {
    fn new(data_path: PathBuf) -> Self {
        Self { path: data_path.join("user/dlq") }
    }

    async fn push(&self, record: &PersistedRecord, reason: &str) {
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

/// A background worker that manages asynchronous, batched persistence of 
/// optimization results to the database.
///
/// It uses a Write-Ahead Log (WAL) on the local filesystem to ensure data 
/// durability even if the database is temporarily unavailable or if the 
/// process crashes before a batch is flushed.
pub struct WriteQueue {
    sender: mpsc::Sender<InternalEvent>,
    queue_dir: PathBuf,
    dlq: DeadLetterQueue,
}

impl WriteQueue {
    /// Creates and starts a new `WriteQueue`.
    ///
    /// It initializes the WAL and DLQ directories and spawns the background 
    /// processing task.
    pub fn new(repo: ResultRepository, data_path: PathBuf, assets: Arc<ValkeyProvider>) -> Self {
        let queue_dir = data_path.join("user/queue");
        let dlq = DeadLetterQueue::new(data_path.clone());
        let capacity = 1000; 

        let (tx, mut rx) = mpsc::channel(capacity);
        let queue_dir_clone = queue_dir.clone();
        let dlq_clone = DeadLetterQueue::new(data_path.clone());
        let assets_clone = assets.clone();

        tokio::spawn(async move {
            if let Err(e) = fs::create_dir_all(&queue_dir_clone).await {
                error!("FATAL: Could not create queue directory: {}", e);
                return;
            }

            let mut buffer = Vec::new();

            loop {
                // FIX: Use generic load_config_asset
                let current_config: Arc<HiveConfig> = assets_clone.load_config_asset("hive").await;
                let batch_size = current_config.queue.batch_size;
                let flush_interval = current_config.queue.flush_interval_ms;

                let timeout = sleep(Duration::from_millis(flush_interval));

                tokio::select! {
                    msg = rx.recv() => {
                        match msg {
                            Some(InternalEvent::Item { id, data, ack }) => {
                                buffer.push((id, data, ack));
                                if buffer.len() >= batch_size {
                                    flush_buffer(&repo, &queue_dir_clone, &dlq_clone, &mut buffer).await;
                                }
                            }
                            Some(InternalEvent::Shutdown(signal)) => {
                                info!("🛑 WriteQueue flushing remaining {} items...", buffer.len());
                                while let Ok(InternalEvent::Item { id, data, ack }) = rx.try_recv() {
                                    buffer.push((id, data, ack));
                                }
                                flush_buffer(&repo, &queue_dir_clone, &dlq_clone, &mut buffer).await;
                                let _ = signal.send(());
                                break;
                            }
                            None => break,
                        }
                    }
                    _ = timeout => {
                        if !buffer.is_empty() {
                            flush_buffer(&repo, &queue_dir_clone, &dlq_clone, &mut buffer).await;
                        }
                    }
                }
            }
        });

        Self {
            sender: tx,
            queue_dir,
            dlq,
        }
    }

    /// Pushes a new event onto the queue.
    ///
    /// The event is first written to the WAL before being sent to the background task.
    pub async fn push(&self, event: DbEvent) {
        match event {
            DbEvent::Result { job_id, layout, score, node_id, ack } => {
                let id = Uuid::new_v4();
                let record = PersistedRecord { job_id, layout, score, node_id };
                let record_bytes = postcard::to_stdvec(&record).unwrap_or_default();
                let checksum = crc32fast::hash(&record_bytes);
                let entry = WalEntry { checksum, record: record.clone() };
                let final_path = self.queue_dir.join(format!("{}.bin", id));
                let temp_path = self.queue_dir.join(format!("{}.tmp", id));

                if let Ok(bytes) = postcard::to_stdvec(&entry) {
                    if let Ok(mut file) = File::create(&temp_path).await {
                        if file.write_all(&bytes).await.is_ok() {
                            let _ = file.sync_all().await;
                            if fs::rename(&temp_path, &final_path).await.is_ok() {
                                if self.sender.send(InternalEvent::Item { id, data: record, ack }).await.is_err() {
                                    error!("Queue channel closed");
                                }
                                return;
                            }
                        }
                    }
                }
                error!("WAL Write failed.");
                self.dlq.push(&record, "WAL Write Failed").await;
            }
            DbEvent::Shutdown(tx) => {
                let _ = self.sender.send(InternalEvent::Shutdown(tx)).await;
            }
        }
    }

    /// Returns the current number of messages waiting in the queue buffer.
    pub async fn current_depth(&self) -> usize {
        10000 - self.sender.capacity()
    }

    /// Triggers a graceful shutdown of the queue, ensuring all items are flushed.
    pub async fn shutdown(&self) {
        let (tx, rx) = oneshot::channel();
        if self.sender.send(InternalEvent::Shutdown(tx)).await.is_ok() {
            let _ = rx.await;
        }
    }
}

async fn flush_buffer(
    repo: &ResultRepository,
    queue_dir: &Path,
    dlq: &DeadLetterQueue,
    buffer: &mut Vec<(Uuid, PersistedRecord, Option<oneshot::Sender<()>>)>,
) {
    if buffer.is_empty() { return; }

    let items: Vec<(&str, &str, f32, &str)> = buffer.iter()
        .map(|(_, r, _)| (r.job_id.as_str(), r.layout.as_str(), r.score, r.node_id.as_str()))
        .collect();

    let mut attempts = 0;
    let mut success = false;

    while attempts < 3 {
        if repo.insert_batch(&items).await.is_ok() {
            success = true;
            break;
        }
        attempts += 1;
        sleep(Duration::from_millis(100 * attempts)).await;
    }

    if success {
        for (id, _, ack) in buffer.drain(..) {
            let path = queue_dir.join(format!("{}.bin", id));
            let _ = fs::remove_file(&path).await;
            if let Some(tx) = ack { let _ = tx.send(()); }
        }
    } else {
        error!("❌ Batch Insert Failed. DLQ.");
        for (id, record, _) in buffer.drain(..) {
            dlq.push(&record, "Batch Insert Failed").await;
            let path = queue_dir.join(format!("{}.bin", id));
            let _ = fs::remove_file(&path).await;
        }
    }
}
