use crate::store::Store;
use tokio::sync::mpsc;
use tracing::{error, info};

pub enum DbEvent {
    Result {
        job_id: String,
        layout: String,
        score: f32,
        node_id: String,
    },
}

pub struct WriteQueue {
    sender: mpsc::Sender<DbEvent>,
}

impl WriteQueue {
    pub fn new(store: Store, buffer_size: usize) -> Self {
        let (tx, mut rx) = mpsc::channel(buffer_size);

        tokio::spawn(async move {
            let mut batch = Vec::with_capacity(100);

            loop {
                // 1. Fetch first message (wait if empty)
                let first = rx.recv().await;
                if first.is_none() {
                    break;
                } // Channel closed
                batch.push(first.unwrap());

                // 2. Drain remaining available (up to limit)
                while batch.len() < 100 {
                    match rx.try_recv() {
                        Ok(msg) => batch.push(msg),
                        Err(_) => break, // Empty
                    }
                }

                // 3. Flush Batch
                if !batch.is_empty() {
                    // This requires Store::begin() to be available
                    if let Err(e) = flush_batch(&store, &batch).await {
                        error!("❌ Failed to flush batch of {} records: {}", batch.len(), e);
                    } else if batch.len() > 10 {
                        info!("💾 Flushed {} records to DB", batch.len());
                    }
                    batch.clear();
                }
            }
        });

        Self { sender: tx }
    }

    pub async fn push(&self, event: DbEvent) {
        if let Err(e) = self.sender.send(event).await {
            error!("Failed to enqueue DB event: {}", e);
        }
    }
}

async fn flush_batch(store: &Store, batch: &[DbEvent]) -> Result<(), String> {
    // Start transaction
    let mut tx = store.begin().await?;

    for event in batch {
        match event {
            DbEvent::Result {
                job_id,
                layout,
                score,
                node_id,
            } => {
                sqlx::query(
                    "INSERT INTO results (job_id, layout, score, node_id) VALUES (?, ?, ?, ?)",
                )
                .bind(job_id)
                .bind(layout)
                .bind(score)
                .bind(node_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| e.to_string())?;
            }
        }
    }

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}
