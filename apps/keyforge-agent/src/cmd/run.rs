use anyhow::Result;
use std::path::PathBuf;
use tokio::sync::mpsc;
use tracing::error;

use crate::config_loader::read_job_config;
use crate::models::AgentConfig;

/// Executes a local optimization job.
///
/// # Errors
///
/// Returns an error if the job configuration cannot be read or optimization fails.
///
/// # Panics
///
/// Panics if result serialization fails.
pub async fn run(mut config: AgentConfig, job_file: PathBuf, timeout: Option<u64>) -> Result<()> {
    let job = read_job_config(&job_file).await?;

    if let Some(t) = timeout {
        config.compute.job_timeout_sec = t;
    }

    let (result_tx, mut result_rx) = mpsc::channel(1);

    let agent = crate::agent::Agent::new(config.clone(), result_tx)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to init agent: {e}"))?;

    let (job_tx, job_rx) = mpsc::channel(1);
    let (_stop_tx, stop_rx) = mpsc::channel(1);

    let agent_handle = tokio::spawn(async move { agent.run(job_rx, stop_rx).await });

    let job_id = "local-job".to_string();
    job_tx.send((job_id, job)).await.ok();

    if let Some(result) = result_rx.recv().await {
        let json = serde_json::to_string(&result)
            .map_err(|e| anyhow::anyhow!("Failed to serialize result: {e}"))?;
        println!("{json}");
    } else {
        error!("No result produced!");
        std::process::exit(1);
    }

    drop(job_tx);
    let _ = agent_handle.await;
    Ok(())
}
