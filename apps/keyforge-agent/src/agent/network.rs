// apps/keyforge-agent/src/agent/network.rs

use crate::agent::errors::AgentResult;
use crate::models::{AgentConfig, SharedTelemetry}; // Added SharedTelemetry
use crate::agent::crypto;
use futures::{SinkExt, StreamExt};
use keyforge_protocol::{JobConfig, NodeTelemetry, ResultSubmission, PROTOCOL_VERSION};
use reqwest::Client;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{mpsc, oneshot};
use tokio::time::interval;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{error, info, warn, debug};
use url::Url;
use serde::{Deserialize, Serialize}; // Added Serialize

pub struct NetworkManager {
    client: Client,
    config: AgentConfig,
    telemetry: SharedTelemetry,
    job_tx: mpsc::Sender<JobConfig>,
    stop_tx: mpsc::Sender<()>,
}

#[derive(Deserialize, Serialize, Debug)] // Added Serialize, Debug
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

impl NetworkManager {
    pub fn new(
        config: AgentConfig,
        telemetry: SharedTelemetry,
        job_tx: mpsc::Sender<JobConfig>,
        stop_tx: mpsc::Sender<()>,
    ) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap(),
            config,
            telemetry,
            job_tx,
            stop_tx,
        }
    }

    pub async fn run(self) {
        let mut backoff = Duration::from_secs(1);
        loop {
            if let Err(e) = self.connect_and_loop().await {
                error!("🔌 Connection Lost: {}. Retrying in {:?}...", e, backoff);
                tokio::time::sleep(backoff).await;
                backoff = (backoff * 2).min(Duration::from_secs(60));
            } else {
                // Graceful exit
                break;
            }
        }
    }

    async fn connect_and_loop(&self) -> AgentResult<()> {
        let ws_url = Url::parse(&self.config.hive_url)?
            .join(&format!("ws?node_id={}", self.config.node_id))?;

        info!("🌐 Connecting to Hive: {}", ws_url);
        
        let (ws_stream, _) = connect_async(ws_url.to_string()).await?;
        let (mut write, mut read) = ws_stream.split();
        
        info!("✅ WebSocket Connected");

        let mut heartbeat = interval(Duration::from_secs(15));
        let telemetry = self.telemetry.clone();
        let node_id = self.config.node_id.clone();

        loop {
            tokio::select! {
                _ = heartbeat.tick() => {
                    // Send Telemetry
                    let t = telemetry.snapshot();
                    let payload = NodeTelemetry {
                        job_id: t.job_id,
                        ips: t.ips,
                        temp: t.temp,
                        current_best: t.current_best,
                        memory_usage: t.memory_usage,
                        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
                    };
                    
                    if let Ok(json) = serde_json::to_string(&payload) {
                        // Fix: Tungstenite 0.28 requires Into::into() for String -> Utf8Bytes
                        if let Err(e) = write.send(Message::Text(json.into())).await {
                            return Err(e.into());
                        }
                    }
                }
                msg = read.next() => {
                    match msg {
                        Some(Ok(Message::Text(txt))) => {
                            // Fix: Utf8Bytes derefs to str
                            if let Ok(server_msg) = serde_json::from_str::<ServerMessage>(&txt) {
                                match server_msg {
                                    ServerMessage::Job { id } => {
                                        info!("📩 Received Job Signal: {}", id);
                                        self.fetch_and_start_job(&id).await?;
                                    }
                                    ServerMessage::Cancel { id } => {
                                        warn!("🛑 Received Cancel Signal for {}", id);
                                        let _ = self.stop_tx.send(()).await;
                                    }
                                }
                            }
                        }
                        Some(Ok(Message::Close(_))) => return Err("Server closed connection".into()),
                        Some(Err(e)) => return Err(e.into()),
                        None => return Err("Stream ended".into()),
                        _ => {}
                    }
                }
            }
        }
    }

    async fn fetch_and_start_job(&self, job_id: &str) -> AgentResult<()> {
        let url = format!("{}/jobs/{}/config", self.config.hive_url, job_id); // This endpoint needs to exist on server or be handled
        // Note: The server currently doesn't expose /jobs/:id/config publicly in the new API? 
        // Checking Hive... We have /jobs/queue (polling). 
        // The WebSocket signal is "Wake Up", logic implies we should poll queue?
        // OR we implement specific config fetch.
        // For now, let's assume we hit the queue endpoint which returns the job.
        
        // Actually, previous logic was: Signal -> Poll Queue.
        // Let's implement that.
        
        let url = format!("{}/jobs/queue", self.config.hive_url);
        let resp = self.client.get(&url)
            .header("X-Keyforge-Secret", &self.config.secret)
            .send()
            .await?;

        if resp.status().is_success() {
            let queue_resp: keyforge_protocol::JobQueueResponse = resp.json().await?;
            if let Some(config) = queue_resp.config {
                // We have a job!
                let _ = self.job_tx.send(config).await;
            }
        }
        Ok(())
    }

    pub async fn submit_result(&self, result: ResultSubmission) -> AgentResult<()> {
        let url = format!("{}/results", self.config.hive_url);
        
        // Sign the result
        let signature = crypto::sign_result_direct(
            &self.config.private_key, // Assuming hex key in config
            &result.job_id,
            &result.layout,
            result.score,
            result.timestamp,
            result.nonce
        )?;

        let mut signed_result = result;
        signed_result.signature = Some(signature);

        let resp = self.client.post(&url)
            .header("X-Keyforge-Secret", &self.config.secret)
            .json(&signed_result)
            .send()
            .await?;

        if !resp.status().is_success() {
            let txt = resp.text().await.unwrap_or_default();
            error!("❌ Submission Failed: {}", txt);
            return Err(format!("Submission failed: {}", txt).into());
        }
        
        Ok(())
    }
}
