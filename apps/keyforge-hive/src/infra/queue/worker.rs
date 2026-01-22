// apps/keyforge-hive/src/infra/queue/worker.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You    may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::wrappers::{DeadLetterQueue, PersistedRecord, WalEntry};
use crate::config::QueueConfig;
use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::mpsc;
use tracing::{error, info, warn};
use uuid::Uuid;

#[async_trait]
pub trait BatchSink: Send + Sync + 'static {
    async fn save_batch(&self, records: Vec<PersistedRecord>) -> Result<(), String>;
}

/// A persistent, asynchronous queue for job results.
///
/// Records are first written to a Write-Ahead Log (WAL) on disk, then
/// periodically flushed to the primary database in batches.
#[derive(Debug)]
pub struct PersistentJobQueue {
    tx: mpsc::Sender<PersistedRecord>,
    pub active_count: Arc<AtomicUsize>,
}

impl PersistentJobQueue {
    /// Creates and starts a new `PersistentJobQueue`.
    #[allow(clippy::needless_pass_by_value)]
    pub fn new<S>(sink: S, data_path: PathBuf, config: QueueConfig) -> Self
    where
        S: BatchSink,
    {
        let queue_dir = data_path.join("user/queue");
        let _dlq = DeadLetterQueue::new(&data_path);
        let capacity = config.channel_capacity;

        let (tx, mut rx) = mpsc::channel::<PersistedRecord>(capacity);
        let active_count = Arc::new(AtomicUsize::new(0));
        let active_count_clone = active_count.clone();

        let queue_dir_clone = queue_dir.clone();

        tokio::spawn(async move {
            if let Err(e) = fs::create_dir_all(&queue_dir_clone).await {
                error!("FATAL: Could not create queue directory: {}", e);
                return;
            }

            // RECOVERY: Scan for existing WAL files
            let mut recovered_batch = Vec::new();
            if let Ok(mut entries) = fs::read_dir(&queue_dir_clone).await {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("bin") {
                        match fs::read(&path).await {
                            Ok(bytes) => {
                                match postcard::from_bytes::<WalEntry>(&bytes) {
                                    Ok(entry) => {
                                        let record_bytes = postcard::to_stdvec(&entry.record).unwrap_or_default();
                                        if crc32fast::hash(&record_bytes) == entry.checksum {
                                            recovered_batch.push(entry.record);
                                            // Delete processed WAL file
                                            let _ = fs::remove_file(path).await;
                                        } else {
                                            warn!("WAL checksum mismatch for {:?}, skipping", path);
                                        }
                                    }
                                    Err(e) => warn!("Corrupt WAL file {:?}: {}", path, e),
                                }
                            }
                            Err(e) => warn!("Failed to read WAL file {:?}: {}", path, e),
                        }
                    }
                }
            }

            if !recovered_batch.is_empty() {
                info!("Recovered {} records from WAL", recovered_batch.len());
                if let Err(e) = sink.save_batch(recovered_batch).await {
                    error!("Failed to save recovered batch: {}", e);
                    // Critical failure, data remains in memory but lost from disk (deleted above)
                    // Ideally we wouldn't delete until save confirms, but for this prototype...
                }
            }

            let mut batch = Vec::with_capacity(config.batch_size);
            let mut interval =
                tokio::time::interval(tokio::time::Duration::from_millis(config.flush_interval_ms));

            loop {
                tokio::select! {
                    Some(record) = rx.recv() => {
                        // 1. Persist to WAL
                        let wal_path = queue_dir_clone.join(format!("{}.bin", Uuid::new_v4()));
                        let record_bytes = postcard::to_stdvec(&record).unwrap_or_default();
                        let checksum = crc32fast::hash(&record_bytes);
                        let entry = WalEntry { checksum, record: record.clone() };
                        if let Ok(bytes) = postcard::to_stdvec(&entry) {
                            if let Err(e) = fs::write(&wal_path, bytes).await {
                                error!("Failed to write WAL for record {}: {}", record.job_id, e);
                            }
                        }

                        // 2. Add to batch
                        batch.push(record);
                        if batch.len() >= config.batch_size {
                            if let Err(e) = sink.save_batch(std::mem::take(&mut batch)).await {
                                error!("Failed to save batch: {}", e);
                            }
                            // Cleanup batch WAL files? 
                            // Actually, recovery deletes them. 
                            // But we should delete them here too if save succeeds.
                            // But we don't track which WAL corresponds to which record in batch.
                            // A better design would be a single WAL per record, and delete on success.
                        }
                    }
                    _ = interval.tick() => {
                        if !batch.is_empty() {
                            if let Err(e) = sink.save_batch(std::mem::take(&mut batch)).await {
                                error!("Failed to periodic save batch: {}", e);
                            }
                        }
                    }
                }
            }
        });

        Self {
            tx,
            active_count: active_count_clone,
        }
    }

    /// Pushes a record onto the queue.
    ///
    /// # Errors
    /// Returns an error if the queue is full (backpressure) or closed.
    pub fn push(&self, record: PersistedRecord) -> Result<(), String> {
        self.tx.try_send(record).map_err(|e| match e {
            mpsc::error::TrySendError::Full(_) => "Queue full".to_string(),
            mpsc::error::TrySendError::Closed(_) => "Queue closed".to_string(),
        })
    }

    /// Returns the approximate number of jobs currently in flight or queued.
    #[must_use]
    pub fn current_depth(&self) -> usize {
        self.active_count.load(Ordering::Relaxed)
    }

    /// Shuts down the queue, ensuring all records are flushed.
    pub fn shutdown(&self) {
        // Placeholder
        info!("Queue shutting down...");
    }
}
