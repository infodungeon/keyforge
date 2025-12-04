use crate::models::SearchUpdate;
use keyforge_core::optimizer::ProgressCallback;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager, Window};
use tempfile::NamedTempFile;

/// Resolves the absolute path to the 'data' directory.
/// Checks bundled resources first, then AppData, then dev paths.
pub fn get_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    // 1. Production: Check Resource Directory (Bundled with App)
    if let Ok(resource_path) = app.path().resource_dir() {
        let bundled = resource_path.join("data");
        if bundled.exists() {
            return Ok(bundled);
        }
    }

    // 2. Production/Dev: Check App Data Directory (User Writable)
    if let Ok(app_data) = app.path().app_data_dir() {
        let data = app_data.join("data");
        if data.exists() {
            return Ok(data);
        }
    }

    // 3. Development Fallback (Cargo run from crate root or workspace root)
    let dev_paths = [
        "data",       // ui/src-tauri/data
        "../data",    // ui/data
        "../../data", // workspace root data
    ];

    for p in dev_paths {
        let path = PathBuf::from(p);
        if path.exists() && path.is_dir() && path.join("keyboards").exists() {
            return Ok(path);
        }
    }

    // 4. Last Resort: Return AppData path even if it doesn't exist yet.
    if let Ok(app_data) = app.path().app_data_dir() {
        let data = app_data.join("data");
        return Ok(data);
    }

    Err("Could not resolve data directory.".into())
}

/// Safely writes content to a file using atomic rename strategy via `tempfile` crate.
/// This avoids partial writes if the app crashes mid-save.
pub fn atomic_write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> std::io::Result<()> {
    let path = path.as_ref();
    let dir = path.parent().ok_or(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "Invalid path",
    ))?;

    // Create temp file in the SAME directory to ensure atomic rename works (same filesystem)
    let mut temp_file = NamedTempFile::new_in(dir)?;
    temp_file.write_all(contents.as_ref())?;

    // Persist ensures the file is not deleted, then we rename it over the target
    temp_file.persist(path).map_err(|e| e.error)?;

    Ok(())
}

/// Bridge for Core Optimizer -> UI Event
/// Bridges the synchronous optimizer loop with the asynchronous Tauri event system.
pub struct TauriBridge {
    pub window: Window,
    pub stop_signal: Arc<Mutex<bool>>,
}

impl ProgressCallback for TauriBridge {
    fn on_progress(&self, epoch: usize, score: f32, best_layout: &[u16], ips: f32) -> bool {
        // 1. Check stop signal
        if let Ok(guard) = self.stop_signal.lock() {
            if *guard {
                return false;
            }
        }

        // 2. Convert u16 layout to string for UI preview (simplified ASCII mapping for speed)
        let bytes: Vec<u8> = best_layout
            .iter()
            .map(|&c| if c < 255 { c as u8 } else { b'?' })
            .collect();
        let layout_str = String::from_utf8_lossy(&bytes).to_string();

        // 3. Emit event
        let _ = self.window.emit(
            "search-update",
            SearchUpdate {
                epoch,
                score,
                layout: layout_str,
                ips,
            },
        );

        true
    }
}
