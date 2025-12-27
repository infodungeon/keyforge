use crate::models::SearchUpdate;
use keyforge_core::ProgressCallback;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager, Window};
use tempfile::NamedTempFile;

/// Resolves the absolute path to the 'data' directory.
pub fn get_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    // 0. Environment Variable (Sandbox/Override)
    // This allows 'just ui' to inject the sandbox path dynamically.
    if let Ok(env_path) = std::env::var("KEYFORGE_DATA_DIR") {
        let p = PathBuf::from(env_path);
        println!("🔍 [Tauri] Checking Env Var KEYFORGE_DATA_DIR: {:?}", p);
        if p.exists() {
            println!("✅ [Tauri] Resolved Data Dir: {:?}", p);
            return Ok(p);
        }
    }

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

    // 3. Development Fallback
    let dev_paths = ["data", "../data", "../../data"];
    for p in dev_paths {
        let path = PathBuf::from(p);
        if path.exists() && path.is_dir() && path.join("keyboards").exists() {
            return Ok(path);
        }
    }

    // 4. Last Resort
    if let Ok(app_data) = app.path().app_data_dir() {
        let data = app_data.join("data");
        return Ok(data);
    }

    Err("Could not resolve data directory.".into())
}

/// Safely writes content to a file using atomic rename strategy.
pub fn atomic_write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> std::io::Result<()> {
    let path = path.as_ref();
    let dir = path.parent().ok_or(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "Invalid path",
    ))?;

    let mut temp_file = NamedTempFile::new_in(dir)?;
    temp_file.write_all(contents.as_ref())?;
    temp_file.persist(path).map_err(|e| e.error)?;

    Ok(())
}

/// Bridge for Core Optimizer -> UI Event
pub struct TauriBridge {
    pub window: Window,
    pub stop_signal: Arc<Mutex<bool>>,
}

impl ProgressCallback for TauriBridge {
    fn on_progress(&self, epoch: usize, score: f32, best_layout: &[u16], ips: f32) -> bool {
        if let Ok(guard) = self.stop_signal.lock() {
            if *guard {
                return false;
            }
        }

        let bytes: Vec<u8> = best_layout
            .iter()
            .map(|&c| if c < 255 { c as u8 } else { b'?' })
            .collect();
        let layout_str = String::from_utf8_lossy(&bytes).to_string();

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
