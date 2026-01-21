// libs/keyforge-infra/src/fs/paths.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use keyforge_model::constants::DATA_DIR_CANDIDATES;
use crate::fs::init::WORKSPACE_MARKER;
use std::env;
use std::path::PathBuf;

/// Resolves the absolute path to the data root.
///
/// # Errors
///
/// Returns an error if the explicit path is provided but does not exist.
pub fn resolve_root(explicit: Option<PathBuf>) -> Result<PathBuf, String> {
    if let Some(p) = explicit {
        if !p.exists() {
            return Err(format!("Explicit data path not found: {}", p.display()));
        }
        return Ok(p);
    }

    if let Ok(env_path) = env::var("KEYFORGE_DATA_DIR") {
        let p = PathBuf::from(env_path);
        if p.exists() {
            return Ok(p);
        }
    }

    let candidates = DATA_DIR_CANDIDATES;

    for c in candidates {
        let p = PathBuf::from(c);
        // Task-infra-rev-003: Check for marker file primarily
        let has_marker = p.join(WORKSPACE_MARKER).exists();
        // Fallback to legacy keyboards check
        let has_keyboards = p.join("keyboards").exists();

        if p.exists() && (has_marker || has_keyboards) {
            return std::fs::canonicalize(p)
                .map_err(|e| format!("Failed to canonicalize path: {e}"));
        }
    }

    Err("Could not locate KeyForge 'data' directory.".to_string())
}

