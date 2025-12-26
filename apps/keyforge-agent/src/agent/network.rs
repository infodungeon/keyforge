// ===== keyforge/crates/keyforge-agent/src/agent/network.rs =====
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use keyforge_infra::HiveClient;
use keyforge_protocol::{
    JobConfig, JobQueueResponse, NodeRequest, NodeResponse, PopulationResponse, ResultSubmission,
    TuningProfile,
};
use rand::Rng;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::fs;
use tokio::sync::mpsc;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Constructs a HiveClient with optional authentication secret.
#[must_use]
pub fn build_client(base_url: &str, secret: Option<String>) -> HiveClient {
    match HiveClient::new(base_url.to_string(), secret) {
        Ok(c) => c,
        Err(e) => {
            warn!(
                error = %e,
                "failed to build HiveClient, retrying with defaults"
            );
            HiveClient::new(base_url.to_string(), None).unwrap()
        }
    }
}

/// Registers the node with the Hive server and retrieves a tuning profile.
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
