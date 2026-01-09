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
//!
//! The Agent is responsible for receiving optimization jobs from the Hive,
//! executing them, and reporting back the results.


use crate::models::{AgentConfig, SharedTelemetry};
use crate::agent::network::NetworkManager;
use keyforge_protocol::{JobConfig, ResultSubmission};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info};
use ed25519_dalek::SigningKey;
use tokio::sync::broadcast;
use std::time::{SystemTime, UNIX_EPOCH};

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
pub mod crypto; // Re-export crypto for network module

use self::errors::AgentResult;

/// The main Agent coordinator that orchestrates optimization jobs.
use keyforge_infra::net::client::ClientConfig;

/// The main Agent coordinator that orchestrates optimization jobs.
pub struct Agent {
    config: AgentConfig,
    telemetry: SharedTelemetry,
    assets: keyforge_infra::AssetManager,
    result_tx: mpsc::Sender<ResultSubmission>,
}

impl Agent {
    /// Creates a new  with the given configuration.
    pub async fn new(config: AgentConfig, result_tx: mpsc::Sender<ResultSubmission>) -> AgentResult<Self> {
        let telemetry = SharedTelemetry::default();
        
        // Initialize Hive Client for asset downloads
        let client_config = ClientConfig {
            base_url: config.hive_url.clone(),
            secret: Some(config.secret.clone()),
            ..Default::default()
        };
        
        let client = keyforge_infra::HiveClient::new(client_config)
            .map_err(|e| errors::AgentError::Internal(format!("Failed to create hive client: {}", e)))?;
            
        let assets = keyforge_infra::AssetManager::new(client, config.data_dir.clone());

        Ok(Self { config, telemetry, assets, result_tx })
    }

    /// Starts the agent's job processing loop.
    ///
    /// It listens for  messages from the network layer and coordinates
    /// the execution of optimization tasks.
    pub async fn run(
        &self, 
        mut job_rx: mpsc::Receiver<(String, JobConfig)>,
        mut stop_rx: mpsc::Receiver<()>
    ) -> AgentResult<()> {
        info!("🤖 KeyForge Agent v0.8.0 Starting...");
        info!("   Node ID: {}", self.config.node_id);

        // Job Loop
        loop {
            let (job_id, job) = tokio::select! {
                Some((id, j)) = job_rx.recv() => (id, j),
                _ = stop_rx.recv() => {
                    info!("Received stop signal while idle.");
                    continue;
                }
                else => break, // Channel closed
            };

            info!("⚙️  Starting Job (ID: {})...", job_id);
            self.telemetry.set_job_id(&job_id);

            // 1. Sync Assets
            info!("   Syncing assets...");
            let (cost_file, _corpus_dir) = match compute::prepare_assets(&self.assets, &job).await {
                Ok(res) => res,
                Err(e) => {
                    error!("Asset sync failed: {}", e);
                    self.telemetry.set_job_id("idle");
                    continue;
                }
            };

            // 2. Prepare Engine Request
            let loader = Box::new(keyforge_infra::FsProvider::new(self.config.data_dir.clone()));
            let prepared_job = match compute::create_engine_request(
                loader, 
                self.config.data_dir.clone(), 
                &job, 
                &cost_file, 
                "corpora" // Standard corpora dir
            ).await {
                Ok(pj) => pj,
                Err(e) => {
                    error!("Failed to prepare job: {}", e);
                    self.telemetry.set_job_id("idle");
                    continue;
                }
            };

            // 3. Execution (with Cancellation Support)
            let stop_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
            let limiter = Arc::new(tokio::sync::Semaphore::new(self.config.cores.max(1))); // Per-job limiter for now

            info!("   Launching optimization Engine...");
            tokio::select! {
                res = compute::run_optimization(
                    prepared_job.req,
                    job_id.clone(),
                    stop_flag.clone(),
                    limiter,
                    self.telemetry.clone()
                ) => {
                    match res {
                        Ok(opt_res) => {
                            info!("✅ Job {} Completed. Score: {:.4}", job_id, opt_res.score);
                            
                            // Serialize layout to JSON string for protocol
                            let layout_str = serde_json::to_string(&opt_res.layout).unwrap_or_default();

                            let submission = ResultSubmission {
                                version: 1,
                                job_id: job_id.clone(),
                                layout: layout_str,
                                score: opt_res.score,
                                node_id: self.config.node_id.clone(),
                                timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
                                nonce: rand::random(),
                                signature: None, // Will be signed by NetworkManager
                            };
                            
                            if let Err(e) = self.result_tx.send(submission).await {
                                error!("Failed to queue result for submission: {}", e);
                            }
                        }
                        Err(e) => {
                            error!("❌ Job {} Failed: {}", job_id, e);
                        }
                    }
                }
                _ = stop_rx.recv() => {
                    info!("🛑 Cancellation received for job {}", job_id);
                    stop_flag.store(true, std::sync::atomic::Ordering::SeqCst);
                }
            }
            
            self.telemetry.set_job_id("idle");
        }

        Ok(())
    }
}

/// The primary entry point for starting a KeyForge worker agent.
///
/// This function initializes the agent, sets up networking and telemetry,
/// and starts the job processing loop. It runs until a shutdown signal is received.
pub async fn run_worker(
    mut config: AgentConfig,
    node_id: String,
    signing_key: SigningKey,
    mut shutdown_rx: broadcast::Receiver<()>,
) {
    // Inject Identity
    config.node_id = node_id;
    config.private_key = hex::encode(signing_key.to_bytes());

    let (result_tx, result_rx) = mpsc::channel(100);

    let agent = match Agent::new(config.clone(), result_tx).await {
        Ok(a) => a,
        Err(e) => {
            error!("Failed to initialize agent: {}", e);
            return;
        }
    };
    
    let (job_tx, job_rx) = mpsc::channel(1);
    let (stop_tx, stop_rx) = mpsc::channel(1);

    // Network Task
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

    // Wait for shutdown signal
    tokio::select! {
        _ = shutdown_rx.recv() => {
            info!("Shutdown signal received");
        }
    }
    
    // Cleanup
    net_handle.abort();
    agent_handle.abort();
}
