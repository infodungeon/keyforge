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
        if fastrand::f32() < 0.05 {
            info!("   .. optimizing .. best: {:.0} ({:.1} M/s)", score, ips);
        }
        true
    }
}

pub async fn run_worker(hive_url: String, node_id: String) {
    let client = Client::new();
    info!("🤖 Worker {} connecting to Hive at {}", node_id, hive_url);

    let registry_path = "data/keycodes.json";
    let registry = if std::path::Path::new(registry_path).exists() {
        KeycodeRegistry::load_from_file(registry_path)
            .unwrap_or_else(|_| KeycodeRegistry::new_with_defaults())
    } else {
        KeycodeRegistry::new_with_defaults()
    };
    let registry_arc = Arc::new(registry);

    loop {
        info!("zzz... Polling for work...");
        let job_resp: JobQueueResponse =
            match client.get(format!("{}/jobs/queue", hive_url)).send().await {
                Ok(r) => r.json().await.unwrap_or(JobQueueResponse {
                    job_id: None,
                    config: None,
                }),
                Err(_) => {
                    warn!("Hive unreachable. Retrying in 5s...");
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
            };

        let (job_id, config) = match (job_resp.job_id, job_resp.config) {
            (Some(id), Some(cfg)) => (id, cfg),
            _ => {
                info!("No jobs available.");
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }
        };

        info!("📋 Received Job: {}", &job_id[0..8]);

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

        info!(
            "🧬 Downloaded {} parents from Hive.",
            pop_resp.layouts.len()
        );

        let (cost_path, ngrams_path) = match config.corpus_name.as_str() {
            "test_corpus" | "default" => ("data/cost_matrix.csv", "data/ngrams-all.tsv"),
            other => {
                warn!("Unknown corpus '{}', defaulting to standard files", other);
                ("data/cost_matrix.csv", "data/ngrams-all.tsv")
            }
        };

        let search_params = keyforge_core::config::SearchParams {
            pinned_keys: config.pinned_keys,
            search_epochs: 1000,
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

        let scorer = match Scorer::new(cost_path, ngrams_path, &config.geometry, sys_config, false)
        {
            Ok(s) => Arc::new(s),
            Err(e) => {
                error!("Failed to init scorer: {}", e);
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }
        };

        info!("🔨 Working...");
        let optimizer = Optimizer::new(scorer.clone(), options);
        let result = tokio::task::spawn_blocking(move || optimizer.run(None, WorkerLogger))
            .await
            .unwrap();

        let layout_str = result
            .layout
            .iter()
            .map(|&c| registry_arc.get_label(c))
            .collect::<Vec<String>>()
            .join(" ");

        // FIXED: Removed redundant variable declaration
        let pm = keyforge_core::optimizer::mutation::build_pos_map(&result.layout);

        let verified_score = scorer.score_details(&pm, 3000).layout_score;

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
                if r.status().is_success() {
                    info!("✅ Submission accepted!");
                } else {
                    let err = r.text().await.unwrap_or_default();
                    warn!("❌ Submission rejected: {}", err);
                }
            }
            Err(e) => error!("Failed to submit: {}", e),
        }
    }
}
