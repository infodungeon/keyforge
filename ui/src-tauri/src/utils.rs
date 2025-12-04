use crate::models::SearchUpdate;
use keyforge_core::optimizer::ProgressCallback;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager, Window};

/// Resolves the absolute path to the 'data' directory.
///
/// Priority Order:
/// 1. **Resource Directory**: Bundled assets (read-only in installed apps).
/// 2. **App Data Directory**: User-writable standard location (XDG/AppData).
/// 3. **Dev Paths**: Relative paths for `cargo run` in workspace.
/// 4. **Fallback**: Returns the App Data path (even if missing) so it can be created.
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
        // FIXED: Collapsed nested if statement
        if path.exists() && path.is_dir() && path.join("keyboards").exists() {
            return Ok(path);
        }
    }

    // 4. Last Resort: Return AppData path even if it doesn't exist yet.
    if let Ok(app_data) = app.path().app_data_dir() {
        let data = app_data.join("data");
        return Ok(data);
    }

    Err("Could not resolve data directory. Please ensure the 'data' folder is bundled or available.".into())
}

/// safely writes content to a file using atomic rename strategy.
/// 1. Write to {filename}.tmp
/// 2. Sync to disk
/// 3. Rename {filename}.tmp -> {filename}
pub fn atomic_write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> std::io::Result<()> {
    let path = path.as_ref();
    // FIXED: Removed unused `dir` variable

    // Create a temp file in the same directory (ensures same filesystem for rename)
    let temp_path = path.with_extension("tmp");

    {
        let mut file = fs::File::create(&temp_path)?;
        std::io::Write::write_all(&mut file, contents.as_ref())?;
        // CRITICAL: Ensure data hits the physical disk
        file.sync_all()?;
    }

    // Atomic replacement
    fs::rename(&temp_path, path)?;
    Ok(())
}

/// Bridge for Core Optimizer -> UI Event
///
/// This struct implements the `ProgressCallback` trait required by `keyforge-core`.
/// It bridges the synchronous optimizer loop with the asynchronous Tauri event system.
pub struct TauriBridge {
    pub window: Window,
    pub stop_signal: Arc<Mutex<bool>>,
}

impl ProgressCallback for TauriBridge {
    // CHANGED: Signature accepts &[u16] layout from Phase 2 Core
    fn on_progress(&self, epoch: usize, score: f32, best_layout: &[u16], ips: f32) -> bool {
        // 1. Check if the user requested a stop via the UI
        if let Ok(guard) = self.stop_signal.lock() {
            if *guard {
                return false; // Stop the optimizer
            }
        }

        // 2. Convert u16 layout to string for UI preview
        // For the live preview, mapping to standard ASCII (u8) is usually sufficient
        // and much faster than doing a full Registry lookup 100s of times a second.
        // Complex macros (>255) will wrap, but visual feedback remains responsive.
        let bytes: Vec<u8> = best_layout.iter().map(|&c| c as u8).collect();
        let layout_str = String::from_utf8_lossy(&bytes).to_string();

        // 3. Emit event to Frontend (ignore errors if window is closed)
        let _ = self.window.emit(
            "search-update",
            SearchUpdate {
                epoch,
                score,
                layout: layout_str,
                ips,
            },
        );

        true // Continue optimization
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn test_atomic_write_creates_file() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test_file.txt");
        let content = b"Hello, World!";

        atomic_write(&file_path, content).expect("Atomic write failed");

        let mut file = fs::File::open(&file_path).expect("Failed to open file");
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).expect("Failed to read file");

        assert_eq!(buffer, content);
    }

    #[test]
    fn test_atomic_write_overwrites_existing() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("config.json");

        // 1. Write Initial
        atomic_write(&file_path, b"initial").unwrap();

        // 2. Overwrite
        atomic_write(&file_path, b"updated").unwrap();

        let read_back = fs::read_to_string(&file_path).unwrap();
        assert_eq!(read_back, "updated");

        // 3. Ensure temp file is gone
        let temp_path = file_path.with_extension("tmp");
        assert!(!temp_path.exists(), "Temp file was not cleaned up");
    }
}
