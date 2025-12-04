// ===== keyforge/crates/keyforge-hive/src/queue.rs =====
use crate::store::Store;
use sqlx::{Postgres, QueryBuilder}; // FIXED: Postgres
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

        tokio::spawn(async move {
            let mut batch = Vec::with_capacity(100);
            let mut shutdown_signal: Option<oneshot::Sender<()>> = None;

            loop {
                let first = rx.recv().await;
                if first.is_none() {
                    break;
                }
                let msg = first.unwrap();

                if let DbEvent::Shutdown(signal) = msg {
                    shutdown_signal = Some(signal);
                    break;
                }
                batch.push(msg);

                let mut stop = false;
                while batch.len() < 100 {
                    match rx.try_recv() {
                        Ok(DbEvent::Shutdown(signal)) => {
                            shutdown_signal = Some(signal);
                            stop = true;
                            break;
                        }
                        Ok(item) => batch.push(item),
                        Err(_) => break,
                    }
                }

                if !batch.is_empty() {
                    if let Err(e) = flush_batch(&store, &batch).await {
                        error!("❌ Failed to flush batch: {}", e);
                    } else if batch.len() > 10 {
                        info!("💾 Flushed {} records to DB", batch.len());
                    }
                    batch.clear();
                }
                if stop {
                    break;
                }
            }

            if !batch.is_empty() {
                info!("🛑 Shutdown: Flushing final {} records...", batch.len());
                let _ = flush_batch(&store, &batch).await;
            }
            if let Some(s) = shutdown_signal {
                let _ = s.send(());
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

async fn flush_batch(store: &Store, batch: &[DbEvent]) -> Result<(), String> {
    let results: Vec<_> = batch
        .iter()
        .filter_map(|e| {
            if let DbEvent::Result {
                job_id,
                layout,
                score,
                node_id,
            } = e
            {
                Some((job_id, layout, score, node_id))
            } else {
                None
            }
        })
        .collect();

    if results.is_empty() {
        return Ok(());
    }

    // FIXED: Use QueryBuilder<Postgres>
    let mut query_builder: QueryBuilder<Postgres> =
        QueryBuilder::new("INSERT INTO results (job_id, layout, score, node_id) ");

    query_builder.push_values(results, |mut b, (job_id, layout, score, node_id)| {
        b.push_bind(job_id)
            .push_bind(layout)
            .push_bind(*score as f64) // Postgres uses f64 for floats usually
            .push_bind(node_id);
    });

    let query = query_builder.build();
    query
        .execute(&store.db)
        .await
        .map_err(|e| format!("Batch Insert Error: {}", e))?;

    Ok(())
}
