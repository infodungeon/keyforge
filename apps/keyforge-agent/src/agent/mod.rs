// apps/keyforge-agent/src/agent/mod.rs

//! Core logic for the KeyForge Agent.

use crate::agent::network::NetworkManager;
use crate::models::{AgentConfig, SharedTelemetry};
use ed25519_dalek::SigningKey;
use keyforge_protocol::{JobConfig, ResultSubmission};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::broadcast;
use tokio::sync::{mpsc, Semaphore};
use tracing::{error, info, warn};

pub mod calibration;
pub mod compute;
pub mod crypto;
pub mod errors;
pub mod maintenance;
pub mod network;
pub mod telemetry;

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
    pub async fn new(
        config: AgentConfig,
        result_tx: mpsc::Sender<ResultSubmission>,
    ) -> AgentResult<Self> {
        let telemetry = SharedTelemetry::default();

        let client_config = ClientConfig {
            api_url: config.hive_url.clone(),
            asset_url: config.asset_url.clone(), // Pass through
            secret: Some(config.secret.clone()),
            ..Default::default()
        };

        let client = keyforge_infra::HiveClient::new(client_config).map_err(|e| {
            errors::AgentError::Internal(format!("Failed to create hive client: {e}"))
        })?;

        let assets = keyforge_infra::AssetManager::new(client, config.data_dir.clone());

        if let Err(e) = calibration::calibrate(&assets, &config.data_dir, &config.calibration).await
        {
            tracing::error!("Calibration failed: {}. Using safe default.", e);
        }

        Ok(Self {
            config,
            telemetry,
            assets: Arc::new(assets),
            result_tx,
        })
    }

    #[allow(clippy::too_many_lines)]
    pub async fn run(
        &self,
        mut job_rx: mpsc::Receiver<(String, JobConfig)>,
        mut stop_rx: mpsc::Receiver<()>,
    ) -> AgentResult<()> {
        info!("🤖 KeyForge Agent v0.9.0 Starting...");
        info!("   Node ID: {}", self.config.node_id);
        info!("   Capacity: {} Cores", self.config.cores);

        let global_semaphore = Arc::new(Semaphore::new(self.config.cores));
        let mut running_jobs: HashMap<String, Arc<std::sync::atomic::AtomicBool>> = HashMap::new();

        loop {
            tokio::select! {
                msg = job_rx.recv() => {
                    let Some((job_id, job)) = msg else {
                        info!("Job queue closed. Exiting agent loop.");
                        break Ok(());
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
                        let Ok(_permit) = semaphore.acquire_many(permits_needed).await else { return };

                        info!("🚀 Starting Job {} (Acquired resources)", job_id);
                        telemetry.set_job_id(&job_id);

                        if let Err(e) = compute::prepare_assets(&*assets, &job, &config_compute).await {
                            error!("Job {} Asset preparation failed: {}", job_id, e);
                            return;
                        }

                        let loader = keyforge_infra::FsProvider::new(data_dir.clone());
                        let mut builder = keyforge_compute::SessionBuilder::new(&loader);

                        builder = builder.with_keyboard_def(Arc::new(job.definition.clone()));

                        builder = match builder.with_corpus(&job.corpora).await {
                            Ok(b) => b,
                            Err(e) => {
                                error!("Job {} Corpus load failed: {}", job_id, e);
                                return;
                            }
                        };

                        builder = match builder.with_cost_matrix(&job.cost_matrix).await {
                            Ok(b) => b,
                            Err(e) => {
                                error!("Job {} Cost matrix load failed: {}", job_id, e);
                                return;
                            }
                        };

                        builder = match builder.with_keycodes(&config_compute.keycodes_file).await {
                            Ok(b) => b,
                            Err(e) => {
                                error!("Job {} Keycodes load failed: {}", job_id, e);
                                return;
                            }
                        };

                        let builder = builder
                            .with_rubric(keyforge_adapter::conversion::to_domain_rubric(&job.weights))
                            .with_biometrics(job.biometrics.clone())
                            .with_config(keyforge_model::SearchConfig::Annealing {
                                steps: job.params.get_search_steps(),
                                start_temp: job.params.get_temp_max(),
                                end_temp: job.params.get_temp_min(),
                                seed: job.params.seed.unwrap_or(42),
                                patience: job.params.get_search_patience(),
                                reheats: job.params.get_reheats(),
                                reheat_factor: job.params.get_reheat_factor(),
                                include_thumbs: job.params.include_thumbs,
                            });

                        let prepared_session = match builder.build() {
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
                                    opt_res.raw_score,
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
                                    raw_score: opt_res.raw_score,
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
                    if let Some(()) = msg {
                        info!("🛑 Global Stop Signal Received. Cancelling {} jobs...", running_jobs.len());
                        for (jid, flag) in &running_jobs {
                            info!("   Cancelling {}", jid);
                            flag.store(true, std::sync::atomic::Ordering::SeqCst);
                        }
                        running_jobs.clear();
                    } else {
                        info!("Example: Stop channel closed. Exiting agent loop.");
                        break Ok(());
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

    let net = match NetworkManager::new(config, agent.telemetry.clone(), job_tx, result_rx, stop_tx)
    {
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
