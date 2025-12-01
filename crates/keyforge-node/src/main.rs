use clap::{Parser, Subcommand};
use keyforge_core::config::{Config, LayoutDefinitions, ScoringWeights};
use keyforge_core::geometry::{KeyNode, KeyboardGeometry};
use keyforge_core::optimizer::{mutation, OptimizationOptions, Optimizer, ProgressCallback};
use keyforge_core::scorer::{Scorer, ScorerBuilder};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::sync::Arc;
use std::time::{Duration, Instant};
use sysinfo::{CpuRefreshKind, RefreshKind, System};
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Hive Server URL
    #[arg(long, default_value = "http://localhost:3000")]
    hive: String,
}

#[derive(Subcommand)]
enum Commands {
    /// measures hardware performance to determine batch sizes
    Calibrate,
    /// Connects to the Hive and starts processing jobs
    Work,
}

// --- Data structures matching Hive API ---

#[derive(Deserialize)]
struct JobQueueResponse {
    job_id: Option<String>,
    config: Option<JobConfig>,
}

#[derive(Deserialize)]
struct JobConfig {
    geometry: KeyboardGeometry,
    weights: ScoringWeights,
    pinned_keys: String,
    corpus_name: String,
}

#[derive(Deserialize)]
struct PopulationResponse {
    layouts: Vec<String>,
}

#[derive(Serialize)]
struct SubmitResultRequest {
    job_id: String,
    layout: String,
    score: f32,
    node_id: String,
}

// --- Logger ---

struct WorkerLogger;
impl ProgressCallback for WorkerLogger {
    fn on_progress(&self, _ep: usize, score: f32, _layout: &[u8], ips: f32) -> bool {
        // Only log periodically to avoid spamming stdout
        if fastrand::f32() < 0.05 {
            info!("   .. optimizing .. best: {:.0} ({:.1} M/s)", score, ips);
        }
        true
    }
}

// --- Entry Point ---

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    // Generate a session ID for this run
    let node_id = format!(
        "node-{}",
        Uuid::new_v4().to_string().split('-').next().unwrap()
    );

    match cli.command {
        Commands::Calibrate => run_calibration(),
        Commands::Work => run_worker(cli.hive, node_id).await,
    }
}

async fn run_worker(hive_url: String, node_id: String) {
    let client = Client::new();
    info!("🤖 Worker {} connecting to Hive at {}", node_id, hive_url);

    loop {
        // 1. Get Job
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

        // 2. Get Population (Genetic Material)
        let pop_resp: PopulationResponse = client
            .get(format!("{}/jobs/{}/population", hive_url, job_id))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        info!(
            "🧬 Downloaded {} parents from Hive.",
            pop_resp.layouts.len()
        );

        // 3. Setup Optimizer
        // Resolve paths based on corpus_name from Hive
        let (cost_path, ngrams_path) = match config.corpus_name.as_str() {
            "test_corpus" | "default" => ("data/cost_matrix.csv", "data/ngrams-all.tsv"),
            other => {
                warn!("Unknown corpus '{}', defaulting to standard files", other);
                ("data/cost_matrix.csv", "data/ngrams-all.tsv")
            }
        };

        // Construct Config cleanly
        let search_params = keyforge_core::config::SearchParams {
            pinned_keys: config.pinned_keys,
            // Run short bursts (1000 epochs) to submit frequently
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

        // Convert Population Strings to Byte Vectors
        // We need key_count to pad strings correctly
        let key_count = config.geometry.keys.len();
        options.initial_population = pop_resp
            .layouts
            .iter()
            .map(|s| keyforge_core::layouts::layout_string_to_bytes(s, key_count))
            .collect();

        // Build Scorer
        let scorer = match Scorer::new(cost_path, ngrams_path, &config.geometry, sys_config, false)
        {
            Ok(s) => Arc::new(s),
            Err(e) => {
                error!("Failed to init scorer: {}", e);
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }
        };

        // 4. Run Optimization
        info!("🔨 Working...");
        let optimizer = Optimizer::new(scorer.clone(), options);
        let result = tokio::task::spawn_blocking(move || optimizer.run(None, WorkerLogger))
            .await
            .unwrap();

        // 5. Submit Result
        let layout_str = String::from_utf8_lossy(&result.layout_bytes).to_string();

        // --- CRITICAL FIX: Re-Score using score_details to match Server ---
        // The optimizer uses incremental updates which may drift slightly.
        // The server validates using score_details (absolute calculation).
        // We must calculate the canonical score here to ensure acceptance.
        let pos_map = mutation::build_pos_map(&result.layout_bytes);
        let verified_score = scorer.score_details(&pos_map, 3000).layout_score;

        info!(
            "📤 Submitting result: {:.0} ({})",
            verified_score, layout_str
        );

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

// --- Calibration Logic ---

fn run_calibration() {
    info!("🔌 Initializing KeyForge Node Calibration...");

    let mut sys =
        System::new_with_specifics(RefreshKind::nothing().with_cpu(CpuRefreshKind::everything()));
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    sys.refresh_cpu_all();

    let cpu_count = sys.cpus().len();
    let memory = sys.total_memory() / 1024 / 1024;
    let host_name = System::host_name().unwrap_or("Unknown".into());

    info!("🖥️  Host: {}", host_name);
    info!("🧠  CPU: {} cores", cpu_count);
    info!("💾  RAM: {} MB", memory);

    info!("🚀 Preparing Physics Engine for Stress Test...");
    let scorer = setup_benchmark_scorer();
    let layout_bytes = b"abcdefghijklmnopqrstuvwxyz.,;/"[0..scorer.key_count].to_vec();
    let pos_map = mutation::build_pos_map(&layout_bytes);

    let warmup_iters = 50_000;
    for _ in 0..warmup_iters {
        std::hint::black_box(scorer.score_full(&pos_map, 3000));
    }

    info!("🔥 Running Benchmark (5s)...");
    let start = Instant::now();
    let duration = Duration::from_secs(5);
    let mut iterations: u64 = 0;

    while start.elapsed() < duration {
        for _ in 0..100 {
            std::hint::black_box(scorer.score_full(&pos_map, 3000));
        }
        iterations += 100;
    }

    let elapsed = start.elapsed().as_secs_f64();
    let sops = iterations as f64 / elapsed;

    info!("✅ Calibration Complete");
    info!(
        "⚡ Speed: {:.2} Million Evaluations/sec (Single Core)",
        sops / 1_000_000.0
    );
}

// Helper to build a scorer without reading from disk
fn setup_benchmark_scorer() -> Scorer {
    let mut keys = Vec::new();
    for r in 0..3 {
        for c in 0..10 {
            keys.push(KeyNode {
                id: format!("k_{}_{}", r, c),
                hand: if c < 5 { 0 } else { 1 },
                finger: (c % 5) as u8,
                row: r as i8,
                col: c as i8,
                x: c as f32,
                y: r as f32,
                is_stretch: false,
            });
        }
    }

    let mut geom = KeyboardGeometry {
        keys,
        prime_slots: vec![13, 14, 15, 16],
        med_slots: vec![1, 2, 3, 4],
        low_slots: vec![20, 21, 22],
        home_row: 1,
        finger_origins: [[(0.0, 0.0); 5]; 2],
    };
    geom.calculate_origins();

    let mut ngram_data = String::new();
    let chars = "abcdefghijklmnopqrstuvwxyz.,;/";
    for c in chars.chars() {
        ngram_data.push_str(&format!("{}\t1000\n", c));
    }
    ngram_data.push_str("th\t5000\n");
    ngram_data.push_str("he\t4000\n");

    let cursor = Cursor::new(ngram_data);
    let weights = ScoringWeights::default();
    let defs = LayoutDefinitions::default();

    ScorerBuilder::new()
        .with_weights(weights)
        .with_defs(defs)
        .with_geometry(geom)
        .with_ngrams_from_reader(cursor)
        .expect("Failed to build bench scorer")
        .build()
        .expect("Failed to build scorer")
}
