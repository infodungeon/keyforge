use std::path::PathBuf;
use tauri::AppHandle;
use std::env;

/// Determines the canonical data directory for the application.
///
/// Priority:
/// 1. `KEYFORGE_DATA_DIR` environment variable.
/// 2. XDG/System data directory (e.g., `~/.local/share/keyforge` on Linux, `AppData` on Windows).
pub fn get_data_dir(_app: &AppHandle) -> Result<PathBuf, String> {
    // Priority 1: Environment Variable (Dev/Sandbox)
    if let Ok(dir) = env::var("KEYFORGE_DATA_DIR") {
        return Ok(PathBuf::from(dir));
    }

    // Priority 2: OS App Data Dir (Production)
    if let Some(mut dir) = dirs::data_dir() {
        dir.push("keyforge");
        return Ok(dir);
    }

    Err("Could not determine data directory".to_string())
}
