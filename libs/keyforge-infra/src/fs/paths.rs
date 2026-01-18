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
use std::env;
use std::path::PathBuf;

/// Attempts to locate the 'data' directory.
pub fn resolve_root(explicit: Option<PathBuf>) -> Result<PathBuf, String> {
    if let Some(p) = explicit {
        if p.exists() {
            return Ok(p);
        }
        return Err(format!("Explicit data path not found: {p:?}"));
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
        // Sanity check: A valid workspace must contain 'keyboards'
        if p.exists() && p.join("keyboards").exists() {
            return std::fs::canonicalize(p)
                .map_err(|e| format!("Failed to canonicalize path: {e}"));
        }
    }

    Err("Could not locate KeyForge 'data' directory.".to_string())
}

/// Resolves the absolute paths for a Job's assets (Cost Matrix and Corpus).
#[must_use] 
pub fn resolve_job_paths(
    root: &std::path::Path,
    corpus_name: &str,
    cost_matrix_name: &str,
) -> Option<(PathBuf, PathBuf)> {
    let cost_path = root.join(cost_matrix_name);

    // CHANGED: Removed special handling for "default".
    // The system now expects explicit paths like "text/en_std".
    let corpus_dir = root.join("corpora").join(corpus_name);

    // We don't check existence here, just path construction.
    // The consumer checks existence to return 404/Error.
    Some((cost_path, corpus_dir))
}
