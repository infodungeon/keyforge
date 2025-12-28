use futures::future::join_all;
use keyforge_hive::{create_app, state::AppState};
use keyforge_protocol::config::{CorpusSource, ScoringWeights, SearchParams};
use keyforge_protocol::geometry::KeyboardDefinition;
use keyforge_protocol::{JobRequest, JobResponse, NodeRequest, ResultSubmission, PROTOCOL_VERSION};
use reqwest::{header, Client};
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Once};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tokio::net::TcpListener;

const NODE_COUNT: usize = 50;
const RESULTS_PER_NODE: usize = 100;
const TEST_SECRET: &str = "test_secret_123";

static INIT: Once = Once::new();

fn init_tracing() {
    INIT.call_once(|| {
        std::env::set_var("RATE_LIMIT_PER_SEC", "10000");
        std::env::set_var("RATE_LIMIT_BURST", "10000");
        std::env::set_var("HIVE_SECRET", TEST_SECRET);

        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .with_test_writer()
            .init();
    });
}

fn get_data_path() -> PathBuf {
    let temp = std::env::temp_dir().join("keyforge_stress_test_data");
    let _ = fs::create_dir_all(&temp);
    temp
}

fn ensure_test_assets(data_root: &std::path::Path) {
    // Create User Structure
    let kb_dir = data_root.join("user/keyboards");
    fs::create_dir_all(&kb_dir).unwrap();
    let weights_dir = data_root.join("user/weights");
    fs::create_dir_all(&weights_dir).unwrap();
    let corpus_dir = data_root.join("user/corpora/default");
    fs::create_dir_all(&corpus_dir).unwrap();
    let config_dir = data_root.join("user/config");
    fs::create_dir_all(&config_dir).unwrap();

    let cost_path = weights_dir.join("cost_matrix.json");
    if !cost_path.exists() {
        fs::write(
            cost_path,
            r#"[{"from_key":"KA","to_key":"KB","cost_ms":10.0,"confidence_samples":10}]"#,
        )
        .unwrap();
    }

    if !corpus_dir.join("1grams.json").exists() {
        fs::write(
            corpus_dir.join("1grams.json"),
            r#"[{"char":"a","freq":100}]"#,
        )
        .unwrap();
        fs::write(
            corpus_dir.join("2grams.json"),
            r#"[{"char1":"a","char2":"b","freq":10}]"#,
        )
        .unwrap();
        fs::write(
            corpus_dir.join("3grams.json"),
            r#"[{"char1":"a","char2":"b","char3":"c","freq":5}]"#,
        )
        .unwrap();
        fs::write(
            corpus_dir.join("words.json"),
            r#"[{"word":"test","freq":20}]"#,
        )
        .unwrap();
    }

    // Create dummy keycodes
    if !config_dir.join("keycodes.json").exists() {
        fs::write(
            config_dir.join("keycodes.json"),
            r#"[{"code": 97, "id": "KC_A", "label": "A", "aliases": []}]"#,
        )
        .unwrap();
    }

    // Create dummy keyboards if they don't exist
    for name in ["corne", "szr35"] {
        let path = kb_dir.join(format!("{}.json", name));
        if !path.exists() {
            let json = format!(
                r#"{{
                "meta": {{ "name": "{}", "author": "test", "version": "1", "type": "ortho" }},
                "geometry": {{
                    "keys": [
                        {{"id": "KC_A", "x":0,"y":0,"hand":0,"finger":1,"row":0,"col":0}},
                        {{"id": "KC_B", "x":1,"y":0,"hand":0,"finger":2,"row":0,"col":1}}
                    ],
                    "prime_slots": [0, 1], "med_slots": [], "low_slots": [], "home_row": 0
                }},
                "layouts": {{ "default": "KC_A KC_B" }}
            }}"#,
                name
            );
            fs::write(path, json).unwrap();
        }
    }
}

fn load_keyboard(name: &str) -> KeyboardDefinition {
    let root = get_data_path();
    let path = root.join(format!("user/keyboards/{}.json", name));
    let content = fs::read_to_string(&path).unwrap_or_else(|_| panic!("Failed to read {:?}", path));
    serde_json::from_str(&content).expect("Failed to parse keyboard JSON")
}

async fn setup_server() -> (String, PgPool, Arc<AppState>) {
    init_tracing();

    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://keyforge:forge_password@localhost:5432/keyforge_hive".to_string()
    });

    let max_conns = std::env::var("DATABASE_MAX_CONNECTIONS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);

    let pool = PgPoolOptions::new()
        .max_connections(max_conns)
        .connect(&db_url)
        .await
        .expect("Failed to connect to Postgres");

    let crate_dir = env!("CARGO_MANIFEST_DIR");
    let migrations_path = std::path::Path::new(crate_dir).join("../keyforge-hive/migrations");
    let migrator = sqlx::migrate::Migrator::new(migrations_path)
        .await
        .expect("Failed to load migrations");
    migrator.run(&pool).await.expect("Failed to run migrations");

    let _ = sqlx::query("TRUNCATE results CASCADE").execute(&pool).await;
    let _ = sqlx::query("TRUNCATE nodes CASCADE").execute(&pool).await;

    let data_path = get_data_path();
    ensure_test_assets(&data_path);

    let state = Arc::new(AppState::new(
        pool.clone(),
        data_path.clone(),
        "test-key".to_string(),
    ));

    // Cache Warming
    use keyforge_model::loader::AssetLoader;
    let _ = state.assets.load_cost_matrix("cost_matrix.json");
    let _ = state
        .assets
        .load_corpus(&[keyforge_protocol::config::CorpusSource {
            id: "default".into(),
            weight: 1.0,
            hash: None,
        }]);

    let app = create_app(state.clone(), data_path);
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let port = addr.port();

    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .unwrap();
    });

    let base_url = format!("http://127.0.0.1:{}", port);
    (base_url, pool, state)
}

#[tokio::test]
async fn test_heterogeneous_thundering_herd() {
    let (base_url, _pool, _state) = setup_server().await;

    let mut headers = header::HeaderMap::new();
    let mut val = header::HeaderValue::from_static(TEST_SECRET);
    val.set_sensitive(true);
    headers.insert("X-Keyforge-Secret", val);

    let client = Client::builder()
        .default_headers(headers)
        .pool_max_idle_per_host(NODE_COUNT)
        .build()
        .unwrap();

    println!("🚀 Starting Real-World Stress Test");
    println!("   Target: {}", base_url);

    let kb_corne = load_keyboard("corne");
    let kb_szr = load_keyboard("szr35");

    let weights_std = ScoringWeights::default();
    let weights_alt = ScoringWeights {
        penalty_scissor: 500.0,
        ..Default::default()
    };

    let jobs_config = vec![
        (kb_corne.clone(), weights_std.clone(), "Corne-Std"),
        (kb_corne.clone(), weights_alt.clone(), "Corne-Alt"),
        (kb_szr.clone(), weights_std.clone(), "SZR-Std"),
        (kb_szr.clone(), weights_alt.clone(), "SZR-Alt"),
    ];

    let mut job_ids = Vec::new();

    for (kb, w, label) in jobs_config {
        let req = JobRequest {
            version: PROTOCOL_VERSION,
            definition: kb,
            weights: w,
            params: SearchParams::default(),
            pinned_keys: vec![],
            corpora: vec![CorpusSource {
                id: "default".to_string(),
                weight: 1.0,
                hash: None,
            }],
            cost_matrix: keyforge_protocol::CostMatrixSource::Predefined("cost_matrix.json".into()),
            biometrics: vec![],
            parent_job_id: None,
            baseline_score: None,
            parents: vec![],
        };

        let resp = client
            .post(format!("{}/jobs", base_url))
            .json(&req)
            .send()
            .await
            .expect("Failed to send request");

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            panic!("❌ Job Registration Failed: {} - {}", status, text);
        }

        let body: JobResponse = resp.json().await.expect("Failed to parse JobResponse");
        println!("   📝 Registered {}: {}", label, &body.job_id[0..8]);
        job_ids.push(body.job_id);

        tokio::time::sleep(std::time::Duration::from_millis(1100)).await;
    }

    let start_time = Instant::now();
    let mut handles = Vec::new();

    for i in 0..NODE_COUNT {
        let client_ref = client.clone();
        let url_ref = base_url.clone();
        let job_idx = i % 4;
        let target_job = job_ids[job_idx].clone();

        handles.push(tokio::spawn(async move {
            let node_id = format!("worker-{}-job-{}", i, job_idx);

            let reg_req = NodeRequest {
                version: PROTOCOL_VERSION,
                node_id: node_id.clone(),
                cpu_model: "RustTestCPU".into(),
                cores: 8,
                l2_cache_kb: Some(1024),
                ops_per_sec: 1_000_000.0,
                public_key: None,
            };

            let reg_resp = client_ref
                .post(format!("{}/nodes/register", url_ref))
                .json(&reg_req)
                .send()
                .await
                .expect("Reg failed");

            if !reg_resp.status().is_success() {
                return Err(format!("Node {} reg failed: {}", i, reg_resp.status()));
            }

            for k in 0..RESULTS_PER_NODE {
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                let res_req = ResultSubmission {
                    version: PROTOCOL_VERSION,
                    job_id: target_job.clone(),
                    layout: format!("Q W E R T Y U I O P A S D F G H J K L ; {} {}", i, k),
                    score: 500.0,
                    node_id: node_id.clone(),
                    timestamp,
                    nonce: 0,
                    signature: None,
                };

                let res_resp = client_ref
                    .post(format!("{}/results", url_ref))
                    .json(&res_req)
                    .send()
                    .await
                    .expect("Submit failed");

                let status = res_resp.status();

                if status.is_server_error() {
                    let txt = res_resp.text().await.unwrap_or_default();
                    return Err(format!("Node {} crash: {} - {}", i, status, txt));
                }
            }
            Ok(())
        }));
    }

    let results = join_all(handles).await;
    let duration = start_time.elapsed();

    let mut failures = 0;
    for res in results {
        if let Ok(Err(e)) = res {
            println!("   ⚠️  Worker Error: {}", e);
            failures += 1;
        }
    }

    assert_eq!(failures, 0, "Clients encountered errors");

    let total_reqs = NODE_COUNT * RESULTS_PER_NODE;
    println!(
        "✅ Processed {} requests across 4 distinct jobs in {:.2}s",
        total_reqs,
        duration.as_secs_f32()
    );
    println!(
        "   Throughput: {:.0} req/sec",
        total_reqs as f32 / duration.as_secs_f32()
    );
}
