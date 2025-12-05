use crate::hw_detect;
use crate::models::{JobQueueResponse, PopulationResponse, SubmitResultRequest};
use keyforge_core::config::Config;
use keyforge_core::keycodes::KeycodeRegistry;
use keyforge_core::layouts::layout_string_to_u16;
use keyforge_core::optimizer::{OptimizationOptions, Optimizer, ProgressCallback};
use keyforge_core::protocol::{RegisterNodeRequest, RegisterNodeResponse, TuningProfile};
use keyforge_core::scorer::Scorer;
use reqwest::{header, Client};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tracing::{error, info, warn};

struct WorkerLogger;

impl ProgressCallback for WorkerLogger {
    fn on_progress(&self, _ep: usize, score: f32, _layout: &[u16], ips: f32) -> bool {
        // Reduced log frequency to prevent spamming stdout on fast machines
        if fastrand::f32() < 0.01 {
            info!("   .. optimizing .. best: {:.0} ({:.1} M/s)", score, ips);
        }
        true
    }
}

async fn ensure_file(client: &Client, url: &str, local_path: &str) -> Result<(), String> {
    if Path::new(local_path).exists() {
        return Ok(());
    }

    info!("⬇️ Downloading asset: {}", local_path);
    let resp = client.get(url).send().await.map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!("Server missing asset '{}': {}", url, resp.status()));
    }

    let content = resp.bytes().await.map_err(|e| e.to_string())?;

    if let Some(parent) = Path::new(local_path).parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| e.to_string())?;
    }

    let mut file = tokio::fs::File::create(local_path)
        .await
        .map_err(|e| e.to_string())?;
    file.write_all(&content).await.map_err(|e| e.to_string())?;

    Ok(())
}

async fn ensure_corpus_bundle(
    client: &Client,
    hive_url: &str,
    corpus_name: &str,
) -> Result<String, String> {
    let bundle_dir = if corpus_name == "default" {
        "data/corpora/default".to_string()
    } else {
        format!("data/corpora/{}", corpus_name)
    };

    let files = ["1grams.csv", "2grams.csv", "3grams.csv", "words.csv"];

    for f in files {
        let local = format!("{}/{}", bundle_dir, f);
        let remote = format!("{}/{}/{}", hive_url, bundle_dir, f);

        ensure_file(client, &remote, &local).await?;
    }

    Ok(bundle_dir)
}

pub async fn run_worker(hive_url: String, node_id: String, secret: Option<String>) {
    // 1. Configure Auth Headers
    let mut headers = header::HeaderMap::new();
    if let Some(s) = secret {
        let mut auth_val = header::HeaderValue::from_str(&s).unwrap();
        auth_val.set_sensitive(true);
        headers.insert("X-Keyforge-Secret", auth_val);
    } else {
        warn!("⚠️  Starting Worker without HIVE_SECRET. Connection may be rejected.");
    }

    let client = Client::builder()
        .default_headers(headers)
        .timeout(Duration::from_secs(120))
        .connect_timeout(Duration::from_secs(10))
        .build()
        .unwrap_or_else(|_| Client::new());

    info!("🤖 Worker {} initializing...", node_id);

    let topo = hw_detect::detect_topology();
    // Default cap, can be overridden by Hive response
    let ops_per_sec = 5_000_000.0;

    let req = RegisterNodeRequest {
        node_id: node_id.clone(),
        cpu_model: topo.model,
        cores: topo.cores as i32,
        l2_cache_kb: topo.l2_kb.map(|x| x as i32),
        ops_per_sec,
    };

    // 2. Register
    let default_threads = (req.cores - 1).max(1) as usize;
    let default_tuning = TuningProfile {
        strategy: "fly".into(),
        batch_size: 10000,
        thread_count: default_threads,
    };

    let tuning = match client
        .post(format!("{}/nodes/register", hive_url))
        .json(&req)
        .send()
        .await
    {
        Ok(res) => {
            if res.status() == 401 {
                error!("❌ AUTH FAILURE: Hive rejected connection. Check HIVE_SECRET.");
                std::process::exit(1);
            }
            match res.json::<RegisterNodeResponse>().await {
                Ok(r) => {
                    info!(
                        "✅ Registered. Strategy: {} | Batch: {}",
                        r.tuning.strategy, r.tuning.batch_size
                    );
                    r.tuning
                }
                Err(_) => default_tuning,
            }
        }
        Err(e) => {
            warn!("Failed to register node: {}. Retrying...", e);
            default_tuning
        }
    };

    let _ = tokio::fs::create_dir_all("data").await;
    let registry = KeycodeRegistry::new_with_defaults();
    let registry_arc = Arc::new(registry);

    let mut cached_scorer: Option<Arc<Scorer>> = None;
    let mut cached_config_sig: String = String::new();

    loop {
        let job_resp: JobQueueResponse =
            match client.get(format!("{}/jobs/queue", hive_url)).send().await {
                Ok(r) => r.json().await.unwrap_or(JobQueueResponse {
                    job_id: None,
                    config: None,
                }),
                Err(_) => {
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
            };

        let (job_id, config) = match (job_resp.job_id, job_resp.config) {
            (Some(id), Some(cfg)) => (id, cfg),
            _ => {
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }
        };

        info!("📋 Processing Job: {}", &job_id[0..8]);

        // Download Assets
        let cost_local = format!("data/{}", config.cost_matrix);
        let cost_remote = format!("{}/data/{}", hive_url, config.cost_matrix);
        if let Err(e) = ensure_file(&client, &cost_remote, &cost_local).await {
            warn!("Failed to download cost matrix: {}", e);
            tokio::time::sleep(Duration::from_secs(5)).await;
            continue;
        }

        let corpus_dir = match ensure_corpus_bundle(&client, &hive_url, &config.corpus_name).await {
            Ok(d) => d,
            Err(e) => {
                warn!("Failed to download corpus bundle: {}", e);
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }
        };

        let current_sig = format!(
            "{}-{}-{}",
            config.definition.geometry.keys.len(),
            cost_local,
            corpus_dir
        );

        // Rebuild Scorer if configuration changed (Cache Check)
        if cached_scorer.is_none() || cached_config_sig != current_sig {
            let scorer_config = Config {
                weights: config.weights.clone(),
                search: config.params,
                ..Default::default()
            };

            match Scorer::new(
                &cost_local,
                &corpus_dir,
                &config.definition.geometry,
                scorer_config,
                false,
            ) {
                Ok(s) => {
                    cached_scorer = Some(Arc::new(s));
                    cached_config_sig = current_sig;
                }
                Err(e) => {
                    error!("Scorer Init Failed: {}. Skipping.", e);
                    continue;
                }
            }
        }

        let mut active_scorer = cached_scorer.as_ref().unwrap().as_ref().clone();
        active_scorer.weights = config.weights.clone();
        let scorer_arc = Arc::new(active_scorer);

        // Fetch Population
        let pop_resp: PopulationResponse = match client
            .get(format!("{}/jobs/{}/population", hive_url, job_id))
            .send()
            .await
        {
            Ok(r) => r
                .json()
                .await
                .unwrap_or(PopulationResponse { layouts: vec![] }),
            Err(_) => PopulationResponse { layouts: vec![] },
        };

        // Configure Optimizer
        let sys_config = Config {
            weights: config.weights,
            search: config.params,
            ..Default::default()
        };
        let mut options = OptimizationOptions::from(&sys_config);
        options.pinned_keys = config.pinned_keys;
        options.params.search_steps = tuning.batch_size;
        options.num_threads = tuning.thread_count;

        let key_count = config.definition.geometry.keys.len();
        options.initial_population = pop_resp
            .layouts
            .iter()
            .map(|s| layout_string_to_u16(s, key_count, &registry_arc))
            .collect();

        // --- SAFETY: ISOLATE THE OPTIMIZER ---
        // If the optimizer crashes (e.g., bad geometry vs pinned keys), we catch the panic
        // and prevent the Node from dying.
        let optimization_task = tokio::task::spawn_blocking(move || {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let optimizer = Optimizer::new(scorer_arc, options);
                optimizer.run(None, WorkerLogger)
            }))
        });

        match optimization_task.await {
            Ok(run_result) => {
                match run_result {
                    Ok(result) => {
                        // Success: Submit
                        let layout_str = result
                            .layout
                            .iter()
                            .map(|&c| registry_arc.get_label(c))
                            .collect::<Vec<String>>()
                            .join(" ");

                        let submit_req = SubmitResultRequest {
                            job_id: job_id.clone(),
                            layout: layout_str,
                            score: result.score,
                            node_id: node_id.clone(),
                        };

                        if let Err(e) = client
                            .post(format!("{}/results", hive_url))
                            .json(&submit_req)
                            .send()
                            .await
                        {
                            error!("Failed to submit result: {}", e);
                        }
                    }
                    Err(panic_err) => {
                        // Panic caught inside the closure
                        let msg = if let Some(s) = panic_err.downcast_ref::<&str>() {
                            *s
                        } else if let Some(s) = panic_err.downcast_ref::<String>() {
                            s
                        } else {
                            "Unknown Panic"
                        };
                        error!("🔥 CRITICAL: Optimizer Panicked on Job {}: {}", job_id, msg);
                        // We skip this job loop iteration, effectively dropping the poison job
                        tokio::time::sleep(Duration::from_secs(2)).await;
                    }
                }
            }
            Err(join_err) => {
                // Task failed to join (e.g. cancelled or panic propagated weirdly)
                error!("🔥 CRITICAL: Worker thread crashed: {}", join_err);
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    }
}
