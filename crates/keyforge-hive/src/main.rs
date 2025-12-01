use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use keyforge_core::config::{Config, ScoringWeights};
use keyforge_core::geometry::KeyboardGeometry;
use keyforge_core::job::JobIdentifier;
use keyforge_core::layouts::layout_string_to_bytes;
use keyforge_core::optimizer::mutation;
use keyforge_core::scorer::Scorer;
use serde::{Deserialize, Serialize};
use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePoolOptions, Pool, Row, Sqlite};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{info, warn};

// --- Data Structures ---

#[derive(Clone)]
struct AppState {
    db: Pool<Sqlite>,
}

#[derive(Serialize)]
struct StatusResponse {
    status: String,
    version: String,
    message: String,
}

#[derive(Deserialize, Serialize, Clone)]
struct RegisterJobRequest {
    geometry: KeyboardGeometry,
    weights: ScoringWeights,
    pinned_keys: String,
    corpus_name: String,
}

#[derive(Serialize)]
struct RegisterJobResponse {
    job_id: String,
    is_new: bool,
}

#[derive(Serialize)]
struct JobQueueResponse {
    job_id: Option<String>,
    config: Option<RegisterJobRequest>,
}

#[derive(Deserialize)]
struct SubmitResultRequest {
    job_id: String,
    layout: String,
    score: f32,
    node_id: String,
}

#[derive(Serialize)]
struct PopulationResponse {
    layouts: Vec<String>,
}

// --- Main Entry Point ---

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    info!("🐝 KeyForge Hive is initializing...");

    let db_url = "sqlite://hive.db";
    if !Sqlite::database_exists(db_url).await.unwrap_or(false) {
        info!("Creating database: {}", db_url);
        Sqlite::create_database(db_url).await.unwrap();
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await
        .expect("Failed to connect to database");

    // Initialize Schema
    let schema = include_str!("../schema.sql");
    sqlx::query(schema)
        .execute(&pool)
        .await
        .expect("Failed to run migrations");

    info!("💾 Database connected.");

    let state = Arc::new(AppState { db: pool });

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/jobs", post(register_job))
        .route("/jobs/queue", get(get_job_queue))
        .route("/jobs/{job_id}/population", get(get_population))
        .route("/results", post(submit_result))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("🚀 Hive listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// --- Handlers ---

async fn root() -> &'static str {
    "KeyForge Hive API v0.7"
}

async fn health() -> Json<StatusResponse> {
    Json(StatusResponse {
        status: "ok".to_string(),
        version: "0.7.0".to_string(),
        message: "Genetic Reservoir Active".to_string(),
    })
}

/// Registers a new search configuration (Job).
async fn register_job(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterJobRequest>,
) -> Json<RegisterJobResponse> {
    let job_id = JobIdentifier::from_parts(
        &payload.geometry,
        &payload.weights,
        &payload.pinned_keys,
        &payload.corpus_name,
    )
    .hash;

    let exists = sqlx::query("SELECT 1 FROM jobs WHERE id = ?")
        .bind(&job_id)
        .fetch_optional(&state.db)
        .await
        .unwrap()
        .is_some();

    if exists {
        return Json(RegisterJobResponse {
            job_id,
            is_new: false,
        });
    }

    let geo_json = serde_json::to_string(&payload.geometry).unwrap();
    let weights_json = serde_json::to_string(&payload.weights).unwrap();

    sqlx::query(
        "INSERT INTO jobs (id, geometry_json, weights_json, pinned_keys, corpus_name) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&job_id)
    .bind(geo_json)
    .bind(weights_json)
    .bind(&payload.pinned_keys)
    .bind(&payload.corpus_name)
    .execute(&state.db)
    .await
    .unwrap();

    info!("🆕 Registered Job: {}", &job_id[0..8]);
    Json(RegisterJobResponse {
        job_id,
        is_new: true,
    })
}

/// Returns the most recently created job to a worker node.
async fn get_job_queue(State(state): State<Arc<AppState>>) -> Json<JobQueueResponse> {
    let row = sqlx::query!(
        "SELECT id, geometry_json, weights_json, pinned_keys, corpus_name FROM jobs ORDER BY created_at DESC LIMIT 1"
    )
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    if let Some(r) = row {
        let geometry = serde_json::from_str(&r.geometry_json).unwrap();
        let weights = serde_json::from_str(&r.weights_json).unwrap();

        Json(JobQueueResponse {
            job_id: Some(r.id.expect("Job ID should not be null")),
            config: Some(RegisterJobRequest {
                geometry,
                weights,
                pinned_keys: r.pinned_keys,
                corpus_name: r.corpus_name,
            }),
        })
    } else {
        Json(JobQueueResponse {
            job_id: None,
            config: None,
        })
    }
}

/// Validates a result submitted by a node and stores it if valid.
async fn submit_result(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SubmitResultRequest>,
) -> Result<String, (StatusCode, String)> {
    // 1. Fetch Job Config
    let row = sqlx::query("SELECT geometry_json, weights_json, corpus_name FROM jobs WHERE id = ?")
        .bind(&payload.job_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Job not found".to_string()))?;

    // 2. Decode Config
    let geo_str: String = row.get("geometry_json");
    let weights_str: String = row.get("weights_json");
    let corpus_name: String = row.get("corpus_name");

    let geometry: KeyboardGeometry = serde_json::from_str(&geo_str).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Bad geometry data".into(),
        )
    })?;

    let weights: ScoringWeights = serde_json::from_str(&weights_str)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Bad weights data".into()))?;

    // 3. Resolve Data Files
    let (cost_path, ngram_path) = resolve_paths(&corpus_name).ok_or((
        StatusCode::BAD_REQUEST,
        format!("Unknown corpus: {}", corpus_name),
    ))?;

    // 4. Initialize Scorer (Validation)
    let config = Config {
        weights,
        ..Default::default()
    };

    let scorer = Scorer::new(&cost_path, &ngram_path, &geometry, config, false).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Scorer Init: {}", e),
        )
    })?;

    // 5. Verify Score
    let key_count = geometry.keys.len();
    if payload.layout.chars().count() < key_count {
        return Err((StatusCode::BAD_REQUEST, "Layout string too short".into()));
    }
    let layout_bytes = layout_string_to_bytes(&payload.layout, key_count);
    let pos_map = mutation::build_pos_map(&layout_bytes);

    let details = scorer.score_details(&pos_map, 3000);
    let calc_score = details.layout_score;

    // Tolerance check for float drift
    let tolerance = 5.0;
    let diff = (calc_score - payload.score).abs();

    if diff > tolerance {
        warn!(
            "⚠️ Rejected Score. Claimed: {}, Calc: {}",
            payload.score, calc_score
        );
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Score mismatch. Server calc: {:.2}", calc_score),
        ));
    }

    // 6. Check for New Record (Pre-insertion check)
    let current_best: Option<f32> =
        sqlx::query("SELECT min(score) as min_score FROM results WHERE job_id = ?")
            .bind(&payload.job_id)
            .fetch_optional(&state.db)
            .await
            .unwrap_or(None)
            .and_then(|r| r.get("min_score")); // Retrieve 'min_score' column which might be NULL

    // 7. Save Result
    sqlx::query("INSERT INTO results (job_id, layout, score, node_id) VALUES (?, ?, ?, ?)")
        .bind(&payload.job_id)
        .bind(&payload.layout)
        .bind(payload.score)
        .bind(&payload.node_id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 8. Log Appropriately
    let is_new_record = match current_best {
        Some(best) => payload.score < best,
        None => true, // First result is always a record
    };

    if is_new_record {
        info!(
            "🏆 NEW RECORD! Job: {} | Score: {:.0} | Node: {}",
            &payload.job_id[0..8],
            payload.score,
            payload.node_id
        );
    } else {
        info!(
            "📥 Contribution. Job: {} | Score: {:.0} | Node: {}",
            &payload.job_id[0..8],
            payload.score,
            payload.node_id
        );
    }

    Ok("Result Accepted".to_string())
}

/// Returns the top layouts for a specific job to use for genetic crossover.
async fn get_population(
    State(state): State<Arc<AppState>>,
    Path(job_id): Path<String>,
) -> Json<PopulationResponse> {
    // Fetch top 50 unique layouts by score
    let rows = sqlx::query(
        r#"
        SELECT layout 
        FROM results 
        WHERE job_id = ? 
        GROUP BY layout 
        ORDER BY score ASC 
        LIMIT 50
        "#,
    )
    .bind(job_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let layouts = rows.iter().map(|r| r.get("layout")).collect();

    Json(PopulationResponse { layouts })
}

// Helper to map Corpus Name -> Local Files
fn resolve_paths(name: &str) -> Option<(String, String)> {
    match name {
        "default" | "test_corpus" => Some((
            "data/cost_matrix.csv".to_string(),
            "data/ngrams-all.tsv".to_string(),
        )),
        _ => None,
    }
}
