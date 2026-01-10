// apps/keyforge-agent/src/agent/network.rs

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


use crate::agent::errors::AgentResult;
use crate::models::{AgentConfig, SharedTelemetry};
use crate::agent::crypto;
use futures::{SinkExt, StreamExt};
use keyforge_protocol::{JobConfig, NodeTelemetry, ResultSubmission};
use reqwest::Client;
use std::time::{Duration, SystemTime, UNIX_EPOCH, Instant};
use tokio::sync::mpsc;
use tokio::time::interval;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{error, info, warn, debug};
use url::Url;
use serde::{Deserialize, Serialize};
use sysinfo::{System, ProcessesToUpdate};
use std::path::PathBuf;
use keyforge_infra::HiveClient;

/// Manages high-level network operations for the agent.
pub struct NetworkManager {
    client: Client,
    config: AgentConfig,
    telemetry: SharedTelemetry,
    job_tx: mpsc::Sender<(String, JobConfig)>,
    result_rx: mpsc::Receiver<ResultSubmission>,
    stop_tx: mpsc::Sender<()>,
    outbox: ResultOutbox,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type", content = "payload")]
enum ServerMessage {
    Job { 
        #[serde(rename = "id")]
        id: String 
    },
    Cancel { 
        #[serde(rename = "id")]
        id: String 
    },
}

/// A circuit breaker for network requests.
pub struct CircuitBreaker {
    failures: u32,
    threshold: u32,
    last_failure: Option<Instant>,
    cooldown: Duration,
}

impl CircuitBreaker {
    /// Creates a new `CircuitBreaker`.
    pub fn new(threshold: u32, cooldown_secs: u64) -> Self {
        Self {
            failures: 0,
            threshold,
            last_failure: None,
            cooldown: Duration::from_secs(cooldown_secs),
        }
    }

    /// Returns `true` if an attempt is allowed.
    pub fn can_attempt(&self) -> bool {
        if self.failures < self.threshold {
            return true;
        }
        if let Some(last) = self.last_failure {
            if last.elapsed() > self.cooldown {
                return true;
            }
        }
        false
    }

    /// Records a failure.
    pub fn record_failure(&mut self) {
        self.failures += 1;
        self.last_failure = Some(Instant::now());
    }

    /// Resets the failure counter.
    pub fn record_success(&mut self) {
        self.failures = 0;
        self.last_failure = None;
    }
}

/// A persistent outbox for result submissions.
pub struct ResultOutbox {
    _client: HiveClient,
    wal_dir: PathBuf,
    _breaker: CircuitBreaker,
}

impl ResultOutbox {
    /// Creates a new `ResultOutbox`.
    pub fn new(client: HiveClient, data_root: PathBuf, threshold: u32) -> Self {
        let wal_dir = data_root.join("user/agent_wal");
        std::fs::create_dir_all(&wal_dir).ok();
        Self {
            _client: client,
            wal_dir,
            _breaker: CircuitBreaker::new(threshold, 60),
        }
    }

    /// Buffers a result to disk.
    pub fn save_to_wal(&self, submission: &ResultSubmission) -> AgentResult<()> {
        let path = self.wal_dir.join(format!("{}.json", submission.nonce));
        if let Ok(json) = serde_json::to_string(submission) {
             if let Err(e) = std::fs::write(&path, json) {
                 error!("CRITICAL: Failed to write result to WAL at {:?}: {}", path, e);
                 return Err(crate::agent::errors::AgentError::Resource(e.to_string()));
             }
             info!("Buffered result {} to WAL", submission.job_id);
        }
        Ok(())
    }

    /// Retrieves all pending submissions from disk.
    pub fn get_pending(&self) -> Vec<(PathBuf, ResultSubmission)> {
        let mut pending = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&self.wal_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(sub) = serde_json::from_str::<ResultSubmission>(&content) {
                            pending.push((path, sub));
                        } else {
                            warn!("Deleting corrupt WAL file: {:?}", path);
                            let _ = std::fs::remove_file(path);
                        }
                    }
                }
            }
        }
        pending
    }

    /// Deletes a WAL file.
    pub fn delete(&self, path: &PathBuf) {
        if let Err(e) = std::fs::remove_file(path) {
            warn!("Failed to delete WAL file {:?}: {}", path, e);
        } else {
            debug!("Removed WAL file {:?}", path);
        }
    }
}

impl NetworkManager {
    /// Creates a new `NetworkManager`.
    pub fn new(
        config: AgentConfig,
        telemetry: SharedTelemetry,
        job_tx: mpsc::Sender<(String, JobConfig)>,
        result_rx: mpsc::Receiver<ResultSubmission>,
        stop_tx: mpsc::Sender<()>,
    ) -> AgentResult<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.network.timeout_seconds))
            .build()
            .map_err(|e| crate::agent::errors::AgentError::Internal(format!("Failed to build HTTP client: {}", e)))?;

        let hive_client = HiveClient::new(keyforge_infra::net::client::ClientConfig {
            base_url: config.hive_url.clone(),
            secret: Some(config.secret.clone()),
            ..Default::default()
        }).map_err(|e| crate::agent::errors::AgentError::Internal(format!("Failed to init outbox client: {}", e)))?;

        let outbox = ResultOutbox::new(
            hive_client, 
            config.data_dir.clone(), 
            config.network.circuit_breaker_threshold
        );

        Ok(Self {
            client,
            config,
            telemetry,
            job_tx,
            result_rx,
            stop_tx,
            outbox,
        })
    }

    /// Starts the main network event loop.
    pub async fn run(mut self) {
        let mut backoff = Duration::from_secs(self.config.network.initial_backoff_seconds); 
        loop {
            if let Err(e) = self.connect_and_loop().await {
                error!("🔌 Connection Lost: {}. Retrying in {:?}...", e, backoff);
                tokio::time::sleep(backoff).await;
                backoff = (backoff * 2).min(Duration::from_secs(self.config.network.max_backoff_seconds));
            } else {
                break;
            }
        }
    }

    // ... (rest of implementation unchanged, already fixed in previous steps)
    async fn connect_and_loop(&mut self) -> AgentResult<()> {
        let mut ws_url = Url::parse(&self.config.hive_url)
            .map_err(|e| crate::agent::errors::AgentError::Network(e.to_string()))?;
        
        if ws_url.scheme() == "http" {
            let _ = ws_url.set_scheme("ws");
        } else if ws_url.scheme() == "https" {
            let _ = ws_url.set_scheme("wss");
        }
        
        ws_url.set_path("ws");
        ws_url.query_pairs_mut().append_pair("node_id", &self.config.node_id);

        info!("🌐 Connecting to Hive: {}", ws_url);
        
        let (ws_stream, _) = connect_async(ws_url.to_string())
            .await
            .map_err(|e| crate::agent::errors::AgentError::Network(e.to_string()))?;
            
        let (mut write, mut read) = ws_stream.split();
        
        info!("✅ WebSocket Connected");

        let mut heartbeat = interval(Duration::from_secs(self.config.network.heartbeat_interval_seconds));
        let telemetry = self.telemetry.clone();
        
        let mut sys = System::new();
        let pid = sysinfo::get_current_pid().map_err(|e| crate::agent::errors::AgentError::Internal(e.to_string()))?;

        loop {
            tokio::select! {
                _ = heartbeat.tick() => {
                    sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);
                    let memory_usage = if let Some(process) = sys.process(pid) {
                        process.memory()
                    } else {
                        0
                    };

                    let (ips, temp, best) = telemetry.snapshot();
                    let job_id = telemetry.get_job_id();
                    
                    let job_id_opt = if job_id == self.config.system.idle_job_id { None } else { Some(job_id) };
                    let best_opt = if best == 0.0 { None } else { Some(best) };

                    let payload = NodeTelemetry {
                        job_id: job_id_opt,
                        ips,
                        temp,
                        current_best: best_opt,
                        memory_usage,
                        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
                    };
                    
                    if let Ok(json) = serde_json::to_string(&payload) {
                        if let Err(e) = write.send(Message::Text(json.into())).await {
                            return Err(crate::agent::errors::AgentError::Network(e.to_string()));
                        }
                    }

                    self.flush_wal().await;
                }
                msg = read.next() => {
                    match msg {
                        Some(Ok(Message::Text(txt))) => {
                            if let Ok(server_msg) = serde_json::from_str::<ServerMessage>(&txt) {
                                match server_msg {
                                    ServerMessage::Job { id } => {
                                        info!("Received Job Signal: {}", id);
                                        self.fetch_and_start_job(&id).await?;
                                    }
                                    ServerMessage::Cancel { id } => {
                                        warn!("🛑 Received Cancel Signal for {}", id);
                                        let _ = self.stop_tx.send(()).await;
                                    }
                                }
                            }
                        }
                        Some(Ok(Message::Close(_))) => return Err(crate::agent::errors::AgentError::Network("Server closed connection".into())),
                        Some(Err(e)) => return Err(crate::agent::errors::AgentError::Network(e.to_string())),
                        None => return Err(crate::agent::errors::AgentError::Network("Stream ended".into())),
                        _ => {}
                    }
                }
                Some(result) = self.result_rx.recv() => {
                    info!("📤 Submitting result for job {}", result.job_id);
                    if let Err(e) = self.submit_result(result).await {
                        error!("Failed to submit result: {}. Buffering to WAL.", e);
                    }
                }
            }
        }
    }

    async fn fetch_and_start_job(&self, _job_id: &str) -> AgentResult<()> {
        let url = format!("{}/jobs/queue", self.config.hive_url);
        let resp = self.client.get(&url)
            .header("X-Keyforge-Secret", &self.config.secret)
            .send()
            .await
            .map_err(|e| crate::agent::errors::AgentError::Network(e.to_string()))?;

        if resp.status().is_success() {
            let queue_resp: keyforge_protocol::JobQueueResponse = resp.json()
                .await
                .map_err(|e| crate::agent::errors::AgentError::Network(e.to_string()))?;
                
            if let (Some(jid), Some(config)) = (queue_resp.job_id, queue_resp.config) {
                 info!("📥 Acquired Job {}", jid);
                 let _ = self.job_tx.send((jid, config)).await;
            }
        }
        Ok(())
    }

    pub async fn submit_result(&self, result: ResultSubmission) -> AgentResult<()> {
        let url = format!("{}/results", self.config.hive_url);
        
        let mut signed_result = result.clone();

        if signed_result.signature.is_none() {
            let signature = crypto::sign_result_direct(
                &self.config.private_key,
                &result.job_id,
                &result.layout,
                result.score,
                result.timestamp,
                result.nonce
            )?;
            signed_result.signature = Some(signature);
        }

        let resp = self.client.post(&url)
            .header("X-Keyforge-Secret", &self.config.secret)
            .json(&signed_result)
            .send()
            .await;

        match resp {
            Ok(r) => {
                let status = r.status();
                if !status.is_success() {
                    let txt = r.text().await.unwrap_or_default();
                    error!("❌ Submission Rejected: {}", txt);
                    if status.is_server_error() {
                        self.outbox.save_to_wal(&signed_result)?;
                    }
                    return Err(crate::agent::errors::AgentError::Network(format!("Submission rejected: {}", txt)));
                }
                Ok(())
            }
            Err(e) => {
                error!("❌ Network Error during submission: {}", e);
                self.outbox.save_to_wal(&signed_result)?;
                Err(crate::agent::errors::AgentError::Network(e.to_string()))
            }
        }
    }

    async fn flush_wal(&self) {
        let pending = self.outbox.get_pending();
        if pending.is_empty() {
            return;
        }

        info!("🔄 WAL Flush: Attempting to resend {} pending submissions...", pending.len());

        for (path, submission) in pending {
            match self.submit_result(submission).await {
                Ok(_) => {
                    info!("✅ Resent successfully: {:?}", path.file_name());
                    self.outbox.delete(&path);
                }
                Err(e) => {
                    warn!("⚠️ Failed to resend {:?}: {}", path.file_name(), e);
                    break;
                }
            }
        }
    }
}
