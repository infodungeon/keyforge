use anyhow::Result;
use ed25519_dalek::VerifyingKey;
use sha2::{Digest, Sha256};
use tokio::sync::broadcast;
use tracing::info;

use crate::identity::load_or_create_identity;
use crate::models::AgentConfig;

/// Starts the agent in worker mode, connecting to the Hive.
///
/// # Errors
///
/// Returns an error if identity loading or signal registration fails.
pub async fn run(config: AgentConfig) -> Result<()> {
    let signing_key = load_or_create_identity(&config.system)?;
    let public_key = VerifyingKey::from(&signing_key);

    let mut hasher = Sha256::new();
    hasher.update(public_key.to_bytes());
    let pk_hash = hex::encode(hasher.finalize());
    let node_id = format!("{}{}", config.system.node_id_prefix, &pk_hash[0..8]);

    info!("agent starting in WORKER mode");
    info!(hive_url = %config.hive_url, "connecting to hive");
    info!(data_dir = ?config.data_dir, "data directory configured");

    let (tx, rx) = broadcast::channel(config.system.shutdown_channel_capacity);
    #[cfg(unix)]
    let mut sig_usr1 =
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::user_defined1())
            .map_err(|e| anyhow::anyhow!("failed to register SIGUSR1: {e}"))?;
    #[cfg(not(unix))]
    let mut sig_usr1 = std::future::pending::<()>();

    let tx_clone = tx.clone();
    tokio::spawn(async move {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                let _ = tx_clone.send(());
            }
            _ = sig_usr1.recv() => {
                let _ = tx_clone.send(());
            }
        }
    });

    crate::agent::run_worker(config, node_id, signing_key, rx).await;
    Ok(())
}
