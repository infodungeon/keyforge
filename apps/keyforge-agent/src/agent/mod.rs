pub mod errors;
pub mod calibration;
pub mod compute;
pub mod maintenance;
pub mod network;
pub mod telemetry;
use crate::hw_detect;
use futures::SinkExt;
use futures::StreamExt;
use keyforge_core::DeterministicScorer;
use keyforge_infra::init::initialize_workspace;
use keyforge_infra::HiveClient;
use keyforge_protocol::{NodeRequest, ResultSubmission, PROTOCOL_VERSION};
use keyforge_security as crypto;
use rand::Rng;
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{broadcast, Semaphore};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{error, info, warn};
use url::Url;

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

/// Main entry point for the agent worker.
///
/// Orchestrates identity registration, hardware detection, calibration,
/// and the persistent connection to the Hive.
pub async fn run_worker(
    hive_url: String,
    node_id: String,
    secret: Option<String>,
    signing_key: ed25519_dalek::SigningKey,
    data_root: PathBuf,
    mut shutdown: broadcast::Receiver<()>,
    cores: usize,
) {


    let client = network::build_client(&hive_url, secret.clone());
    info!(node_id = %node_id, "agent initializing");
    let _ = crate::agent::maintenance::prune_stale_data(data_root.clone()).await;

    let topo = hw_detect::detect_topology().await.unwrap_or_else(|e| {
        error!(error = %e, "hardware detection failed, using defaults");
        hw_detect::CpuCacheTopology::default()
    });

    let ops_per_sec: f64 =
        tokio::task::spawn_blocking(crate::agent::calibration::measure_performance)
            .await
            .unwrap_or(Ok(5_000_000.0))
            .unwrap_or_else(|e| {
                warn!(error = %e, "calibration failed, using default throughput");
                5_000_000.0
            });

    let public_key = ed25519_dalek::VerifyingKey::from(&signing_key);

    let req = NodeRequest {
        version: PROTOCOL_VERSION,
        node_id: node_id.clone(),
        cpu_model: topo.model.clone(),
        cores: (topo.cores as i32).max(1),
        l2_cache_kb: topo.l2_kb.map(|x| (x as i32).max(0)),
        ops_per_sec: ops_per_sec as f32,
        public_key: Some(hex::encode(public_key.to_bytes())),
    };

    let _ = network::register_node(&client, &req, 4).await;

    if let Err(e) = initialize_workspace(&data_root, keyforge_infra::InitMode::Validate) {
        error!(error = %e, "workspace initialization failed");
    }

    let outbox = Arc::new(network::ResultOutbox::new(
        client.clone(),
        data_root.clone(),
        100,
    ));

    let compute_limiter = Arc::new(Semaphore::new(cores.max(1)));

    let ws_url = match Url::parse(&hive_url) {
        Ok(mut u) => {
            let scheme = if u.scheme() == "https" { "wss" } else { "ws" };
            let _ = u.set_scheme(scheme);
            if let Ok(mut segments) = u.path_segments_mut() {
                segments.push("ws");
            }
            u.query_pairs_mut().append_pair("node_id", &node_id);
            u.to_string()
        }
        Err(_) => {
            error!(url = %hive_url, "invalid hive url");
            return;
        }
    };

    // Health Check Server
    let health_port = std::env::var("HEALTH_CHECK_PORT").unwrap_or_else(|_| "9090".to_string());
    let health_handle = tokio::spawn(async move {
        let addr = format!("0.0.0.0:{}", health_port);
        if let Ok(listener) = tokio::net::TcpListener::bind(&addr).await {
            info!(port = %health_port, "health check listening");
            loop {
                if let Ok((mut socket, _)) = listener.accept().await {
                    tokio::spawn(async move {
                        let _ = tokio::time::timeout(Duration::from_secs(5), async move {
                            use tokio::io::AsyncWriteExt;
                            let _ = socket
                                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n")
                                .await;
                            let _ = socket.shutdown().await;
                        })
                        .await;
                    });
                }
            }
        }
    });

    let mut retry_delay = 1;
    let max_retry = 30;

    loop {
        if shutdown.try_recv().is_ok() {
            info!("agent shutting down");
            break;
        }

        match connect_async(&ws_url).await {
            Ok((ws_stream, _)) => {
                info!("connected, waiting for work");
                retry_delay = 1;

                process_connection(
                    ws_stream,
                    &client,
                    &outbox,
                    &node_id,
                    &data_root,
                    &signing_key,
                    &compute_limiter,
                    &mut shutdown,
                )
                .await;
            }
            Err(e) => {
                warn!(error = %e, delay = %retry_delay, "connection failed, retrying");
                tokio::select! {
                    _ = shutdown.recv() => return,
                    _ = tokio::time::sleep(Duration::from_secs(retry_delay)) => {
                        retry_delay = (retry_delay * 2).min(max_retry);
                    }
                }
            }
        }
    }

    health_handle.abort();
}

/// Manages a single WebSocket connection to the Hive.
///
/// Handles job signals, heartbeats, and cancellations.
#[allow(clippy::too_many_arguments)]
async fn process_connection(
    mut ws_stream: tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    client: &HiveClient,
    outbox: &Arc<network::ResultOutbox>,
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

    loop {
        if active_job_handle.is_none() && check_queue {
            check_queue = false;
            if let Some((job_id, config)) = network::fetch_job_queue(client).await {
                info!(job_id = %job_id, "processing job");

                let outbox_clone = outbox.clone();
                let node_id_clone = node_id.to_string();
                let data_root_clone = data_root.to_path_buf();
                let client_clone = client.clone();
                let signing_key = Arc::new(signing_key.clone());
                let limiter_clone = compute_limiter.clone();

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

                        let nonce = rand::rng().random::<u64>();

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
                            version: PROTOCOL_VERSION,
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
