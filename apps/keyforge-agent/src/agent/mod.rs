// apps/keyforge-agent/src/agent/mod.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.


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

/// JIT calibration for CPU-bound scoring tasks.
pub mod calibration;
/// Optimization engine orchestration and job execution.
pub mod compute;
/// Agent-specific error types and results.
pub mod errors;
/// System maintenance and garbage collection.
pub mod maintenance;
/// Secure WebSocket and HTTP communication with the Hive.
pub mod network;
/// Real-time telemetry and metrics reporting.
pub mod telemetry;
/// Cryptographic primitives for agent identity and signing.
pub mod crypto;

use self::errors::AgentResult;
use keyforge_infra::net::client::ClientConfig;

/// The main Agent coordinator that orchestrates optimization jobs.
pub struct Agent {
    config: AgentConfig,
    telemetry: SharedTelemetry,
    assets: Arc<keyforge_infra::AssetManager>, // Wrapped in Arc for sharing
    result_tx: mpsc::Sender<ResultSubmission>,
}

impl Agent {
    /// Creates a new Agent instance with the given configuration.
    pub async fn new(config: AgentConfig, result_tx: mpsc::Sender<ResultSubmission>) -> AgentResult<Self> {
        let telemetry = SharedTelemetry::default();
        
        let client_config = ClientConfig {
            base_url: config.hive_url.clone(),
            secret: Some(config.secret.clone()),
            ..Default::default()
        };
        
        let client = keyforge_infra::HiveClient::new(client_config)
            .map_err(|e| errors::AgentError::Internal(format!("Failed to create hive client: {}", e)))?;
            
        let assets = keyforge_infra::AssetManager::new(client, config.data_dir.clone());

        if let Err(e) = calibration::calibrate(&assets, &config.data_dir).await {
            tracing::error!("Calibration failed: {}. Using safe default.", e);
        }

        Ok(Self { 
            config, 
            telemetry, 
            assets: Arc::new(assets), // Arc wrapping
            result_tx 
        })
    }

    /// Starts the agent's job processing loop.
    pub async fn run(
        &self, 
        mut job_rx: mpsc::Receiver<(String, JobConfig)>,
        mut stop_rx: mpsc::Receiver<()>
    ) -> AgentResult<()> {
        info!("🤖 KeyForge Agent v0.8.0 Starting...");
        info!("   Node ID: {}", self.config.node_id);
        info!("   Capacity: {} Cores", self.config.cores);

        // Global Semaphore for Core Management
        let global_semaphore = Arc::new(Semaphore::new(self.config.cores));
        
        let mut running_jobs: HashMap<String, Arc<std::sync::atomic::AtomicBool>> = HashMap::new();

        loop {
            tokio::select! {
                // 1. New Job Arrival
                Some((job_id, job)) = job_rx.recv() => {
                    info!("⚙️  Queued Job (ID: {})...", job_id);
                    
                    let stop_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
                    running_jobs.insert(job_id.clone(), stop_flag.clone());

                    let assets = self.assets.clone(); // Cheap Arc clone
                    let config_compute = self.config.compute.clone();
                    let config_system = self.config.system.clone();
                    let config_telemetry = self.config.telemetry.clone();
                    let data_dir = self.config.data_dir.clone();
                    let result_tx = self.result_tx.clone();
                    let telemetry = self.telemetry.clone();
                    let node_id = self.config.node_id.clone();
                    let semaphore = global_semaphore.clone();
                    
                    // Determine cores needed (Assume 1 for now)
                    let permits_needed = 1; 

                    tokio::spawn(async move {
                        // 1. Acquire Resources (Wait if full)
                        let _permit = match semaphore.acquire_many(permits_needed).await {
                            Ok(p) => p,
                            Err(_) => return, // Semaphore closed
                        };

                        info!("🚀 Starting Job {} (Acquired resources)", job_id);
                        telemetry.set_job_id(&job_id);

                        // 2. Sync Assets
                        let (cost_file, _corpus_dir) = match compute::prepare_assets(&*assets, &job, &config_compute).await {
                            Ok(res) => res,
                            Err(e) => {
                                error!("Job {} Asset sync failed: {}", job_id, e);
                                return;
                            }
                        };

                        // 3. Prepare
                        let loader = Box::new(keyforge_infra::FsProvider::new(data_dir.clone()));
                        let prepared_job = match compute::create_engine_request(
                            loader, 
                            data_dir, 
                            &job, 
                            &cost_file, 
                            &config_system.corpora_dir_name, 
                            &config_compute
                        ).await {
                            Ok(pj) => pj,
                            Err(e) => {
                                error!("Job {} Preparation failed: {}", job_id, e);
                                return;
                            }
                        };

                        // 4. Run
                        let inner_limiter = Arc::new(Semaphore::new(1));

                        match compute::run_optimization(
                            prepared_job.req,
                            job_id.clone(),
                            stop_flag,
                            inner_limiter,
                            telemetry.clone(),
                            config_compute.job_timeout_sec,
                            config_telemetry.progress_log_sampling_rate
                        ).await {
                            Ok(opt_res) => {
                                info!("✅ Job {} Completed. Score: {:.4}", job_id, opt_res.score);
                                let layout_str = serde_json::to_string(&opt_res.layout).unwrap_or_default();
                                let submission = ResultSubmission {
                                    version: 1,
                                    job_id: job_id.clone(),
                                    layout: layout_str,
                                    score: opt_res.score,
                                    node_id,
                                    timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
                                    nonce: rand::random(),
                                    signature: None, 
                                };
                                if let Err(e) = result_tx.send(submission).await {
                                    error!("Failed to queue result for {}: {}", job_id, e);
                                }
                            }
                            Err(e) => {
                                warn!("❌ Job {} Terminated: {}", job_id, e);
                            }
                        }
                        
                        // Drop permit automatically here
                        telemetry.set_job_id(&config_system.idle_job_id);
                    });
                }

                // 2. Stop Signal (Cancellation)
                _ = stop_rx.recv() => {
                    info!("🛑 Global Stop Signal Received. Cancelling {} jobs...", running_jobs.len());
                    for (jid, flag) in running_jobs.iter() {
                        info!("   Cancelling {}", jid);
                        flag.store(true, std::sync::atomic::Ordering::SeqCst);
                    }
                    running_jobs.clear();
                }
            }
        }
    }
}

/// The primary entry point for starting a KeyForge worker agent.
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
