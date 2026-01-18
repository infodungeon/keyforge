use crate::infra::queue::wrappers::{DeadLetterQueue, PersistedRecord, WalEntry};
use crate::config::QueueConfig;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use tokio::sync::{mpsc, oneshot};
use tokio::time::sleep;
use tracing::{error, info};
use uuid::Uuid;

// --- Constants ---
pub const QUEUE_MAX_RETRIES: u32 = 3;
pub const QUEUE_RETRY_DELAY_MS: u64 = 100;

/// Trait for a sink that can accept batches of records.
/// Decouples the queue from the concrete `ResultRepository`.
#[async_trait::async_trait]
pub trait BatchSink: Send + Sync + 'static {
    async fn insert_batch(&self, items: &[(&str, &str, f32, &str)]) -> Result<(), String>;
}

/// Events that can be submitted to the background write queue for persistence.
#[derive(Debug)]
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

/// A background worker that manages asynchronous, batched persistence of 
/// optimization results to the database.
#[derive(Debug)]
pub struct PersistentJobQueue {
    sender: mpsc::Sender<InternalEvent>,
    queue_dir: PathBuf,
    dlq: DeadLetterQueue,
    capacity: usize,
}

impl PersistentJobQueue {
    /// Creates and starts a new `PersistentJobQueue`.
    pub fn new<S>(sink: S, data_path: PathBuf, config: QueueConfig) -> Self 
    where S: BatchSink {
        let queue_dir = data_path.join("user/queue");
        let dlq = DeadLetterQueue::new(data_path.clone());
        let capacity = config.channel_capacity; 

        let (tx, mut rx) = mpsc::channel(capacity);
        let queue_dir_clone = queue_dir.clone();
        let dlq_clone = DeadLetterQueue::new(data_path.clone());

        tokio::spawn(async move {
            if let Err(e) = fs::create_dir_all(&queue_dir_clone).await {
                error!("FATAL: Could not create queue directory: {}", e);
                return;
            }

            let mut buffer = Vec::new();

            // --- WAL RECOVERY PHASE (Ordered) ---
            if let Ok(mut entries) = fs::read_dir(&queue_dir_clone).await {
                let mut wal_files = Vec::new();
                while let Ok(Some(entry)) = entries.next_entry().await {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("bin") {
                        if let Ok(meta) = entry.metadata().await {
                            let created = meta.created().unwrap_or(SystemTime::now());
                            wal_files.push((created, path));
                        }
                    }
                }
                
                wal_files.sort_by_key(|(t, _)| *t);

                for (_, path) in wal_files {
                    if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                        if let Ok(id) = Uuid::parse_str(file_stem) {
                            if let Ok(bytes) = fs::read(&path).await {
                                if let Ok(val_entry) = postcard::from_bytes::<WalEntry>(&bytes) {
                                    let record_bytes = postcard::to_stdvec(&val_entry.record).unwrap_or_default();
                                    if crc32fast::hash(&record_bytes) == val_entry.checksum {
                                        buffer.push((id, val_entry.record, None));
                                    }
                                }
                            }
                        }
                    }
                }
                if !buffer.is_empty() {
                    info!("🔄 WAL Recovery: Found {} orphaned records, re-injecting...", buffer.len());
                }
            }

            loop {
                let batch_size = config.batch_size;
                let flush_interval = config.flush_interval_ms;
                let timeout = sleep(Duration::from_millis(flush_interval));

                tokio::select! {
                    msg = rx.recv() => {
                        match msg {
                            Some(InternalEvent::Item { id, data, ack }) => {
                                buffer.push((id, data, ack));
                                if buffer.len() >= batch_size {
                                    flush_buffer(&sink, &queue_dir_clone, &dlq_clone, &mut buffer).await;
                                }
                            }
                            Some(InternalEvent::Shutdown(signal)) => {
                                info!("🛑 WriteQueue flushing remaining {} items...", buffer.len());
                                while let Ok(InternalEvent::Item { id, data, ack }) = rx.try_recv() {
                                    buffer.push((id, data, ack));
                                }
                                flush_buffer(&sink, &queue_dir_clone, &dlq_clone, &mut buffer).await;
                                let _ = signal.send(());
                                break;
                            }
                            None => break,
                        }
                    }
                    () = timeout => {
                        if !buffer.is_empty() {
                            flush_buffer(&sink, &queue_dir_clone, &dlq_clone, &mut buffer).await;
                        }
                    }
                }
            }
        });

        Self {
            sender: tx,
            queue_dir,
            dlq,
            capacity,
        }
    }

    pub async fn push(&self, event: DbEvent) {
        match event {
            DbEvent::Result { job_id, layout, score, node_id, ack } => {
                let id = Uuid::new_v4();
                let record = PersistedRecord { job_id, layout, score, node_id };
                let record_bytes = postcard::to_stdvec(&record).unwrap_or_default();
                let checksum = crc32fast::hash(&record_bytes);
                let entry = WalEntry { checksum, record: record.clone() };
                let final_path = self.queue_dir.join(format!("{id}.bin"));
                let temp_path = self.queue_dir.join(format!("{id}.tmp"));

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

    pub async fn current_depth(&self) -> usize {
        self.capacity - self.sender.capacity()
    }

    pub async fn shutdown(&self) {
        let (tx, rx) = oneshot::channel();
        if self.sender.send(InternalEvent::Shutdown(tx)).await.is_ok() {
            let _ = rx.await;
        }
    }
}

async fn flush_buffer<S: BatchSink>(
    sink: &S,
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

    while attempts < QUEUE_MAX_RETRIES {
        if sink.insert_batch(&items).await.is_ok() {
            success = true;
            break;
        }
        attempts += 1;
        sleep(Duration::from_millis(QUEUE_RETRY_DELAY_MS * u64::from(attempts))).await;
    }

    if success {
        for (id, _, ack) in buffer.drain(..) {
            let path = queue_dir.join(format!("{id}.bin"));
            let _ = fs::remove_file(&path).await;
            if let Some(tx) = ack { let _ = tx.send(()); }
        }
    } else {
        error!("❌ Batch Insert Failed. DLQ.");
        for (id, record, _) in buffer.drain(..) {
            dlq.push(&record, "Batch Insert Failed").await;
            let path = queue_dir.join(format!("{id}.bin"));
            let _ = fs::remove_file(&path).await;
        }
    }
}
