use super::outbox::ResultOutbox;
use crate::agent::errors::{AgentError, AgentResult};
use crate::models::{AgentConfig, SharedTelemetry};
use futures::{SinkExt, StreamExt};
use keyforge_infra::HiveClient;
use keyforge_protocol::{
    JobConfig, JobQueueResponse, NodeRequest, NodeResponse, NodeTelemetry, ResultSubmission,
    PROTOCOL_VERSION,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use sysinfo::{ProcessesToUpdate, System};
use tokio::sync::mpsc;
use tokio::time::interval;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{error, info, warn};
use url::Url;

#[derive(Debug)]
pub struct NetworkManager {
    client: Client,
    config: AgentConfig,
    telemetry: SharedTelemetry,
    hardware: crate::agent::hardware::HardwareInfo,
    performance_ips: f64,
    job_tx: mpsc::Sender<(String, JobConfig)>,
    result_rx: mpsc::Receiver<ResultSubmission>,
    stop_tx: mpsc::Sender<()>,
    outbox: ResultOutbox,
    session_token: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type", content = "payload")]
enum ServerMessage {
    Job {
        #[serde(rename = "id")]
        id: String,
    },
    Cancel {
        #[serde(rename = "id")]
        id: String,
    },
}

impl NetworkManager {
    pub fn new(
        config: AgentConfig,
        telemetry: SharedTelemetry,
        hardware: crate::agent::hardware::HardwareInfo,
        performance_ips: f64,
        job_tx: mpsc::Sender<(String, JobConfig)>,
        result_rx: mpsc::Receiver<ResultSubmission>,
        stop_tx: mpsc::Sender<()>,
    ) -> AgentResult<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.network.timeout_seconds))
            .build()
            .map_err(|e| AgentError::Internal(format!("Failed to build HTTP client: {e}")))?;

        let hive_client = HiveClient::new(keyforge_infra::net::client::ClientConfig {
            api_url: config.hive_url.clone(),
            asset_url: config.asset_url.clone(), // Passthrough
            secret: Some(config.secret.clone()),
            ..Default::default()
        })
        .map_err(|e| AgentError::Internal(format!("Failed to init outbox client: {e}")))?;

        let outbox = ResultOutbox::new(
            hive_client,
            &config.data_dir,
            config.network.circuit_breaker_threshold,
            config.network.circuit_breaker_cooldown,
        );

        Ok(Self {
            client,
            config,
            telemetry,
            hardware,
            performance_ips,
            job_tx,
            result_rx,
            stop_tx,
            outbox,
            session_token: None,
        })
    }

    pub async fn run(mut self) {
        let mut backoff = Duration::from_secs(self.config.network.initial_backoff_seconds);

        while let Err(e) = self.register_with_hive().await {
            error!(
                "🚨 Registration Failed: {}. Retrying in {:?}...",
                e, backoff
            );
            tokio::time::sleep(backoff).await;
            backoff =
                (backoff * 2).min(Duration::from_secs(self.config.network.max_backoff_seconds));
        }

        backoff = Duration::from_secs(self.config.network.initial_backoff_seconds);

        while let Err(e) = self.connect_and_loop().await {
            error!("🔌 Connection Lost: {}. Retrying in {:?}...", e, backoff);
            tokio::time::sleep(backoff).await;
            backoff =
                (backoff * 2).min(Duration::from_secs(self.config.network.max_backoff_seconds));
        }
    }

    async fn register_with_hive(&mut self) -> AgentResult<()> {
        info!("📝 Registering Node with Hive...");
        let url = format!("{}/nodes/register", self.config.hive_url);

        let req = NodeRequest {
            version: PROTOCOL_VERSION,
            node_id: self.config.node_id.clone(),
            hostname: hostname::get()
                .map(|h| h.to_string_lossy().to_string())
                .unwrap_or_default(),
            cpu_cores: self.hardware.cores.try_into().unwrap_or_default(),
            cpu_model: self.hardware.cpu_model.clone(),
            capabilities: self.hardware.capabilities.clone(),
            cores: self.hardware.cores,
            l2_cache_kb: self.hardware.l2_cache_kb,
            #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
            ops_per_sec: self.performance_ips as f32,
            public_key: None,
        };

        let resp = self
            .client
            .post(&url)
            .header("X-Keyforge-Secret", &self.config.secret)
            .json(&req)
            .send()
            .await
            .map_err(|e| AgentError::Network(e.to_string()))?;

        if !resp.status().is_success() {
            let txt = resp.text().await.unwrap_or_default();
            return Err(AgentError::Network(format!("Registration rejected: {txt}")));
        }

        let body: NodeResponse = resp
            .json()
            .await
            .map_err(|e| AgentError::Internal(format!("Invalid registration response: {e}")))?;

        if let Some(token) = body.token {
            info!("✅ Registration successful (Session Token Acquired)");
            self.session_token = Some(token);
        } else {
            warn!("⚠️ Registration successful but NO token received. Falling back to secret.");
        }

        Ok(())
    }

    async fn connect_and_loop(&mut self) -> AgentResult<()> {
        let mut ws_url =
            Url::parse(&self.config.hive_url).map_err(|e| AgentError::Network(e.to_string()))?;

        if ws_url.scheme() == "http" {
            let _ = ws_url.set_scheme("ws");
        } else if ws_url.scheme() == "https" {
            let _ = ws_url.set_scheme("wss");
        }

        ws_url.set_path("ws");
        ws_url
            .query_pairs_mut()
            .append_pair("node_id", &self.config.node_id);

        if let Some(token) = &self.session_token {
            ws_url.query_pairs_mut().append_pair("token", token);
        }

        info!("🌐 Connecting to Hive: {}", ws_url);

        let (ws_stream, _) = connect_async(ws_url.to_string())
            .await
            .map_err(|e| AgentError::Network(e.to_string()))?;

        let (mut write, mut read) = ws_stream.split();

        info!("✅ WebSocket Connected");

        let mut heartbeat = interval(Duration::from_secs(
            self.config.network.heartbeat_interval_seconds,
        ));
        let telemetry = self.telemetry.clone();

        let mut sys = System::new();
        let pid = sysinfo::get_current_pid().map_err(|e| AgentError::Internal(e.to_string()))?;

        loop {
            tokio::select! {
                _ = heartbeat.tick() => {
                    sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);
                    let (memory_bytes, cpu_usage) = if let Some(process) = sys.process(pid) {
                        (process.memory(), process.cpu_usage())
                    } else {
                        (0, 0.0)
                    };

                    let (ips, temp, best) = telemetry.snapshot();
                    let job_id = telemetry.get_job_id();

                    let job_id_opt = if job_id == self.config.system.idle_job_id { None } else { Some(job_id) };
                    let best_opt = if best == 0.0 { None } else { Some(best) };

                    let payload = NodeTelemetry {
                        cpu_usage,
                        memory_bytes,
                        ips,
                        active_threads: 0, // Default for now
                        job_id: job_id_opt,
                        temp,
                        current_best: best_opt,
                        #[allow(clippy::cast_precision_loss)]
                        memory_usage: format!("{:.2} MB", (memory_bytes as f64) / 1024.0 / 1024.0),
                        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
                    };

                    if let Ok(json) = serde_json::to_string(&payload) {
                        if let Err(e) = write.send(Message::Text(json.into())).await {
                            return Err(AgentError::Network(e.to_string()));
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
                        Some(Ok(Message::Close(_))) => return Err(AgentError::Network("Server closed connection".into())),
                        Some(Err(e)) => return Err(AgentError::Network(e.to_string())),
                        None => return Err(AgentError::Network("Stream ended".into())),
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
        let resp = self
            .auth_builder(self.client.get(&url))
            .send()
            .await
            .map_err(|e| AgentError::Network(e.to_string()))?;

        if resp.status().is_success() {
            let queue_resp: JobQueueResponse = resp
                .json()
                .await
                .map_err(|e| AgentError::Network(e.to_string()))?;

            if let (Some(jid), Some(config)) = (queue_resp.job_id, queue_resp.config) {
                info!("📥 Acquired Job {}", jid);
                let _ = self.job_tx.send((jid, config)).await;
            }
        }
        Ok(())
    }

    pub async fn submit_result(&self, result: ResultSubmission) -> AgentResult<()> {
        let url = format!("{}/results", self.config.hive_url);

        let resp = self
            .auth_builder(self.client.post(&url))
            .json(&result)
            .send()
            .await;

        match resp {
            Ok(r) => {
                let status = r.status();
                if !status.is_success() {
                    let txt = r.text().await.unwrap_or_default();
                    error!("❌ Submission Rejected: {}", txt);
                    if status.is_server_error() {
                        self.outbox.save_to_wal(&result)?;
                    } else if status.is_client_error() {
                        self.outbox.save_to_dead_letter(&result, &txt)?;
                    }
                    return Err(AgentError::Network(format!("Submission rejected: {txt}")));
                }
                Ok(())
            }
            Err(e) => {
                error!("❌ Network Error during submission: {}", e);
                self.outbox.save_to_wal(&result)?;
                Err(AgentError::Network(e.to_string()))
            }
        }
    }

    async fn flush_wal(&self) {
        let pending = self.outbox.get_pending();
        if pending.is_empty() {
            return;
        }
        info!(
            "🔄 WAL Flush: Attempting to resend {} pending submissions...",
            pending.len()
        );
        for (path, submission) in pending {
            match self.submit_result(submission).await {
                Ok(()) => {
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

    fn auth_builder(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(token) = &self.session_token {
            builder.bearer_auth(token)
        } else {
            builder.header("X-Keyforge-Secret", &self.config.secret)
        }
    }
}
