#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    clippy::unnecessary_debug_formatting,
    clippy::case_sensitive_file_extension_comparisons,
    clippy::print_stdout
)]

#[keyforge_testing_macros::kf_test]
use keyforge_hive::{create_app, infra::db::init_db, state::AppState};
use keyforge_model::geometry::KeyboardDefinition;
use keyforge_protocol::AssetManifestEntry;
use std::fs;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::{Arc, Once};
use testcontainers_modules::redis::Redis;
use testcontainers_modules::testcontainers::runners::AsyncRunner;
use testcontainers_modules::testcontainers::ContainerAsync;
use tokio::net::TcpListener;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use walkdir::WalkDir;

pub const TEST_SECRET: &str = "test_secret_123";

static INIT: Once = Once::new();

pub fn init_tracing() {
    INIT.call_once(|| {
        std::env::set_var("RATE_LIMIT_PER_SEC", "10000");
        std::env::set_var("RATE_LIMIT_BURST", "10000");
        std::env::set_var("HIVE_SECRET", TEST_SECRET);

        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into());

        tracing_subscriber::registry()
            .with(fmt::layer().with_test_writer())
            .with(filter)
            .init();
    });
}

pub fn ensure_test_assets(data_root: &Path) {
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

    // Create dummy keyboards
    for name in ["corne", "szr35"] {
        let path = kb_dir.join(format!("{name}.json"));
        if !path.exists() {
            let json = format!(
                r#"{{
                "name": "{name}", "author": "test", "version": "1", "type": "ortho",
                "geometry": {{
                    "keys": [
                        {{"index": 0, "label": "KC_A", "x":0,"y":0,"hand":0,"finger":1,"row":0,"col":0}},
                        {{"index": 1, "label": "KC_B", "x":1,"y":0,"hand":0,"finger":2,"row":0,"col":1}}
                    ],
                    "prime_slots": [0, 1], "med_slots": [], "low_slots": [], "home_row": 0
                }}
            }}"#
            );
            fs::write(path, json).unwrap();
        }
    }
}

pub fn load_keyboard(data_root: &Path, name: &str) -> KeyboardDefinition {
    let path = data_root.join(format!("user/keyboards/{name}.json"));
    let content = fs::read_to_string(&path).unwrap_or_else(|_| panic!("Failed to read {path:?}"));
    serde_json::from_str(&content).expect("Failed to parse keyboard JSON")
}

// Hydrates the Valkey instance with test assets from the temp directory.
pub async fn hydrate_test_valkey(state: &Arc<AppState>, root: &Path) {
    let system_root = root.join("user"); // Test uses user/ for convenience
    let walker = WalkDir::new(&system_root).follow_links(true);

    for entry in walker.into_iter().filter_map(std::result::Result::ok) {
        if entry.file_type().is_file() {
            let path = entry.path();
            if let Ok(rel) = path.strip_prefix(&system_root) {
                let key_path = rel.to_string_lossy().replace('\\', "/");
                let valkey_key = format!("asset:blob:{key_path}");

                if let Ok(content) = tokio::fs::read(path).await {
                    let packed = rmp_serde::to_vec(
                        &serde_json::from_slice::<serde_json::Value>(&content).unwrap(),
                    )
                    .unwrap();
                    let compressed =
                        zstd::stream::encode_all(std::io::Cursor::new(packed), 0).unwrap();

                    let store_key = if valkey_key.ends_with(".json") {
                        valkey_key.replace(".json", ".mpk.zst")
                    } else {
                        valkey_key
                    };

                    let _ = state.coordinator.set_bin(&store_key, &compressed).await;

                    let entry_id = if key_path.ends_with(".json") {
                        key_path.replace(".json", ".mpk.zst")
                    } else {
                        key_path.clone()
                    };

                    let hash = "test_hash".to_string(); // Mock hash
                    let entry = AssetManifestEntry {
                        id: entry_id,
                        hash,
                        size_bytes: compressed.len() as u64,
                        updated_at: 0,
                    };
                    let _ = state.coordinator.set_manifest_entry(&entry).await;
                }
            }
        }
    }
}

/// Sets up the test environment, including a Redis container and a temporary database.
pub async fn setup_server() -> (
    String,
    Arc<AppState>,
    tempfile::TempDir,
    ContainerAsync<Redis>,
) {
    init_tracing();

    let valkey_node = Redis::default()
        .start()
        .await
        .expect("Failed to start Valkey");
    let valkey_port = valkey_node
        .get_host_port_ipv4(6379)
        .await
        .expect("Failed to get port");
    let valkey_url = format!("redis://127.0.0.1:{valkey_port}");
    std::env::set_var("KEYFORGE_VALKEY_URL", &valkey_url);

    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://keyforge:forge_password@localhost:5432/keyforge_hive".to_string()
    });

    let pool = init_db(&db_url).await;
    let _ = sqlx::query("TRUNCATE results CASCADE").execute(&pool).await;
    let _ = sqlx::query("TRUNCATE nodes CASCADE").execute(&pool).await;
    let _ = sqlx::query("TRUNCATE jobs CASCADE").execute(&pool).await;

    let temp_dir = tempfile::tempdir().unwrap();
    let data_path = temp_dir.path().to_path_buf();
    ensure_test_assets(&data_path);

    let mut config = keyforge_hive::config::AppConfig::mock();
    config.valkey_url = valkey_url;
    config.hive_secret = TEST_SECRET.to_string();

    let state = Arc::new(
        AppState::new(
            pool.clone(),
            data_path.clone(),
            "test-key".to_string(),
            config.clone(),
        )
        .await
        .expect("Failed to init state"),
    );

    hydrate_test_valkey(&state, &data_path).await;

    let app = create_app(state.clone(), &config, data_path);
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let port = addr.port();

    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });

    let base_url = format!("http://127.0.0.1:{port}");
    (base_url, state, temp_dir, valkey_node)
}
