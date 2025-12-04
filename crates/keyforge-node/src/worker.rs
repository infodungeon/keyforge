use crate::models::{JobQueueResponse, PopulationResponse, SubmitResultRequest};
use keyforge_core::config::Config;
use keyforge_core::keycodes::KeycodeRegistry;
use keyforge_core::layouts::layout_string_to_u16;
use keyforge_core::optimizer::{OptimizationOptions, Optimizer, ProgressCallback};
use keyforge_core::scorer::Scorer;
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};

struct WorkerLogger;
impl ProgressCallback for WorkerLogger {
    fn on_progress(&self, _ep: usize, score: f32, _layout: &[u16], ips: f32) -> bool {
        if fastrand::f32() < 0.01 {
            // Log even less frequently (1%)
            info!("   .. optimizing .. best: {:.0} ({:.1} M/s)", score, ips);
        }
        true
    }
}

pub async fn run_worker(hive_url: String, node_id: String) {
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .build()
        .unwrap_or_else(|_| Client::new());

    info!("🤖 Worker {} connecting to Hive at {}", node_id, hive_url);

    let registry_path = "data/keycodes.json";
    let registry = if std::path::Path::new(registry_path).exists() {
        KeycodeRegistry::load_from_file(registry_path)
            .unwrap_or_else(|_| KeycodeRegistry::new_with_defaults())
    } else {
        KeycodeRegistry::new_with_defaults()
    };
    let registry_arc = Arc::new(registry);

    // --- CACHE STATE ---
    let mut cached_scorer: Option<Arc<Scorer>> = None;
    let mut cached_corpus_name: String = String::new();
    let mut cached_geometry_len: usize = 0;

    loop {
        info!("zzz... Polling for work...");

        let job_resp: JobQueueResponse =
            match client.get(format!("{}/jobs/queue", hive_url)).send().await {
                Ok(r) => r.json().await.unwrap_or(JobQueueResponse {
                    job_id: None,
                    config: None,
                }),
                Err(e) => {
                    warn!("Hive unreachable ({}). Retrying in 5s...", e);
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

        info!("📋 Received Job: {}", &job_id[0..8]);

        // --- SCORER CACHING LOGIC ---
        // Only reload from disk if the corpus or geometry size changes.
        // (Comparing geometry length is a cheap proxy for geometry equality here)
        let needs_reload = cached_scorer.is_none()
            || cached_corpus_name != config.corpus_name
            || cached_geometry_len != config.geometry.keys.len();

        if needs_reload {
            info!(
                "🔄 Loading Data Context (Corpus: {})...",
                config.corpus_name
            );

            let (cost_path, ngrams_path) = match config.corpus_name.as_str() {
                "test_corpus" | "default" => ("data/cost_matrix.csv", "data/ngrams-all.tsv"),
                other => {
                    warn!("Unknown corpus '{}', defaulting to standard files", other);
                    ("data/cost_matrix.csv", "data/ngrams-all.tsv")
                }
            };

            // Create a dummy config for the builder (weights are applied per-job later)
            let dummy_sys_config = Config::default();

            match Scorer::new(
                cost_path,
                ngrams_path,
                &config.geometry,
                dummy_sys_config,
                false,
            ) {
                Ok(s) => {
                    cached_scorer = Some(Arc::new(s));
                    cached_corpus_name = config.corpus_name.clone();
                    cached_geometry_len = config.geometry.keys.len();
                }
                Err(e) => {
                    error!("Failed to init scorer: {}. Skipping job.", e);
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
            }
        }

        // Get a cheap reference to the cached data
        // We clone the Arc, not the data.
        let mut active_scorer = cached_scorer.as_ref().unwrap().as_ref().clone();

        // Apply Job-Specific Weights (Cheap copy of f32 struct)
        active_scorer.weights = config.weights.clone();
        let scorer_arc = Arc::new(active_scorer);

        // --- FETCH POPULATION ---
        let pop_resp: PopulationResponse = match client
            .get(format!("{}/jobs/{}/population", hive_url, job_id))
            .send()
            .await
        {
            Ok(res) => res
                .json()
                .await
                .unwrap_or(PopulationResponse { layouts: vec![] }),
            Err(_) => PopulationResponse { layouts: vec![] },
        };

        // --- PERFORMANCE RESTORED ---
        let search_params = keyforge_core::config::SearchParams {
            pinned_keys: config.pinned_keys,
            search_epochs: 1000,  // RESTORED: High iteration count
            search_steps: 50_000, // RESTORED: High step count
            opt_limit_slow: 3000,
            ..Default::default()
        };

        let sys_config = Config {
            weights: config.weights,
            search: search_params,
            ..Default::default()
        };

        let mut options = OptimizationOptions::from(&sys_config);
        let key_count = config.geometry.keys.len();

        options.initial_population = pop_resp
            .layouts
            .iter()
            .map(|s| layout_string_to_u16(s, key_count, &registry_arc))
            .collect();

        info!("🔨 Working (Cached Context)...");

        let result =
            tokio::task::spawn_blocking(move || optimizer_run_wrapper(scorer_arc, options))
                .await
                .unwrap();

        let layout_str = result
            .layout
            .iter()
            .map(|&c| registry_arc.get_label(c))
            .collect::<Vec<String>>()
            .join(" ");

        let verified_score = result.score;

        info!("📤 Submitting result: {:.0}", verified_score);

        let submit_req = SubmitResultRequest {
            job_id: job_id.clone(),
            layout: layout_str,
            score: verified_score,
            node_id: node_id.clone(),
        };

        match client
            .post(format!("{}/results", hive_url))
            .json(&submit_req)
            .send()
            .await
        {
            Ok(r) => {
                if !r.status().is_success() {
                    let err = r.text().await.unwrap_or_default();
                    warn!("❌ Submission rejected: {}", err);
                }
            }
            Err(e) => error!("Failed to submit result: {}", e),
        }
    }
}

fn optimizer_run_wrapper(
    scorer: Arc<Scorer>,
    options: OptimizationOptions,
) -> keyforge_core::optimizer::runner::OptimizationResult {
    let optimizer = Optimizer::new(scorer, options);
    optimizer.run(None, WorkerLogger)
}
