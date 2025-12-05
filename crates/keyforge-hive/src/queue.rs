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
    Shutdown(oneshot::Sender<()>),
}

pub struct WriteQueue {
    sender: mpsc::Sender<DbEvent>,
}

impl WriteQueue {
    pub fn new(store: Store, buffer_size: usize) -> Self {
        let (tx, mut rx) = mpsc::channel(buffer_size);

        // Spawn a background task to handle writes sequentially
        // This ensures DB connection pool isn't exhausted by thousands of concurrent inserts
        tokio::spawn(async move {
            info!("💾 WriteQueue started (Sequential Writer Mode)");

            while let Some(msg) = rx.recv().await {
                match msg {
                    DbEvent::Result {
                        job_id,
                        layout,
                        score,
                        node_id,
                    } => {
                        // Direct insert, no batching.
                        // Reliability > Micro-optimization at this stage.
                        if let Err(e) =
                            insert_result(&store, &job_id, &layout, score, &node_id).await
                        {
                            error!("❌ DB Insert Failed: {}", e);
                        }
                    }
                    DbEvent::Shutdown(signal) => {
                        info!("🛑 WriteQueue shutting down...");
                        // Process remaining items in channel (best effort)
                        while let Ok(msg) = rx.try_recv() {
                            if let DbEvent::Result {
                                job_id,
                                layout,
                                score,
                                node_id,
                            } = msg
                            {
                                let _ =
                                    insert_result(&store, &job_id, &layout, score, &node_id).await;
                            }
                        }
                        let _ = signal.send(());
                        break;
                    }
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

    pub async fn shutdown(&self) {
        let (tx, rx) = oneshot::channel();
        if self.sender.send(DbEvent::Shutdown(tx)).await.is_ok() {
            let _ = rx.await;
            info!("💾 WriteQueue shut down gracefully.");
        }
    }
}

async fn insert_result(
    store: &Store,
    job_id: &str,
    layout: &str,
    score: f32,
    node_id: &str,
) -> Result<(), String> {
    sqlx::query("INSERT INTO results (job_id, layout, score, node_id) VALUES ($1, $2, $3, $4)")
        .bind(job_id)
        .bind(layout)
        .bind(score as f64)
        .bind(node_id)
        .execute(&store.db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
