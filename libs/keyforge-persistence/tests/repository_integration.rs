// libs/keyforge-persistence/tests/repository_integration.rs
//
// Integration tests for keyforge-persistence filesystem operations.
//
// These tests verify correct interaction with the filesystem using tempfile
// for isolation. They cover:
// - `UserRepo` layout/biometrics/keyboard persistence
// - `AutoSaveService` session save/load/debounce

use keyforge_model::constants::MAX_SESSION_FILE_SIZE;
use keyforge_model::geometry::KeyboardDefinition;
use keyforge_model::types::path::SafePath;
use keyforge_persistence::store::autosave::{AutoSaveService, SessionSnapshot};
use keyforge_persistence::UserRepo;
use keyforge_protocol::BiometricSample;
use std::fs;
use std::time::{Duration, Instant};
use tempfile::tempdir;

#[keyforge_testing_macros::kf_test]
mod integration_tests {
    use super::*;

    // =============================================================================
    // UserRepo Integration Tests
    // ============================================================================

    /// Intent: Verify basic layout CRUD lifecycle.
    /// Expected: Layouts can be saved, retrieved, and deleted correctly.
    #[test]
    fn user_repo_layout_lifecycle() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let repo = UserRepo::new(SafePath::from_trusted_root_path(dir.path().to_path_buf()));

        repo.save_layout("kb1", "name1", "layout1")?;
        let layouts = repo.get_layouts("kb1");
        assert_eq!(
            layouts
                .get("name1")
                .ok_or_else(|| anyhow::anyhow!("missing layout"))?,
            "layout1"
        );

        repo.delete_layout("kb1", "name1")?;
        assert!(repo.get_layouts("kb1").is_empty());
        Ok(())
    }

    /// Intent: Verify biometric sample recording and retrieval.
    /// Expected: Samples are persisted and can be retrieved; reset clears them.
    #[test]
    fn user_repo_biometrics_lifecycle() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let repo = UserRepo::new(SafePath::from_trusted_root_path(dir.path().to_path_buf()));

        let sample = BiometricSample {
            key_a: 116,
            key_b: 104,
            duration_ms: 100,
        };
        repo.record_biometrics(vec![sample])?;

        let biometrics = repo.get_biometrics();
        assert_eq!(biometrics.len(), 1);
        assert_eq!(biometrics[0].key_a, 116);

        repo.reset_biometrics()?;
        assert!(repo.get_biometrics().is_empty());
        Ok(())
    }

    /// Intent: Verify profile generation requires minimum samples.
    /// Expected: Fails with insufficient data, succeeds after threshold met.
    #[test]
    fn user_repo_profile_generation() -> anyhow::Result<()> {
        use keyforge_compute::biometrics::StreamingProfileBuilder;

        let dir = tempdir()?;
        let repo = UserRepo::new(SafePath::from_trusted_root_path(dir.path().to_path_buf()));

        // 1. Should fail with insufficient data (count < 5)
        let mut builder = StreamingProfileBuilder::new();
        repo.load_stats_streaming(|s| builder.add_sample(&s))?;
        assert!(builder.sample_count < 5);

        // 2. Fill minimum samples
        let samples = (0..10)
            .map(|_| BiometricSample {
                key_a: 116,
                key_b: 104,
                duration_ms: 100,
            })
            .collect();
        repo.record_biometrics(samples)?;

        let mut builder = StreamingProfileBuilder::new();
        repo.load_stats_streaming(|s| builder.add_sample(&s))?;
        assert!(builder.sample_count >= 5);

        let cost_model = builder.build_model();
        assert!(repo.save_personal_cost_model(&cost_model).is_ok());
        assert!(dir.path().join("user/personal_cost.json").exists());
        Ok(())
    }

    /// Intent: Verify keyboard definition persistence.
    /// Expected: Saved definition creates file at expected path.
    #[test]
    fn user_repo_keyboard_definition() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let repo = UserRepo::new(SafePath::from_trusted_root_path(dir.path().to_path_buf()));

        let def = KeyboardDefinition::default();
        repo.save_keyboard_definition("test_kb", &def)?;
        assert!(dir.path().join("user/keyboards/test_kb.json").exists());
        Ok(())
    }

    /// Intent: Verify graceful handling of corrupted data files.
    /// Expected: Returns defaults on JSON corruption; skips broken JSONL lines.
    #[test]
    fn user_repo_corruption_handling() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let repo = UserRepo::new(SafePath::from_trusted_root_path(dir.path().to_path_buf()));
        let path = dir.path().join("user/user_layouts.json");
        fs::create_dir_all(
            path.parent()
                .ok_or_else(|| anyhow::anyhow!("missing parent"))?,
        )?;

        // Broken JSON in layout store - get_layouts should return empty
        fs::write(&path, "{ invalid json }")?;
        let layouts = repo.get_layouts("any_kb");
        assert!(layouts.is_empty(), "Should return empty on corruption");

        // Broken line in biometric JSONL
        let stats_path = dir.path().join("user/user_stats.jsonl");
        let line1 = r#"{"key_a":116,"key_b":104,"duration_ms":100}"#;
        let line2 = r#"{"key_a":116,"key_b":105,"duration_ms":150}"#;
        fs::write(&stats_path, format!("{line1}\n{{broken line}}\n{line2}"))?;
        let biometrics = repo.get_biometrics();
        assert_eq!(biometrics.len(), 2, "Should skip broken lines in JSONL");
        Ok(())
    }

    /// Intent: Verify path traversal attacks are sanitized.
    /// Expected: Malicious filenames are sanitized to safe names.
    #[test]
    fn user_repo_keyboard_filename_sanitization() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let repo = UserRepo::new(SafePath::from_trusted_root_path(dir.path().to_path_buf()));
        let def = KeyboardDefinition::default();

        // Use a "dirty" filename
        repo.save_keyboard_definition("../../../etc/passwd", &def)?;

        // Verify it was sanitized (should be in keyboards dir, not escape)
        let exists = fs::read_dir(dir.path().join("user/keyboards"))?.any(|e| {
            if let Ok(entry) = e {
                entry
                    .file_name()
                    .to_str()
                    .is_some_and(|s| s.contains("passwd"))
            } else {
                false
            }
        });
        assert!(exists);
        Ok(())
    }

    // =============================================================================
    // AutoSaveService Integration Tests
    // ============================================================================

    /// Intent: Verify load returns None for non-existent session.
    /// Expected: Returns Ok(None) when no session file exists.
    #[tokio::test]
    async fn autosave_load_non_existent() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let service = AutoSaveService::new(dir.path());
        assert!(service.load().await?.is_none());
        Ok(())
    }

    /// Intent: Verify oversized session files are rejected.
    /// Expected: Returns Ok(None) when file exceeds size limit.
    #[tokio::test]
    async fn autosave_load_too_large() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let path = dir.path().join("session.json");
        tokio::fs::write(&path, vec![0u8; MAX_SESSION_FILE_SIZE as usize + 1]).await?;
        let service = AutoSaveService::new(dir.path());
        assert!(service.load().await?.is_none());
        Ok(())
    }

    /// Intent: Verify invalid JSON is gracefully rejected.
    /// Expected: Returns Ok(None) when file contains invalid JSON.
    #[tokio::test]
    async fn autosave_load_invalid_json() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let path = dir.path().join("session.json");
        tokio::fs::write(&path, "invalid json").await?;
        let service = AutoSaveService::new(dir.path());
        assert!(service.load().await?.is_none());
        Ok(())
    }

    /// Intent: Verify debounce logic triggers flush after interval.
    /// Expected: Session is persisted when debounce interval expires.
    #[tokio::test]
    async fn autosave_debounce_flush() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let service = AutoSaveService::new(dir.path());
        // Set last_save to the past to trigger immediate flush
        {
            let mut state = service
                .state
                .lock()
                .map_err(|e| anyhow::anyhow!("lock failed: {e}"))?;
            state.last_save = Instant::now()
                .checked_sub(Duration::from_secs(3))
                .ok_or_else(|| anyhow::anyhow!("time math failed"))?;
        }
        service.schedule_save(SessionSnapshot::default()).await;
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(service.load().await?.is_some());
        Ok(())
    }

    /// Intent: Verify read errors are propagated.
    /// Expected: Returns Err when session.json is a directory.
    #[tokio::test]
    async fn autosave_load_read_error() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let path = dir.path().join("session.json");
        // Create a directory named session.json to force a read error
        std::fs::create_dir(&path)?;
        let service = AutoSaveService::new(dir.path());
        assert!(service.load().await.is_err());
        Ok(())
    }

    /// Intent: Verify legacy session format compatibility.
    /// Expected: Sessions without wrapper envelope are loaded correctly.
    #[tokio::test]
    async fn autosave_load_legacy_format() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let path = dir.path().join("session.json");
        let snapshot = SessionSnapshot {
            keyboard: "legacy".into(),
            ..Default::default()
        };
        tokio::fs::write(&path, serde_json::to_string(&snapshot)?).await?;

        let service = AutoSaveService::new(dir.path());
        let loaded = service
            .load()
            .await?
            .ok_or_else(|| anyhow::anyhow!("missing session"))?;
        assert_eq!(loaded.keyboard, "legacy");
        Ok(())
    }

    /// Intent: Verify checksum validation rejects tampered data.
    /// Expected: Returns Ok(None) when checksum doesn't match.
    #[tokio::test]
    async fn autosave_load_checksum_mismatch() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let path = dir.path().join("session.json");
        let persisted = serde_json::json!({
            "snapshot": SessionSnapshot::default(),
            "checksum": "wrong"
        });
        tokio::fs::write(&path, serde_json::to_string(&persisted)?).await?;

        let service = AutoSaveService::new(dir.path());
        assert!(service.load().await?.is_none());
        Ok(())
    }

    /// Intent: Verify force flush and empty pending behavior.
    /// Expected: Force flush persists immediately; empty pending is no-op.
    #[tokio::test]
    async fn autosave_flush_force_and_empty() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let service = AutoSaveService::new(dir.path());

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
        let loaded = service
            .load()
            .await?
            .ok_or_else(|| anyhow::anyhow!("missing session"))?;
        assert_eq!(loaded.keyboard, "forced");
        Ok(())
    }

    /// Intent: Verify flush handles missing pending data.
    /// Expected: Non-force flush with no pending data is no-op.
    #[tokio::test]
    async fn autosave_flush_empty_pending() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let service = AutoSaveService::new(dir.path());
        service.flush(false).await;
        Ok(())
        // Should complete without error
    }
}
