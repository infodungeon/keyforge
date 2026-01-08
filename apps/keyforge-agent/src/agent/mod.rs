// apps/keyforge-agent/src/agent/mod.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
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
use keyforge_protocol::JobConfig;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info};
use std::path::PathBuf;
use ed25519_dalek::SigningKey;
use tokio::sync::broadcast;

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
pub struct Agent {
    config: AgentConfig,
    telemetry: SharedTelemetry,
}

impl Agent {
    /// Creates a new `Agent` with the given configuration.
    pub async fn new(config: AgentConfig) -> Self {
        let telemetry = SharedTelemetry::default();
        Self { config, telemetry }
    }

    /// Starts the agent's job processing loop.
    ///
    /// It listens for `JobConfig` messages from the network layer and coordinates
    /// the execution of optimization tasks.
    pub async fn run(&self, mut job_rx: mpsc::Receiver<JobConfig>) -> AgentResult<()> {
        info!("🤖 KeyForge Agent v0.8.0 Starting...");
        info!("   Node ID: {}", self.config.node_id);

        // Job Loop
        while let Some(job) = job_rx.recv().await {
            info!("⚙️  Starting Job...");
            
            let job_id = "job-pending-id".to_string(); 
            self.telemetry.set_job_id(&job_id);

            // FIX: Construct a valid EngineRequest.
            let keyboard = keyforge_model::Keyboard::new(vec![keyforge_model::KeyNode::default()], 0)
                .map_err(|e| errors::AgentError::Internal(e.to_string()))?;
                
            let corpus = keyforge_model::Corpus::default();
            let rubric = keyforge_model::Rubric::default();
            
            let search_config = keyforge_adapter::conversion::to_domain_config(&job.params, 42);

            let req = keyforge_core::EngineRequest {
                keyboard: Arc::new(keyboard),
                corpus: Arc::new(corpus),
                rubric: Arc::new(rubric),
                config: search_config,
                initial_layout: None,
                pinned_keys: vec![],
                cost_overrides: vec![],
            };

            let limiter = Arc::new(tokio::sync::Semaphore::new(1));

            let _result = compute::run_optimization(
                req,
                job_id,
                Arc::new(std::sync::atomic::AtomicBool::new(false)), // Placeholder for stop flag
                limiter,
                self.telemetry.clone() 
            ).await;
            
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
    hive_url: String,
    node_id: String,
    secret: Option<String>,
    signing_key: SigningKey,
    data_dir: PathBuf,
    mut shutdown_rx: broadcast::Receiver<()>,
    cores: usize,
) {
    // Construct Configuration
    let config = AgentConfig {
        hive_url,
        node_id,
        secret: secret.unwrap_or_default(),
        private_key: hex::encode(signing_key.to_bytes()),
        data_dir,
        cores,
    };

    let agent = Agent::new(config.clone()).await;
    
    let (job_tx, job_rx) = mpsc::channel(1);
    let (stop_tx, _stop_rx) = mpsc::channel(1);

    // Network Task
    let net = NetworkManager::new(
        config,
        agent.telemetry.clone(),
        job_tx,
        stop_tx,
    );
    
    let net_handle = tokio::spawn(net.run());
    let agent_handle = tokio::spawn(async move {
        if let Err(e) = agent.run(job_rx).await {
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
