use std::path::PathBuf;
use tauri::AppHandle;
use std::env;

/// Determines the canonical data directory for the application.
///
/// Priority:
/// 1. `KEYFORGE_DATA_DIR` environment variable.
/// 2. OS-specific user data directory (e.g., `~/.local/share/keyforge` on Linux).
pub fn get_data_dir(_app: &AppHandle) -> Result<PathBuf, String> {
    // Priority 1: Environment Variable (Dev/Sandbox)
    if let Ok(dir) = env::var("KEYFORGE_DATA_DIR") {
        return Ok(PathBuf::from(dir));
    }

    // Priority 2: OS App Data Dir (Production)
    #[cfg(target_os = "linux")]
    {
        if let Ok(home) = env::var("HOME") {
            return Ok(PathBuf::from(home).join(".local/share/keyforge"));
        }
    }

    Err("Could not determine data directory".to_string())
}
