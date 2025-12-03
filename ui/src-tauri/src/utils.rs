use crate::models::SearchUpdate;
use keyforge_core::optimizer::ProgressCallback;
use std::path::PathBuf;
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
        if path.exists() && path.is_dir() {
            // Simple heuristic to ensure it's the right folder
            if path.join("keyboards").exists() {
                return Ok(path);
            }
        }
    }

    // 4. Last Resort: Return AppData path even if it doesn't exist yet.
    // This allows the caller (like the sync command) to create it.
    if let Ok(app_data) = app.path().app_data_dir() {
        let data = app_data.join("data");
        return Ok(data);
    }

    Err("Could not resolve data directory. Please ensure the 'data' folder is bundled or available.".into())
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
