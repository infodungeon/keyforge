// ===== keyforge/crates/keyforge-node/src/worker.rs =====
use crate::hw_detect;
use crate::models::{JobQueueResponse, PopulationResponse, SubmitResultRequest};
use keyforge_core::config::Config;
use keyforge_core::keycodes::KeycodeRegistry;
use keyforge_core::layouts::layout_string_to_u16;
use keyforge_core::optimizer::{OptimizationOptions, Optimizer, ProgressCallback};
// FIXED: Added TuningProfile to imports
use keyforge_core::protocol::{RegisterNodeRequest, RegisterNodeResponse, TuningProfile};
use keyforge_core::scorer::Scorer;
use reqwest::Client;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tracing::{error, info, warn};

struct WorkerLogger;

impl ProgressCallback for WorkerLogger {
    fn on_progress(&self, _ep: usize, score: f32, _layout: &[u16], ips: f32) -> bool {
        if fastrand::f32() < 0.01 {
            info!("   .. optimizing .. best: {:.0} ({:.1} M/s)", score, ips);
        }
        true
    }
}

async fn ensure_asset(client: &Client, hive_url: &str, filename: &str) -> Result<String, String> {
    let local_path = format!("data/{}", filename);
    if Path::new(&local_path).exists() {
        return Ok(local_path);
    }

    info!("⬇️ Downloading asset: {}", filename);
    let url = format!("{}/data/{}", hive_url, filename);

    let resp = client.get(&url).send().await.map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!(
            "Hive missing asset '{}': {}",
            filename,
            resp.status()
        ));
    }

    let content = resp.bytes().await.map_err(|e| e.to_string())?;

    if let Some(parent) = Path::new(&local_path).parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| e.to_string())?;
    }

    let mut file = tokio::fs::File::create(&local_path)
        .await
        .map_err(|e| e.to_string())?;
    file.write_all(&content).await.map_err(|e| e.to_string())?;

    Ok(local_path)
}

pub async fn run_worker(hive_url: String, node_id: String) {
    let client = Client::builder()
        .timeout(Duration::from_secs(120))
        .connect_timeout(Duration::from_secs(10))
        .build()
        .unwrap_or_else(|_| Client::new());

    info!("🤖 Worker {} initializing...", node_id);

    // 1. Hardware Detection
    let topo = hw_detect::detect_topology();

    // Placeholder calibration
    let ops_per_sec = 5_000_000.0;

    // 2. Register with Hive
    let req = RegisterNodeRequest {
        node_id: node_id.clone(),
        cpu_model: topo.model,
        cores: topo.cores as i32,
        l2_cache_kb: topo.l2_kb.map(|x| x as i32),
        ops_per_sec,
    };

    let tuning = match client
        .post(format!("{}/nodes/register", hive_url))
        .json(&req)
        .send()
        .await
    {
        Ok(res) => match res.json::<RegisterNodeResponse>().await {
            Ok(r) => {
                info!(
                    "✅ Registered with Hive. Strategy: {} | Batch: {}",
                    r.tuning.strategy, r.tuning.batch_size
                );
                r.tuning
            }
            Err(e) => {
                warn!(
                    "Failed to parse registration response: {}. Using defaults.",
                    e
                );
                TuningProfile {
                    strategy: "fly".into(),
                    batch_size: 10000,
                }
            }
        },
        Err(e) => {
            warn!("Failed to register node: {}. Proceeding anonymously.", e);
            TuningProfile {
                strategy: "fly".into(),
                batch_size: 10000,
            }
        }
    };

    let _ = tokio::fs::create_dir_all("data").await;

    let registry = KeycodeRegistry::new_with_defaults();
    let registry_arc = Arc::new(registry);

    let mut cached_scorer: Option<Arc<Scorer>> = None;
    let mut cached_config_sig: String = String::new();

    loop {
        // 3. Poll Loop
        let job_resp: JobQueueResponse =
            match client.get(format!("{}/jobs/queue", hive_url)).send().await {
                Ok(r) => r.json().await.unwrap_or(JobQueueResponse {
                    job_id: None,
                    config: None,
                }),
                Err(e) => {
                    warn!("Hive unreachable: {}. Retrying...", e);
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

        // 4. Assets
        let cost_file = match ensure_asset(&client, &hive_url, &config.cost_matrix).await {
            Ok(p) => p,
            Err(_) => {
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }
        };

        let corpus_filename = if config.corpus_name == "default" {
            "ngrams-all.tsv"
        } else {
            &config.corpus_name
        };
        let actual_corpus_name = if corpus_filename.contains('.') {
            corpus_filename.to_string()
        } else {
            format!("{}.tsv", corpus_filename)
        };

        let ngram_file = match ensure_asset(&client, &hive_url, &actual_corpus_name).await {
            Ok(p) => p,
            Err(_) => {
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }
        };

        // 5. Scorer Init
        let current_sig = format!(
            "{}-{}-{}",
            config.definition.geometry.keys.len(),
            cost_file,
            ngram_file
        );

        if cached_scorer.is_none() || cached_config_sig != current_sig {
            let scorer_config = Config {
                weights: config.weights.clone(),
                search: config.params,
                ..Default::default()
            };

            match Scorer::new(
                &cost_file,
                &ngram_file,
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

        // 6. Population (FIXED: Clean async handling)
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

        // 7. Optimize
        let sys_config = Config {
            weights: config.weights,
            search: config.params,
            ..Default::default()
        };
        let mut options = OptimizationOptions::from(&sys_config);
        options.pinned_keys = config.pinned_keys;

        options.params.search_steps = tuning.batch_size;

        let key_count = config.definition.geometry.keys.len();
        options.initial_population = pop_resp
            .layouts
            .iter()
            .map(|s| layout_string_to_u16(s, key_count, &registry_arc))
            .collect();

        let result = tokio::task::spawn_blocking(move || {
            let optimizer = Optimizer::new(scorer_arc, options);
            optimizer.run(None, WorkerLogger)
        })
        .await
        .unwrap();

        // 8. Submit
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
}
