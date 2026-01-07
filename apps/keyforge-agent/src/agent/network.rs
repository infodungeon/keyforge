use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use keyforge_infra::HiveClient;
use keyforge_protocol::{
    JobConfig, JobQueueResponse, NodeRequest, NodeResponse, PopulationResponse, ResultSubmission,
    TuningProfile, NodeTelemetry,
};
use rand::Rng;
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::fs;
use tokio::sync::mpsc;
use tracing::{error, info, warn};
use uuid::Uuid;
use tokio_tungstenite::tungstenite::protocol::Message;
use crate::models::SharedTelemetry;
use crate::agent::compute;
use keyforge_core::DeterministicScorer;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio_tungstenite::connect_async;
use tokio::sync::broadcast;
use futures::{StreamExt, SinkExt};

/// Constructs a HiveClient with optional authentication secret.
/// Handles fallback to unauthenticated client if secret derivation fails.
#[must_use]
pub fn build_client(base_url: &str, secret: Option<String>) -> HiveClient {
    match HiveClient::new(base_url.to_string(), secret.clone()) {
        Ok(c) => c,
        Err(e) => {
            if secret.is_some() {
                warn!(error = %e, "failed to build HiveClient with secret, retrying without");
                HiveClient::new(base_url.to_string(), None).unwrap_or_else(|e2| {
                    error!("FATAL: Failed to build HiveClient: {}. Base URL: {}", e2, base_url);
                    std::process::exit(1);
                })
            } else {
                error!("FATAL: Failed to build HiveClient: {}. Base URL: {}", e, base_url);
                std::process::exit(1);
            }
        }
    }
}

/// Registers the node with the Hive server and retrieves a tuning profile.
///
/// # Security
/// Exits the process on 401 Unauthorized if a secret was provided.
#[must_use]
pub async fn register_node(
    client: &HiveClient,
    req: &NodeRequest,
    default_threads: usize,
) -> TuningProfile {
    match client
        .post("nodes/register")
        .header("Content-Type", "application/json")
        .json(req)
        .send()
        .await
    {
        Ok(res) => {
            if res.status() == 401 {
                error!(node_id = %req.node_id, "auth failure: Hive rejected connection, check HIVE_SECRET");
                std::process::exit(1);
            }
            match res.json::<NodeResponse>().await {
                Ok(r) => r.tuning,
                Err(_) => TuningProfile {
                    strategy: "fly".into(),
                    batch_size: 10000,
                    thread_count: default_threads,
                },
            }
        }
        Err(e) => {
            warn!(error = %e, "failed to register node, defaulting");
            TuningProfile {
                strategy: "fly".into(),
                batch_size: 10000,
                thread_count: default_threads,
            }
        }
    }
}

/// Fetches the next available job from the Hive queue.
#[must_use]
pub async fn fetch_job_queue(client: &HiveClient) -> Option<(String, JobConfig)> {
    match client.get("jobs/queue").send().await {
        Ok(r) => {
            let resp = r
                .json::<JobQueueResponse>()
                .await
                .unwrap_or(JobQueueResponse {
                    job_id: None,
                    config: None,
                });
            match (resp.job_id, resp.config) {
                (Some(id), Some(cfg)) => Some((id, cfg)),
                _ => None,
            }
        }
        Err(e) => {
            warn!(error = %e, "failed to fetch job queue");
            None
        }
    }
}

/// Retrieves the current population of layouts for a given job.
#[must_use]
pub async fn fetch_population(client: &HiveClient, job_id: &str) -> Vec<String> {
    match client
        .get(&format!("jobs/{}/population", job_id))
        .send()
        .await
    {
        Ok(r) => {
            let resp = r.json::<PopulationResponse>().await.ok();
            resp.map(|r| r.layouts).unwrap_or_default()
        }
        Err(_) => vec![],
    }
}

// --- CIRCUIT BREAKER ---

/// A circuit breaker to prevent hammering the Hive server during outages.
#[derive(Debug)]
pub struct CircuitBreaker {
    failures: u32,
    threshold: u32,
    last_failure: Option<Instant>,
    cooldown: Duration,
}

impl CircuitBreaker {
    #[must_use]
    pub fn new(threshold: u32, cooldown_secs: u64) -> Self {
        Self {
            failures: 0,
            threshold,
            last_failure: None,
            cooldown: Duration::from_secs(cooldown_secs),
        }
    }

    pub fn can_attempt(&self) -> bool {
        if self.failures < self.threshold {
            return true;
        }
        if let Some(last) = self.last_failure {
            if last.elapsed() >= self.cooldown {
                return true;
            }
        }
        false
    }

    pub fn record_success(&mut self) {
        if self.failures > 0 {
            info!("circuit breaker recovered");
        }
        self.failures = 0;
        self.last_failure = None;
    }

    pub fn record_failure(&mut self) {
        self.failures += 1;
        self.last_failure = Some(Instant::now());
        if self.failures == self.threshold {
            warn!(
                failures = self.failures,
                cooldown_secs = self.cooldown.as_secs(),
                "circuit breaker tripped"
            );
        }
    }
}

// --- ASYNC OUTBOX WITH PERSISTENCE ---

/// An asynchronous outbox that persists results to a Write-Ahead Log (WAL) before sending.
/// This ensures result durability even if the agent crashes or loses connectivity.
pub struct ResultOutbox {
    sender: mpsc::Sender<ResultSubmission>,
}

impl ResultOutbox {
    #[must_use]
    pub fn new(client: HiveClient, data_root: PathBuf, buffer_size: usize) -> Self {
        let (tx, mut rx) = mpsc::channel::<ResultSubmission>(buffer_size);
        // CHANGED: Use user/agent_wal
        let wal_dir = data_root.join("user/agent_wal");

        tokio::spawn(async move {
            info!("result outbox started");
            let mut cb = CircuitBreaker::new(5, 30);

            if let Err(e) = fs::create_dir_all(&wal_dir).await {
                error!(directory = %wal_dir.display(), error = %e, "failed to create WAL directory");
            }

            if let Ok(mut entries) = fs::read_dir(&wal_dir).await {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("json") {
                        if let Ok(content) = fs::read_to_string(&path).await {
                            if let Ok(sub) = serde_json::from_str::<ResultSubmission>(&content) {
                                info!(job_id = %sub.job_id, "recovered unsent result");
                                if send_with_retry(&client, &sub, &mut cb).await {
                                    let _ = fs::remove_file(path).await;
                                }
                            }
                        }
                    }
                }
            }

            while let Some(submission) = rx.recv().await {
                let wal_id = Uuid::new_v4();
                let wal_path = wal_dir.join(format!("{}.json", wal_id));

                if let Ok(json) = serde_json::to_string(&submission) {
                    if let Err(e) = fs::write(&wal_path, json).await {
                        error!(path = %wal_path.display(), error = %e, "failed to write WAL");
                    }
                }

                if !cb.can_attempt() {
                    warn!(
                        job_id = %submission.job_id,
                        "circuit breaker open, result queued in WAL"
                    );
                    continue;
                }

                if send_with_retry(&client, &submission, &mut cb).await {
                    let _ = fs::remove_file(&wal_path).await;
                } else {
                    error!(job_id = %submission.job_id, "failed to submit result, persisted to WAL");
                }
            }
            info!("result outbox stopped");
        });

        Self { sender: tx }
    }

    /// Attempts to send a result through the outbox.
    /// Returns an error if the internal buffer is full (back-pressure).
    pub fn try_send(&self, result: ResultSubmission) -> Result<(), anyhow::Error> {
        match self.sender.try_send(result) {
            Ok(_) => Ok(()),
            Err(mpsc::error::TrySendError::Full(_)) => {
                warn!("outbox full, back-pressure triggered");
                Err(anyhow::anyhow!("outbox full"))
            }
            Err(e) => Err(anyhow::anyhow!("send error: {}", e)),
        }
    }
}

async fn send_with_retry(
    client: &HiveClient,
    submission: &ResultSubmission,
    cb: &mut CircuitBreaker,
) -> bool {
    let mut attempts = 0;
    let max_attempts = 3;

    while attempts < max_attempts {
        match client
            .post("results")
            .header("Content-Type", "application/json")
            .json(submission)
            .send()
            .await
        {
            Ok(resp) => {
                if resp.status().is_success() {
                    cb.record_success();
                    return true;
                } else {
                    warn!(
                        status = %resp.status(),
                        job_id = %submission.job_id,
                        "submission rejected"
                    );
                    if resp.status().as_u16() >= 400 && resp.status().as_u16() < 500 {
                        return true;
                    }
                }
            }
            Err(e) => {
                warn!(error = %e, "network error sending result");
            }
        }

        attempts += 1;
        if attempts < max_attempts {
            let base_backoff = 500 * (1 << attempts);
            let jitter = rand::thread_rng().gen_range(0..500);
            let backoff = Duration::from_millis(base_backoff + jitter);
            tokio::time::sleep(backoff).await;
        }
    }

    cb.record_failure();
    false
}

/// Verifies the identity of the Hive server using a pinned public key.
///
/// Returns `true` if verification succeeds or if no pinning is configured.
pub fn verify_server_identity(
    server_public_key: &str,
    challenge: &str,
    signature: &str,
) -> Result<bool, String> {
    if server_public_key.is_empty() {
        return Ok(true); // Dev mode / No pinning
    }

    let public_bytes =
        hex::decode(server_public_key).map_err(|_| "Invalid public key hex".to_string())?;

    let public_key = VerifyingKey::from_bytes(
        public_bytes
            .as_slice()
            .try_into()
            .map_err(|_| "Invalid key length")?,
    )
    .map_err(|e| e.to_string())?;

    let sig_bytes = hex::decode(signature).map_err(|_| "Invalid signature hex".to_string())?;

    let sig = Signature::from_bytes(
        sig_bytes
            .as_slice()
            .try_into()
            .map_err(|_| "Invalid signature length")?,
    );

    public_key
        .verify(challenge.as_bytes(), &sig)
        .map(|_| true)
        .map_err(|e| e.to_string())
}

#[derive(Deserialize)]
#[serde(tag = "type", content = "payload")]
enum ServerMessage {
    Job {
        #[serde(rename = "id")]
        _id: String,
    },
    Cancel {
        id: String,
    },
}

/// Manages a single WebSocket connection to the Hive.
///
/// Handles job signals, heartbeats, and cancellations.
#[allow(clippy::too_many_arguments)]
pub async fn process_connection(
    mut ws_stream: tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    client: &HiveClient,
    outbox: &Arc<ResultOutbox>,
    node_id: &str,
    data_root: &std::path::Path,
    signing_key: &ed25519_dalek::SigningKey,
    compute_limiter: &Arc<Semaphore>,
    shutdown: &mut broadcast::Receiver<()>,
) {
    let mut active_job_handle: Option<tokio::task::JoinHandle<()>> = None;
    let mut active_stop_flag: Option<Arc<AtomicBool>> = None;
    let mut active_job_id: Option<String> = None;
    let mut check_queue = true;

    // NEW: Shared Telemetry State
    let telemetry = Arc::new(crate::models::AgentTelemetry::default());
    // NEW: Telemetry Ticker (1Hz)
    let mut telemetry_ticker = tokio::time::interval(Duration::from_secs(1));

    loop {
        if active_job_handle.is_none() && check_queue {
            check_queue = false;
            if let Some((job_id, config)) = fetch_job_queue(client).await {
                info!(job_id = %job_id, "processing job");

                let outbox_clone = outbox.clone();
                let node_id_clone = node_id.to_string();
                let data_root_clone = data_root.to_path_buf();
                let client_clone = client.clone();
                let signing_key = Arc::new(signing_key.clone());
                let limiter_clone = compute_limiter.clone();
                // Pass telemetry to compute task
                let telemetry_clone = telemetry.clone();

                let stop_flag = Arc::new(AtomicBool::new(false));
                active_stop_flag = Some(stop_flag.clone());
                active_job_id = Some(job_id.clone());

                let job_id_task = job_id.clone();

                let spawn_res = tokio::spawn(async move {
                    let compute_res = async {
                        let loader =
                            Box::new(keyforge_infra::FsProvider::new(data_root_clone.clone()));

                        let asset_manager = keyforge_infra::AssetManager::new(
                            client_clone.clone(),
                            data_root_clone.clone(),
                        );

                        let (cost_path, corpus_root) =
                            compute::prepare_assets(&asset_manager, &config).await?;

                        let prepared = compute::create_engine_request(
                            loader,
                            data_root_clone,
                            &config,
                            &cost_path,
                            &corpus_root,
                        ).await?;

                        let registry = prepared.registry.clone();

                        let result = compute::run_optimization(
                            prepared.req,
                            job_id_task.clone(),
                            stop_flag,
                            limiter_clone,
                            telemetry_clone, // <--- New Argument
                        )
                        .await?;

                        info!(
                            job_id = %job_id_task,
                            fast_score = result.score,
                            "optimization complete"
                        );

                        // --- DETERMINISTIC RECALCULATION ---
                        // We ignore the score from the optimizer (approximate float math)
                        // and recalculate using the Sidecar (exact integer math).
                        let deterministic_score = DeterministicScorer::score(
                            &prepared.keyboard,
                            &prepared.corpus,
                            &prepared.rubric,
                            &result.layout,
                            &prepared.cost_overrides,
                        );

                        // Silence Protocol: Discard if no improvement over baseline
                        if let Some(baseline) = config.baseline_score {
                            if deterministic_score >= baseline {
                                info!(
                                    job_id = %job_id_task,
                                    score = deterministic_score,
                                    baseline = baseline,
                                    "result discarded (no improvement)"
                                );
                                return Ok(());
                            }
                        }

                        let layout_str = result
                            .layout
                            .keys
                            .iter()
                            .map(|&c| registry.get_label(c))
                            .collect::<Vec<String>>()
                            .join(" ");

                        let timestamp = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or(Duration::from_secs(0))
                            .as_secs();

                        let nonce = rand::thread_rng().gen::<u64>();

                        // Sign the DETERMINISTIC score
                        let signature = crypto::sign_result_direct(
                            &signing_key,
                            &job_id_task,
                            &layout_str,
                            deterministic_score,
                            timestamp,
                            nonce,
                        )
                        .map_err(|e| anyhow::anyhow!("signing failed: {:?}", e))?;

                        let _ = outbox_clone.try_send(ResultSubmission {
                            version: keyforge_protocol::PROTOCOL_VERSION,
                            job_id: job_id_task,
                            layout: layout_str,
                            score: deterministic_score,
                            node_id: node_id_clone,
                            timestamp,
                            nonce,
                            signature: Some(signature),
                        });
                        Ok::<(), anyhow::Error>(())
                    }
                    .await;

                    if let Err(e) = compute_res {
                        error!(error = %e, "compute error");
                    }
                });

                active_job_handle = Some(spawn_res);
            }
        }

        tokio::select! {
            biased;

            _ = shutdown.recv() => {
                info!("agent shutting down (signal received)");
                if let Some(flag) = active_stop_flag {
                    flag.store(true, Ordering::SeqCst);
                }
                if let Some(handle) = active_job_handle {
                    let _ = handle.await;
                }
                return;
            }

            // NEW: Telemetry Tick
            _ = telemetry_ticker.tick() => {
                let (ips, temp, best) = telemetry.snapshot();
                
                // Only send robust telemetry if active, otherwise simple ping
                if ips > 0.0 || active_job_id.is_some() {
                    let packet = NodeTelemetry {
                        job_id: active_job_id.clone(),
                        ips,
                        temp,
                        current_best: if best > 0.0 { Some(best) } else { None },
                        memory_usage: 0, // Placeholder for sysinfo
                        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    };
                    
                    if let Ok(json) = serde_json::to_string(&packet) {
                        if let Err(e) = ws_stream.send(Message::Text(json)).await {
                            warn!("Telemetry send failed: {}", e);
                            break; 
                        }
                    }
                } else {
                    if let Err(e) = ws_stream.send(Message::Ping(vec![].into())).await {
                        warn!("Ping failed: {}", e);
                        break;
                    }
                }
            }

            msg = ws_stream.next() => {
                match msg {
                    Some(Ok(Message::Text(txt))) => {
                        if let Ok(server_msg) = serde_json::from_str::<ServerMessage>(&txt) {
                            match server_msg {
                                ServerMessage::Job { .. } => {
                                    info!("job signal received");
                                    check_queue = true;
                                }
                                ServerMessage::Cancel { id } => {
                                    if let Some(current_id) = &active_job_id {
                                        if current_id == &id {
                                            info!(job_id = %id, "cancellation received");
                                            if let Some(flag) = &active_stop_flag {
                                                flag.store(true, Ordering::SeqCst);
                                            }
                                        }
                                    }
                                }
                            }
                        } else if txt.contains("\"type\":\"Job\"") {
                            check_queue = true;
                        }
                    }
                    Some(Ok(Message::Ping(d))) => {
                        if let Err(e) = ws_stream.send(Message::Pong(d)).await {
                            warn!(error = %e, "failed to send pong");
                            break;
                        }
                    }
                    Some(Err(e)) => {
                        warn!(error = %e, "websocket error");
                        break;
                    }
                    None => break,
                    _ => {}
                }
            }

            _ = async {
                if let Some(h) = active_job_handle.as_mut() {
                    h.await.ok();
                    true
                } else {
                    std::future::pending().await
                }
            }, if active_job_handle.is_some() => {
                active_job_handle = None;
                active_stop_flag = None;
                active_job_id = None;
                check_queue = true;
            }

            _ = tokio::time::sleep(Duration::from_secs(30)), if active_job_handle.is_none() => {
                check_queue = true;
            }
        }
    }
}