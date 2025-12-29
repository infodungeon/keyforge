use serde::{Deserialize, Serialize};
use std::io::{Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tempfile::NamedTempFile;
use tracing::{error, warn};

// 1MB Limit for session file
const MAX_SESSION_FILE_SIZE: u64 = 1024 * 1024;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct SessionSnapshot {
    pub keyboard: String,
    pub layout_name: String,
    pub layout_string: String,
    pub corpus: String,
    pub cost_matrix: String,
    pub timestamp: u64,
}

struct AutoSaveState {
    pending: Option<SessionSnapshot>,
    last_save: Instant,
}

pub struct AutoSaveService {
    path: PathBuf,
    state: Arc<Mutex<AutoSaveState>>,
}

impl AutoSaveService {
    pub fn new(root_path: PathBuf) -> Self {
        let path = root_path.join("session.json");

        Self {
            path,
            state: Arc::new(Mutex::new(AutoSaveState {
                pending: None,
                last_save: Instant::now(),
            })),
        }
    }

    pub async fn load(&self) -> Option<SessionSnapshot> {
        if !self.path.exists() {
            return None;
        }

        if let Ok(meta) = tokio::fs::metadata(&self.path).await {
            if meta.len() > MAX_SESSION_FILE_SIZE {
                warn!("Session file too large ({} bytes), ignoring.", meta.len());
                return None;
            }
        }

        match tokio::fs::read_to_string(&self.path).await {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(snap) => Some(snap),
                Err(e) => {
                    warn!("Failed to parse session.json: {}", e);
                    None
                }
            },
            Err(e) => {
                warn!("Failed to read session.json: {}", e);
                None
            }
        }
    }

    pub async fn schedule_save(&self, snapshot: SessionSnapshot) {
        let should_flush = {
            let mut state = self.state.lock().unwrap();
            state.pending = Some(snapshot);
            // Debounce: only flush if 2 seconds passed since last save
            state.last_save.elapsed() > Duration::from_secs(2)
        };

        if should_flush {
            self.flush(false).await;
        }
    }

    /// Flushes pending changes to disk.
    /// If `force` is true, ignores the debounce timer.
    pub async fn flush(&self, force: bool) {
        let snapshot_to_save = {
            let mut state = self.state.lock().unwrap();
            if state.pending.is_none() {
                return;
            }

            if !force && state.last_save.elapsed() < Duration::from_secs(2) {
                return;
            }

            state.last_save = Instant::now();
            state.pending.take() // Take ownership to save
        };

        if let Some(snap) = snapshot_to_save {
            let path = self.path.clone();

            let result = tokio::task::spawn_blocking(move || -> std::io::Result<()> {
                let json = serde_json::to_string_pretty(&snap)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

                let dir = path.parent().unwrap_or_else(|| std::path::Path::new("."));

                // Create temp file in the same directory to attempt atomic rename
                let mut temp_file = NamedTempFile::new_in(dir)?;
                temp_file.write_all(json.as_bytes())?;
                temp_file.flush()?;

                // Atomic persist with cross-device fallback
                match temp_file.persist(&path) {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        // If atomic rename fails (e.g. cross-device link), copy manually
                        let mut source = e.file;
                        source.seek(SeekFrom::Start(0))?;
                        let mut dest = std::fs::File::create(&path)?;
                        std::io::copy(&mut source, &mut dest)?;
                        // Temp file is deleted when source goes out of scope (if not persisted)
                        Ok(())
                    }
                }
            })
            .await;

            match result {
                Ok(Ok(_)) => {}
                Ok(Err(e)) => error!("Failed to save session: {}", e),
                Err(e) => error!("Join error during save: {}", e),
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio;

    #[tokio::test]
    async fn test_load_non_existent() {
        let dir = tempdir().unwrap();
        let service = AutoSaveService::new(dir.path().to_path_buf());
        assert!(service.load().await.is_none());
    }

    #[tokio::test]
    async fn test_load_too_large() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("session.json");
        tokio::fs::write(&path, vec![0u8; MAX_SESSION_FILE_SIZE as usize + 1])
            .await
            .unwrap();
        let service = AutoSaveService::new(dir.path().to_path_buf());
        assert!(service.load().await.is_none());
    }

    #[tokio::test]
    async fn test_load_invalid_json() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("session.json");
        tokio::fs::write(&path, "invalid json").await.unwrap();
        let service = AutoSaveService::new(dir.path().to_path_buf());
        assert!(service.load().await.is_none());
    }

    #[tokio::test]
    async fn test_autosave_debounce_flush() {
        let dir = tempdir().unwrap();
        let service = AutoSaveService::new(dir.path().to_path_buf());
        // Set last_save to the past to trigger immediate flush
        {
            let mut state = service.state.lock().unwrap();
            state.last_save = Instant::now() - Duration::from_secs(3);
        }
        service.schedule_save(SessionSnapshot::default()).await;
        // This should have triggered flush(false) on line 81
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(service.load().await.is_some());
    }

    #[tokio::test]
    async fn test_load_read_error() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("session.json");
        // Create a directory named session.json to force a read error (it's a directory, not a file)
        std::fs::create_dir(&path).unwrap();
        let service = AutoSaveService::new(dir.path().to_path_buf());
        assert!(service.load().await.is_none());
    }
}
