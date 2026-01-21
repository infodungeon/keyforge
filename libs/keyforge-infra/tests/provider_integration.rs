// libs/keyforge-infra/tests/provider_integration.rs
//
//! Integration tests for FsProvider.
//! These tests require tempfile/filesystem/async access and validate asset loading contracts.

use keyforge_core::loader::AssetLoader;
use keyforge_infra::asset::AssetServerProvider;
use keyforge_infra::FsProvider;
use keyforge_model::config::CorpusSource;
use keyforge_model::KeyboardDefinition;
use std::fs::{self, File};
use std::sync::Arc;

async fn setup_root() -> (tempfile::TempDir, FsProvider) {
    let temp = tempfile::tempdir().unwrap();
    let provider = FsProvider::new(temp.path().to_path_buf());
    (temp, provider)
}

// ============================================================================
// JSON Loading Tests
// ============================================================================

/// Intent: Verify FsProvider loads JSON assets from various path formats.
/// Expected Result: Assets load by name, with extension, and by absolute path.
#[tokio::test]
async fn test_fs_provider_json_loading() {
    let (temp, provider) = setup_root().await;
    let root = temp.path();
    assert_eq!(provider.root(), &root.to_path_buf());

    let kb_dir = root.join("user/keyboards");
    fs::create_dir_all(&kb_dir).unwrap();

    let kb_json = r#"{
        "meta": { "name": "Test" },
        "geometry": { "keys": [{"x":0, "y":0, "hand":0, "finger":1, "row":0}], "prime_slots":[0], "med_slots":[], "low_slots":[], "home_row": 0 }
    }"#;
    fs::write(kb_dir.join("test.json"), kb_json).unwrap();

    // 1. Standard load
    let res: Arc<KeyboardDefinition> = provider.load("test").await.unwrap();
    assert_eq!(res.meta.name, "Test");

    // 2. Load with extension
    let res: Arc<KeyboardDefinition> = provider.load("test.json").await.unwrap();
    assert_eq!(res.meta.name, "Test");

    // 3. Direct path (absolute)
    let abs_path = kb_dir.join("test.json");
    let res: Arc<KeyboardDefinition> = provider
        .load(abs_path.to_str().unwrap())
        .await
        .unwrap();
    assert_eq!(res.meta.name, "Test");

    // 4. Path traversal attempt
    assert!(provider.load::<KeyboardDefinition>("../secret").await.is_err());
}

/// Intent: Verify FsProvider rejects invalid JSON content.
/// Expected Result: Returns parse error for malformed JSON.
#[tokio::test]
async fn test_fs_provider_json_errors() {
    let (temp, provider) = setup_root().await;
    let root = temp.path();

    let kb_dir = root.join("user/keyboards");
    fs::create_dir_all(&kb_dir).unwrap();

    // Invalid JSON content
    fs::write(kb_dir.join("invalid.json"), "{ broken }").unwrap();
    assert!(provider.load::<KeyboardDefinition>("invalid").await.is_err());
}

// ============================================================================
// Binary Loading Tests
// ============================================================================

/// Intent: Verify FsProvider loads zstd-compressed MessagePack assets.
/// Expected Result: Binary assets are decompressed and deserialized correctly.
#[tokio::test]
async fn test_fs_provider_binary_loading() {
    let (temp, provider) = setup_root().await;
    let root = temp.path();

    let kb_dir = root.join("system/keyboards/models");
    fs::create_dir_all(&kb_dir).unwrap();

    let kb = KeyboardDefinition {
        meta: keyforge_model::geometry::KeyboardMeta {
            name: "Binary".into(),
            ..Default::default()
        },
        geometry: keyforge_model::geometry::KeyboardGeometry {
            keys: vec![keyforge_model::geometry::KeyNode {
                hand: keyforge_model::types::HandIndex::LEFT,
                finger: keyforge_model::types::FingerIndex::INDEX,
                row: keyforge_model::types::RowIndex(0),
                ..Default::default()
            }],
            prime_slots: vec![keyforge_model::types::KeyIndex(0)],
            med_slots: vec![],
            low_slots: vec![],
            home_row: 0,
        },
        ..Default::default()
    };

    let path = kb_dir.join("test.mpk.zst");
    {
        let file = File::create(&path).unwrap();
        let mut encoder = zstd::Encoder::new(file, 3).unwrap();
        rmp_serde::encode::write(&mut encoder, &kb).unwrap();
        encoder.finish().unwrap();
    }

    let res: Arc<KeyboardDefinition> = provider.load("test").await.unwrap();
    assert_eq!(res.meta.name, "Binary");
}

/// Intent: Verify FsProvider rejects corrupt binary files.
/// Expected Result: Returns error for invalid zstd content.
#[tokio::test]
async fn test_fs_provider_binary_errors() {
    let (temp, provider) = setup_root().await;
    let root = temp.path();

    let kb_dir = root.join("system/keyboards/models");
    fs::create_dir_all(&kb_dir).unwrap();

    // Corrupt binary
    fs::write(kb_dir.join("corrupt.mpk.zst"), "not a zstd file").unwrap();
    assert!(provider
        .load::<KeyboardDefinition>("corrupt")
        .await
        .is_err());
}

/// Intent: Verify system JSON assets load correctly.
/// Expected Result: JSON in system/ directory is accessible.
#[tokio::test]
async fn test_fs_provider_system_json() {
    let (temp, provider) = setup_root().await;
    let root = temp.path();

    let sys_dir = root.join("system/keyboards");
    fs::create_dir_all(&sys_dir).unwrap();
    fs::write(
        sys_dir.join("sys.json"),
        r#"{"meta":{"name":"SysJSON"}, "geometry":{"keys":[{"x":0,"y":0,"hand":0,"finger":1,"row":0}],"prime_slots":[0],"med_slots":[],"low_slots":[],"home_row":0}}"#,
    )
    .unwrap();

    let res: Arc<KeyboardDefinition> = provider.load("sys").await.unwrap();
    assert_eq!(res.meta.name, "SysJSON");
}

// ============================================================================
// Corpus Tests
// ============================================================================

/// Intent: Verify FsProvider computes corpus hash from system directory.
/// Expected Result: Hash is non-empty for valid corpus files.
#[tokio::test]
async fn test_fs_provider_corpus_hash_system() {
    let (temp, provider) = setup_root().await;
    let root = temp.path();

    let corp_dir = root.join("system/corpora/en");
    fs::create_dir_all(&corp_dir).unwrap();
    // Create valid empty zstd file
    let path = corp_dir.join("1grams.mpk.zst");
    let file = File::create(&path).unwrap();
    let encoder = zstd::Encoder::new(file, 3).unwrap();
    encoder.finish().unwrap();

    let hash = provider.get_corpus_hash("en").await.unwrap();
    assert!(!hash.is_empty());
}

/// Intent: Verify FsProvider computes corpus hash from user directory.
/// Expected Result: Hash is non-empty for valid corpus files.
#[tokio::test]
async fn test_fs_provider_corpus_hash() {
    let (temp, provider) = setup_root().await;
    let root = temp.path();

    let corp_dir = root.join("user/corpora/en");
    fs::create_dir_all(&corp_dir).unwrap();
    fs::write(corp_dir.join("1grams.json"), "[]").unwrap();

    let hash = provider.get_corpus_hash("en").await.unwrap();
    assert!(!hash.is_empty());
}

/// Intent: Verify FsProvider loads corpus from system directory.
/// Expected Result: Corpus character frequencies are populated correctly.
#[tokio::test]
async fn test_fs_provider_load_corpus_system() {
    let (temp, provider) = setup_root().await;
    let root = temp.path();

    let corp_dir = root.join("system/corpora/en");
    fs::create_dir_all(&corp_dir).unwrap();

    let data = vec![serde_json::json!({"char": "a", "freq": 100})];
    let path = corp_dir.join("1grams.mpk.zst");
    {
        let file = File::create(&path).unwrap();
        let mut encoder = zstd::Encoder::new(file, 3).unwrap();
        rmp_serde::encode::write(&mut encoder, &data).unwrap();
        encoder.finish().unwrap();
    }

    let sources = vec![CorpusSource {
        id: "en".into(),
        weight: 1.0,
        hash: None,
    }];
    let corp = provider.load_corpus(&sources).await.unwrap();
    assert_eq!(corp.char_freqs[97], 100);
}

/// Intent: Verify FsProvider loads corpus from user directory.
/// Expected Result: Corpus character frequencies are populated correctly.
#[tokio::test]
async fn test_fs_provider_load_corpus() {
    let (temp, provider) = setup_root().await;
    let root = temp.path();

    let corp_dir = root.join("user/corpora/en");
    fs::create_dir_all(&corp_dir).unwrap();
    fs::write(corp_dir.join("1grams.json"), r#"[{"char": "a", "freq": 100}]"#).unwrap();

    let sources = vec![CorpusSource {
        id: "en".into(),
        weight: 1.0,
        hash: None,
    }];
    let corp = provider.load_corpus(&sources).await.unwrap();
    assert_eq!(corp.char_freqs[97], 100);
}

// ============================================================================
// Server Provider Tests
// ============================================================================

/// Intent: Verify AssetServerProvider trait implementation.
/// Expected Result: Manifest retrieval and file content access work correctly.
#[tokio::test]
async fn test_fs_provider_server_provider() {
    let (temp, provider) = setup_root().await;
    let root = temp.path();

    let sys_dir = root.join("system");
    fs::create_dir_all(&sys_dir).unwrap();
    fs::write(sys_dir.join("test.txt"), "hello").unwrap();

    let _manifest = provider.get_manifest().await;

    let content = provider.get_file_content("system/test.txt").await.unwrap();
    assert_eq!(content, "hello");

    assert!(provider.get_file_content("missing").await.is_none());
    assert!(provider.get_file_content("../secret").await.is_none());
}

// ============================================================================
// Security Tests
// ============================================================================

/// Intent: Verify safe_join prevents path traversal attacks.
/// Expected Result: Attempts to escape root directory are rejected.
#[tokio::test]
async fn test_fs_provider_safe_join_error() {
    let (_temp, provider) = setup_root().await;
    // Attempt to load asset with null byte or invalid path char if resolver allows
    // PathResolver::safe_join usually catches ".."
    assert!(provider
        .load::<KeyboardDefinition>("../../../etc/passwd")
        .await
        .is_err());
}
