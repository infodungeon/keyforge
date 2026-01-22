// libs/keyforge-persistence/tests/repository_integration.rs
//!
//! Integration tests for keyforge-persistence filesystem operations.
//!
//! These tests verify correct interaction with the filesystem using tempfile
//! for isolation. They cover:
//! - `UserRepo` layout/biometrics/keyboard persistence
//! - `AutoSaveService` session save/load/debounce

use keyforge_model::constants::MAX_SESSION_FILE_SIZE;
use keyforge_model::geometry::KeyboardDefinition;
use keyforge_persistence::store::autosave::{AutoSaveService, SessionSnapshot};
use keyforge_persistence::UserRepo;
use keyforge_protocol::BiometricSample;
use std::fs;
use std::time::{Duration, Instant};
use tempfile::tempdir;

// ============================================================================
// UserRepo Integration Tests
// ============================================================================

/// Intent: Verify basic layout CRUD lifecycle.
/// Expected: Layouts can be saved, retrieved, and deleted correctly.
#[test]
fn user_repo_layout_lifecycle() {
    let dir = tempdir().unwrap();
    let repo = UserRepo::new(dir.path().to_path_buf());

    repo.save_layout("kb1", "name1", "layout1").unwrap();
    let layouts = repo.get_layouts("kb1");
    assert_eq!(layouts.get("name1").unwrap(), "layout1");

    repo.delete_layout("kb1", "name1").unwrap();
    assert!(repo.get_layouts("kb1").is_empty());
}

/// Intent: Verify biometric sample recording and retrieval.
/// Expected: Samples are persisted and can be retrieved; reset clears them.
#[test]
fn user_repo_biometrics_lifecycle() {
    let dir = tempdir().unwrap();
    let repo = UserRepo::new(dir.path().to_path_buf());

    let sample = BiometricSample {
        bigram: "th".into(),
        ms: 100.0,
        timestamp: 0,
    };
    repo.record_biometrics(vec![sample]).unwrap();

    let biometrics = repo.get_biometrics();
    assert_eq!(biometrics.len(), 1);
    assert_eq!(biometrics[0].bigram, "th");

    repo.reset_biometrics().unwrap();
    assert!(repo.get_biometrics().is_empty());
}

/// Intent: Verify profile generation requires minimum samples.
/// Expected: Fails with insufficient data, succeeds after threshold met.
#[test]
fn user_repo_profile_generation() {
    use keyforge_model::constants::MIN_BIOMETRIC_SAMPLES;

    let dir = tempdir().unwrap();
    let repo = UserRepo::new(dir.path().to_path_buf());

    // Should fail with insufficient data
    assert!(repo.generate_profile().is_err());

    // Fill minimum samples
    let samples = (0..MIN_BIOMETRIC_SAMPLES)
        .map(|_| BiometricSample {
            bigram: "th".into(),
            ms: 100.0,
            timestamp: 0,
        })
        .collect();
    repo.record_biometrics(samples).unwrap();
    assert!(repo.generate_profile().is_ok());
}

/// Intent: Verify keyboard definition persistence.
/// Expected: Saved definition creates file at expected path.
#[test]
fn user_repo_keyboard_definition() {
    let dir = tempdir().unwrap();
    let repo = UserRepo::new(dir.path().to_path_buf());

    let def = KeyboardDefinition::default();
    repo.save_keyboard_definition("test_kb", &def).unwrap();
    assert!(dir.path().join("user/keyboards/test_kb.json").exists());
}

/// Intent: Verify graceful handling of corrupted data files.
/// Expected: Returns defaults on JSON corruption; skips broken JSONL lines.
#[test]
fn user_repo_corruption_handling() {
    let dir = tempdir().unwrap();
    let repo = UserRepo::new(dir.path().to_path_buf());
    let path = dir.path().join("user/user_layouts.json");
    fs::create_dir_all(path.parent().unwrap()).unwrap();

    // Broken JSON in layout store - get_layouts should return empty
    fs::write(&path, "{ invalid json }").unwrap();
    let layouts = repo.get_layouts("any_kb");
    assert!(layouts.is_empty(), "Should return empty on corruption");

    // Broken line in biometric JSONL
    let stats_path = dir.path().join("user/user_stats.jsonl");
    let line1 = r#"{"bigram":"th","ms":100.0,"timestamp":123}"#;
    let line2 = r#"{"bigram":"he","ms":150.0,"timestamp":124}"#;
    fs::write(&stats_path, format!("{line1}\n{{broken line}}\n{line2}")).unwrap();
    let biometrics = repo.get_biometrics();
    assert_eq!(biometrics.len(), 2, "Should skip broken lines in JSONL");
}

/// Intent: Verify path traversal attacks are sanitized.
/// Expected: Malicious filenames are sanitized to safe names.
#[test]
fn user_repo_keyboard_filename_sanitization() {
    let dir = tempdir().unwrap();
    let repo = UserRepo::new(dir.path().to_path_buf());
    let def = KeyboardDefinition::default();

    // Use a "dirty" filename
    repo.save_keyboard_definition("../../../etc/passwd", &def)
        .unwrap();

    // Verify it was sanitized (should be in keyboards dir, not escape)
    let exists = fs::read_dir(dir.path().join("user/keyboards"))
        .unwrap()
        .any(|e| e.unwrap().file_name().to_str().unwrap().contains("passwd"));
    assert!(exists);
}

// ============================================================================
// AutoSaveService Integration Tests
// ============================================================================

/// Intent: Verify load returns None for non-existent session.
/// Expected: Returns Ok(None) when no session file exists.
#[tokio::test]
async fn autosave_load_non_existent() {
    let dir = tempdir().unwrap();
    let service = AutoSaveService::new(dir.path().to_path_buf());
    assert!(service.load().await.unwrap().is_none());
}

/// Intent: Verify oversized session files are rejected.
/// Expected: Returns Ok(None) when file exceeds size limit.
#[tokio::test]
async fn autosave_load_too_large() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("session.json");
    tokio::fs::write(&path, vec![0u8; MAX_SESSION_FILE_SIZE as usize + 1])
        .await
        .unwrap();
    let service = AutoSaveService::new(dir.path().to_path_buf());
    assert!(service.load().await.unwrap().is_none());
}

/// Intent: Verify invalid JSON is gracefully rejected.
/// Expected: Returns Ok(None) when file contains invalid JSON.
#[tokio::test]
async fn autosave_load_invalid_json() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("session.json");
    tokio::fs::write(&path, "invalid json").await.unwrap();
    let service = AutoSaveService::new(dir.path().to_path_buf());
    assert!(service.load().await.unwrap().is_none());
}

/// Intent: Verify debounce logic triggers flush after interval.
/// Expected: Session is persisted when debounce interval expires.
#[tokio::test]
async fn autosave_debounce_flush() {
    let dir = tempdir().unwrap();
    let service = AutoSaveService::new(dir.path().to_path_buf());
    // Set last_save to the past to trigger immediate flush
    {
        let mut state = service.state.lock().unwrap();
        state.last_save = Instant::now().checked_sub(Duration::from_secs(3)).unwrap();
    }
    service.schedule_save(SessionSnapshot::default()).await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(service.load().await.unwrap().is_some());
}

/// Intent: Verify read errors are propagated.
/// Expected: Returns Err when session.json is a directory.
#[tokio::test]
async fn autosave_load_read_error() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("session.json");
    // Create a directory named session.json to force a read error
    std::fs::create_dir(&path).unwrap();
    let service = AutoSaveService::new(dir.path().to_path_buf());
    assert!(service.load().await.is_err());
}

/// Intent: Verify legacy session format compatibility.
/// Expected: Sessions without wrapper envelope are loaded correctly.
#[tokio::test]
async fn autosave_load_legacy_format() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("session.json");
    let snapshot = SessionSnapshot {
        keyboard: "legacy".into(),
        ..Default::default()
    };
    tokio::fs::write(&path, serde_json::to_string(&snapshot).unwrap())
        .await
        .unwrap();

    let service = AutoSaveService::new(dir.path().to_path_buf());
    let loaded = service.load().await.unwrap().unwrap();
    assert_eq!(loaded.keyboard, "legacy");
}

/// Intent: Verify checksum validation rejects tampered data.
/// Expected: Returns Ok(None) when checksum doesn't match.
#[tokio::test]
async fn autosave_load_checksum_mismatch() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("session.json");
    let persisted = serde_json::json!({
        "snapshot": SessionSnapshot::default(),
        "checksum": "wrong"
    });
    tokio::fs::write(&path, serde_json::to_string(&persisted).unwrap())
        .await
        .unwrap();

    let service = AutoSaveService::new(dir.path().to_path_buf());
    assert!(service.load().await.unwrap().is_none());
}

/// Intent: Verify force flush and empty pending behavior.
/// Expected: Force flush persists immediately; empty pending is no-op.
#[tokio::test]
async fn autosave_flush_force_and_empty() {
    let dir = tempdir().unwrap();
    let service = AutoSaveService::new(dir.path().to_path_buf());

    // Flush empty should be no-op
    service.flush(true).await;
    assert!(!dir.path().join("session.json").exists());

    // Force flush ignoring debounce
    service
        .schedule_save(SessionSnapshot {
            keyboard: "forced".into(),
            ..Default::default()
        })
        .await;
    service.flush(true).await;
    let loaded = service.load().await.unwrap().unwrap();
    assert_eq!(loaded.keyboard, "forced");
}

/// Intent: Verify flush handles missing pending data.
/// Expected: Non-force flush with no pending data is no-op.
#[tokio::test]
async fn autosave_flush_empty_pending() {
    let dir = tempdir().unwrap();
    let service = AutoSaveService::new(dir.path().to_path_buf());
    service.flush(false).await;
    // Should complete without error
}
