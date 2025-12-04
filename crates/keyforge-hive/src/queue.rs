use crate::store::Store;
use tokio::sync::{mpsc, oneshot};
use tracing::{error, info};

pub enum DbEvent {
    Result {
        job_id: String,
        layout: String,
        score: f32,
        node_id: String,
    },
    // Internal signal to flush and quit
    Shutdown(oneshot::Sender<()>),
}

pub struct WriteQueue {
    sender: mpsc::Sender<DbEvent>,
}

impl WriteQueue {
    pub fn new(store: Store, buffer_size: usize) -> Self {
        let (tx, mut rx) = mpsc::channel(buffer_size);

        tokio::spawn(async move {
            let mut batch = Vec::with_capacity(100);
            let mut shutdown_signal: Option<oneshot::Sender<()>> = None;

            loop {
                // 1. Fetch first message (wait if empty)
                let first = rx.recv().await;
                if first.is_none() {
                    break;
                } // Channel closed

                let msg = first.unwrap();

                // If the very first message is Shutdown, set flag and skip to flush
                if let DbEvent::Shutdown(signal) = msg {
                    shutdown_signal = Some(signal);
                    break;
                }

                batch.push(msg);

                // Flag to break outer loop after flushing
                let mut goto_flush_and_exit = false;

                // 2. Drain remaining available (up to limit)
                while batch.len() < 100 {
                    match rx.try_recv() {
                        Ok(DbEvent::Shutdown(signal)) => {
                            shutdown_signal = Some(signal);
                            goto_flush_and_exit = true;
                            break;
                        }
                        Ok(item) => batch.push(item),
                        Err(_) => break, // Empty
                    }
                }

                // 3. Flush Batch
                if !batch.is_empty() {
                    if let Err(e) = flush_batch(&store, &batch).await {
                        error!("❌ Failed to flush batch of {} records: {}", batch.len(), e);
                    } else if batch.len() > 10 {
                        info!("💾 Flushed {} records to DB", batch.len());
                    }
                    batch.clear();
                }

                if goto_flush_and_exit {
                    break;
                }
            }

            // Final Flush logic
            if !batch.is_empty() {
                info!("🛑 Shutdown: Flushing final {} records...", batch.len());
                if let Err(e) = flush_batch(&store, &batch).await {
                    error!("❌ Failed final flush: {}", e);
                }
            }

            if let Some(signal) = shutdown_signal {
                let _ = signal.send(());
            }
        });

        Self { sender: tx }
    }

    pub async fn push(&self, event: DbEvent) {
        if let Err(e) = self.sender.send(event).await {
            error!("Failed to enqueue DB event: {}", e);
        }
    }

    pub async fn shutdown(&self) {
        let (tx, rx) = oneshot::channel();
        // FIXED: Replaced `if let Err(_) = ...` with `.is_err()`
        if self.sender.send(DbEvent::Shutdown(tx)).await.is_err() {
            error!("Queue already closed");
            return;
        }
        // Wait for the worker to finish flushing
        let _ = rx.await;
        info!("💾 WriteQueue shut down gracefully.");
    }
}

async fn flush_batch(store: &Store, batch: &[DbEvent]) -> Result<(), String> {
    // Start transaction
    let mut tx = store.begin().await?;

    for event in batch {
        if let DbEvent::Result {
            job_id,
            layout,
            score,
            node_id,
        } = event
        {
            sqlx::query("INSERT INTO results (job_id, layout, score, node_id) VALUES (?, ?, ?, ?)")
                .bind(job_id)
                .bind(layout)
                .bind(score)
                .bind(node_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| e.to_string())?;
        }
    }

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}
