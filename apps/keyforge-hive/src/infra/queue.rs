// ===== keyforge/crates/keyforge-hive/src/infra/queue.rs =====
use crate::cache::GlobalAssetCache;
use crate::infra::repositories::ResultRepository;
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

pub enum DbEvent {
    Result {
        job_id: String,
        layout: String,
        score: f32,
        node_id: String,
        ack: Option<oneshot::Sender<()>>,
    },
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
        Self {
            // CHANGED: Use user/dlq
            path: data_path.join("user/dlq"),
        }
    }

    async fn push(&self, record: &PersistedRecord, reason: &str) {
        if let Err(e) = fs::create_dir_all(&self.path).await {
            error!("Failed to create DLQ dir: {}", e);
            return;
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_millis();
        let filename = format!("{}_{}.json", timestamp, Uuid::new_v4());
        let file_path = self.path.join(filename);

        let payload = serde_json::json!({
            "error": reason,
            "record": record
        });

        if let Err(e) = fs::write(&file_path, payload.to_string()).await {
            error!("CRITICAL: Failed to write to DLQ: {}", e);
        } else {
            warn!("Moved record to DLQ: {:?}", file_path);
        }
    }
}

pub struct WriteQueue {
    sender: mpsc::Sender<InternalEvent>,
    queue_dir: PathBuf,
    dlq: DeadLetterQueue,
}

impl WriteQueue {
    pub fn new(repo: ResultRepository, data_path: PathBuf, assets: Arc<GlobalAssetCache>) -> Self {
        // CHANGED: Use user/queue
        let queue_dir = data_path.join("user/queue");
        let dlq = DeadLetterQueue::new(data_path.clone());

        // Load initial config
        let config = assets.load_hive_config();
        let capacity = config.queue.channel_capacity;

        let (tx, mut rx) = mpsc::channel(capacity);

        let queue_dir_clone = queue_dir.clone();
        let dlq_clone = DeadLetterQueue::new(data_path.clone());
        let assets_clone = assets.clone();

        tokio::spawn(async move {
            if let Err(e) = fs::create_dir_all(&queue_dir_clone).await {
                error!("FATAL: Could not create queue directory: {}", e);
                return;
            }

            // Recovery Phase
            let mut buffer = Vec::new();
            match fs::read_dir(&queue_dir_clone).await {
                Ok(mut entries) => {
                    while let Ok(Some(entry)) = entries.next_entry().await {
                        let path = entry.path();
                        if path.extension().and_then(|s| s.to_str()) == Some("bin") {
                            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                                if let Ok(uuid) = Uuid::parse_str(stem) {
                                    match fs::read(&path).await {
                                        Ok(bytes) => {
                                            match postcard::from_bytes::<WalEntry>(&bytes) {
                                                Ok(entry) => {
                                                    // Verify Checksum
                                                    let calc_crc = match postcard::to_stdvec(
                                                        &entry.record,
                                                    ) {
                                                        Ok(b) => crc32fast::hash(&b),
                                                        Err(e) => {
                                                            error!("Corrupt WAL (serialize failure) {:?}: {}", path, e);
                                                            continue;
                                                        }
                                                    };
                                                    if calc_crc == entry.checksum {
                                                        info!("♻️ Recovered WAL record: {}", uuid);
                                                        buffer.push((uuid, entry.record, None));
                                                    } else {
                                                        error!("❌ Corrupt WAL (Checksum Mismatch): {:?}", path);
                                                    }
                                                }
                                                Err(e) => {
                                                    error!(
                                                        "❌ Invalid WAL format {:?}: {}",
                                                        path, e
                                                    );
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            error!("Failed to read WAL file {:?}: {}", path, e)
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => error!("Failed to scan queue directory: {}", e),
            }

            // Dynamic Config Loop
            loop {
                let current_config = assets_clone.load_hive_config();
                let batch_size = current_config.queue.batch_size;
                let flush_interval = current_config.queue.flush_interval_ms;

                // Use timeout to implement flush interval
                let timeout = tokio::time::sleep(Duration::from_millis(flush_interval));

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

    pub async fn push(&self, event: DbEvent) {
        match event {
            DbEvent::Result {
                job_id,
                layout,
                score,
                node_id,
                ack,
            } => {
                let id = Uuid::new_v4();
                let record = PersistedRecord {
                    job_id,
                    layout,
                    score,
                    node_id,
                };

                // Calculate Checksum
                let record_bytes = postcard::to_stdvec(&record).unwrap_or_default();
                let checksum = crc32fast::hash(&record_bytes);

                let entry = WalEntry {
                    checksum,
                    record: record.clone(),
                };

                let final_path = self.queue_dir.join(format!("{}.bin", id));
                let temp_path = self.queue_dir.join(format!("{}.tmp", id));

                match postcard::to_stdvec(&entry) {
                    Ok(bytes) => {
                        // ROBUST WAL WRITE: Create, Write, Sync, Rename (with retry)
                        let mut attempts = 0;
                        let max_attempts = 3;
                        let mut success = false;

                        while attempts < max_attempts {
                            match File::create(&temp_path).await {
                                Ok(mut file) => {
                                    if let Err(e) = file.write_all(&bytes).await {
                                        warn!(
                                            "⚠️ WAL Temp Write Attempt {} Failed: {}",
                                            attempts + 1,
                                            e
                                        );
                                        attempts += 1;
                                        sleep(Duration::from_millis(50)).await;
                                        continue;
                                    }
                                    if let Err(e) = file.sync_all().await {
                                        warn!("⚠️ WAL Sync Attempt {} Failed: {}", attempts + 1, e);
                                        attempts += 1;
                                        sleep(Duration::from_millis(50)).await;
                                        continue;
                                    }
                                    success = true;
                                    break;
                                }
                                Err(e) => {
                                    warn!(
                                        "⚠️ WAL File Create Attempt {} Failed: {}",
                                        attempts + 1,
                                        e
                                    );
                                    attempts += 1;
                                    sleep(Duration::from_millis(50)).await;
                                }
                            }
                        }

                        if !success {
                            error!("❌ WAL Write failed after {} attempts.", max_attempts);
                            self.dlq
                                .push(&record, "WAL write failed after retries")
                                .await;
                            return;
                        }

                        if let Err(e) = fs::rename(&temp_path, &final_path).await {
                            error!("❌ WAL Rename Failed: {}", e);
                            let _ = fs::remove_file(&temp_path).await;
                            self.dlq
                                .push(&record, &format!("WAL Rename Failed: {}", e))
                                .await;
                            return;
                        }
                    }
                    Err(e) => {
                        error!("Serialization Failed: {}", e);
                        self.dlq
                            .push(&record, &format!("Serialization Failed: {}", e))
                            .await;
                        return;
                    }
                }

                if self
                    .sender
                    .send(InternalEvent::Item {
                        id,
                        data: record,
                        ack,
                    })
                    .await
                    .is_err()
                {
                    error!("Queue channel closed");
                }
            }
            DbEvent::Shutdown(tx) => {
                let _ = self.sender.send(InternalEvent::Shutdown(tx)).await;
            }
        }
    }

    pub async fn current_depth(&self) -> usize {
        10000 - self.sender.capacity()
    }

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
    if buffer.is_empty() {
        return;
    }

    let items: Vec<(&str, &str, f32, &str)> = buffer
        .iter()
        .map(|(_, r, _)| {
            (
                r.job_id.as_str(),
                r.layout.as_str(),
                r.score,
                r.node_id.as_str(),
            )
        })
        .collect();

    let mut attempts = 0;
    let max_attempts = 3;
    let mut success = false;

    while attempts < max_attempts {
        if let Err(e) = repo.insert_batch(&items).await {
            attempts += 1;
            warn!(
                "⚠️ Batch Insert Failed (Attempt {}/{}): {}",
                attempts, max_attempts, e
            );
            sleep(Duration::from_millis(100 * attempts as u64)).await;
        } else {
            success = true;
            break;
        }
    }

    if success {
        for (id, _, ack) in buffer.drain(..) {
            let path = queue_dir.join(format!("{}.bin", id));
            if let Err(e) = fs::remove_file(&path).await {
                error!("❌ Failed to delete WAL file {:?}: {}", path, e);
            }
            if let Some(tx) = ack {
                let _ = tx.send(());
            }
        }
    } else {
        error!(
            "❌ Batch Insert Failed after {} attempts. Moving to DLQ.",
            max_attempts
        );
        for (id, record, _ack) in buffer.drain(..) {
            dlq.push(&record, "DB Batch Insert Failed").await;
            let path = queue_dir.join(format!("{}.bin", id));
            let _ = fs::remove_file(path).await;
        }
    }
}
