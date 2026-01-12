// apps/keyforge-agent/src/agent/mod.rs

//! Core logic for the KeyForge Agent.

use crate::models::{AgentConfig, SharedTelemetry};
use crate::agent::network::NetworkManager;
use keyforge_protocol::{JobConfig, ResultSubmission};
use std::sync::Arc;
use tokio::sync::{mpsc, Semaphore};
use tracing::{error, info, warn};
use ed25519_dalek::SigningKey;
use tokio::sync::broadcast;
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap;

pub mod calibration;
pub mod compute;
pub mod errors;
pub mod maintenance;
pub mod network;
pub mod telemetry;
pub mod crypto;

use self::errors::AgentResult;
use keyforge_infra::net::client::ClientConfig;

#[derive(Debug)]
pub struct Agent {
    config: AgentConfig,
    telemetry: SharedTelemetry,
    assets: Arc<keyforge_infra::AssetManager>, 
    result_tx: mpsc::Sender<ResultSubmission>,
}

impl Agent {
    pub async fn new(config: AgentConfig, result_tx: mpsc::Sender<ResultSubmission>) -> AgentResult<Self> {
        let telemetry = SharedTelemetry::default();
        
        let client_config = ClientConfig {
            api_url: config.hive_url.clone(),
            asset_url: config.asset_url.clone(), // Pass through
            secret: Some(config.secret.clone()),
            ..Default::default()
        };
        
        let client = keyforge_infra::HiveClient::new(client_config)
            .map_err(|e| errors::AgentError::Internal(format!("Failed to create hive client: {}", e)))?;
            
        let assets = keyforge_infra::AssetManager::new(client, config.data_dir.clone());

        if let Err(e) = calibration::calibrate(&assets, &config.data_dir, &config.calibration).await {
            tracing::error!("Calibration failed: {}. Using safe default.", e);
        }

        Ok(Self { 
            config, 
            telemetry, 
            assets: Arc::new(assets), 
            result_tx 
        })
    }

    pub async fn run(
        &self, 
        mut job_rx: mpsc::Receiver<(String, JobConfig)>,
        mut stop_rx: mpsc::Receiver<()>
    ) -> AgentResult<()> {
        info!("🤖 KeyForge Agent v0.9.0 Starting...");
        info!("   Node ID: {}", self.config.node_id);
        info!("   Capacity: {} Cores", self.config.cores);

        let global_semaphore = Arc::new(Semaphore::new(self.config.cores));
        let mut running_jobs: HashMap<String, Arc<std::sync::atomic::AtomicBool>> = HashMap::new();

        loop {
            tokio::select! {
                msg = job_rx.recv() => {
                    let (job_id, job) = match msg {
                        Some(j) => j,
                        None => {
                            info!("Job queue closed. Exiting agent loop.");
                            break Ok(());
                        }
                    };
                    info!("⚙️  Queued Job (ID: {})...", job_id);
                    
                    let stop_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
                    running_jobs.insert(job_id.clone(), stop_flag.clone());

                    let assets = self.assets.clone();
                    let config_compute = self.config.compute.clone();
                    let config_system = self.config.system.clone();
                    let config_telemetry = self.config.telemetry.clone();
                    let data_dir = self.config.data_dir.clone();
                    let result_tx = self.result_tx.clone();
                    let telemetry = self.telemetry.clone();
                    let node_id = self.config.node_id.clone();
                    let private_key = self.config.private_key.clone();
                    let semaphore = global_semaphore.clone();
                    let permits_needed = 1; 

                    tokio::spawn(async move {
                        let _permit = match semaphore.acquire_many(permits_needed).await {
                            Ok(p) => p,
                            Err(_) => return,
                        };

                        info!("🚀 Starting Job {} (Acquired resources)", job_id);
                        telemetry.set_job_id(&job_id);

                        let _ = compute::prepare_assets(&*assets, &job, &config_compute).await;

                        let loader = keyforge_infra::FsProvider::new(data_dir.clone());
                        let options = keyforge_runner::RunnerOptions {
                            timeout_sec: config_compute.job_timeout_sec,
                            log_sampling_rate: config_telemetry.progress_log_sampling_rate,
                            keycodes_file: config_compute.keycodes_file.clone(),
                            ..Default::default()
                        };

                        let prepared_session = match keyforge_runner::OptimizationRunner::prepare_session(
                            &loader, 
                            &job, 
                            &options
                        ).await {
                            Ok(pj) => pj,
                            Err(e) => {
                                error!("Job {} Preparation failed: {}", job_id, e);
                                return;
                            }
                        };

                        let inner_limiter = Arc::new(Semaphore::new(1));

                        match compute::run_optimization(
                            prepared_session,
                            job_id.clone(),
                            stop_flag,
                            inner_limiter,
                            telemetry.clone(),
                            config_compute.job_timeout_sec,
                            config_telemetry.progress_log_sampling_rate,
                            &job
                        ).await {
                            Ok(opt_res) => {
                                info!("✅ Job {} Completed. Score: {:.4}", job_id, opt_res.score);
                                let layout_str = serde_json::to_string(&opt_res.layout).unwrap_or_default();
                                
                                let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
                                let nonce = rand::random();

                                // Sign immediately (Secure by Construction)
                                let signature = match crypto::sign_result_direct(
                                    &private_key,
                                    &job_id,
                                    &layout_str,
                                    opt_res.score,
                                    timestamp,
                                    nonce
                                ) {
                                    Ok(sig) => sig,
                                    Err(e) => {
                                        error!("Failed to sign result for {}: {}", job_id, e);
                                        return;
                                    }
                                };

                                let submission = ResultSubmission {
                                    version: 1,
                                    job_id: job_id.clone(),
                                    layout: layout_str,
                                    score: opt_res.score,
                                    node_id,
                                    timestamp,
                                    nonce,
                                    signature, 
                                };
                                if let Err(e) = result_tx.send(submission).await {
                                    error!("Failed to queue result for {}: {}", job_id, e);
                                }
                            }
                            Err(e) => {
                                warn!("❌ Job {} Terminated: {}", job_id, e);
                            }
                        }
                        telemetry.set_job_id(&config_system.idle_job_id);
                    });
                }
                msg = stop_rx.recv() => {
                    match msg {
                        Some(_) => {
                            info!("🛑 Global Stop Signal Received. Cancelling {} jobs...", running_jobs.len());
                            for (jid, flag) in running_jobs.iter() {
                                info!("   Cancelling {}", jid);
                                flag.store(true, std::sync::atomic::Ordering::SeqCst);
                            }
                            running_jobs.clear();
                        }
                        None => {
                            info!("Example: Stop channel closed. Exiting agent loop.");
                            break Ok(());
                        }
                    }
                }
            }
        }
    }
}

pub async fn run_worker(
    mut config: AgentConfig,
    node_id: String,
    signing_key: SigningKey,
    mut shutdown_rx: broadcast::Receiver<()>,
) {
    config.node_id = node_id;
    config.private_key = hex::encode(signing_key.to_bytes());

    let (result_tx, result_rx) = mpsc::channel(config.system.result_channel_capacity);

    let agent = match Agent::new(config.clone(), result_tx).await {
        Ok(a) => a,
        Err(e) => {
            error!("Failed to initialize agent: {}", e);
            return;
        }
    };
    
    let (job_tx, job_rx) = mpsc::channel(100); 
    let (stop_tx, stop_rx) = mpsc::channel(100);

    let net = match NetworkManager::new(
        config,
        agent.telemetry.clone(),
        job_tx,
        result_rx,
        stop_tx,
    ) {
        Ok(n) => n,
        Err(e) => {
            error!("Failed to initialize network manager: {}", e);
            return;
        }
    };
    
    let net_handle = tokio::spawn(net.run());
    let agent_handle = tokio::spawn(async move {
        if let Err(e) = agent.run(job_rx, stop_rx).await {
            error!("Agent run error: {}", e);
        }
    });

    tokio::select! {
        _ = shutdown_rx.recv() => {
            info!("Shutdown signal received");
        }
    }
    
    net_handle.abort();
    agent_handle.abort();
}
