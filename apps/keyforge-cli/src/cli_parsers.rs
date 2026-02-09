// apps/keyforge-cli/src/cli_parsers.rs

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

use crate::constants::MAX_CLI_CORPORA;
use keyforge_model::config::CorpusSource;
use keyforge_model::constants::{MAX_FILENAME_LEN, MAX_KEYBOARD_NAME_LEN};
use keyforge_model::types::path::SafePath;
use keyforge_model::KeyConstraint;
use std::path::{Path, PathBuf};
use std::str::FromStr;

pub fn parse_key_constraint(s: &str) -> Result<KeyConstraint, String> {
    KeyConstraint::from_str(s)
}

/// Checks for a path's existence, and if not found and it has no extension,
/// checks for the same path with a `.json` extension.
fn check_path(path: &Path) -> Option<PathBuf> {
    if path.exists() {
        return Some(path.to_path_buf());
    }
    if path.extension().is_none() {
        let p_json = path.with_extension("json");
        if p_json.exists() {
            return Some(p_json);
        }
    }
    None
}

pub fn resolve_path(input: &str, subdir: Option<&str>, root: &Path) -> Result<SafePath, String> {
    let input_path = Path::new(input);

    // 1. Absolute paths
    if input_path.is_absolute() {
        let p = check_path(input_path)
            .ok_or_else(|| format!("Absolute path does not exist: {input}"))?;
        // For absolute paths, we wrap them in SafePath as untraversable roots
        return Ok(SafePath::from_trusted_root_path(p));
    }

    // 2. Explicit CWD-relative paths
    if input.starts_with("./") || input.starts_with("../") {
        if let Ok(cwd) = std::env::current_dir() {
            if let Some(p) = check_path(&cwd.join(input)) {
                // If the user explicitly used ../, they might be traversing, but if check_path finds it,
                // we treat it as an explicit selection.
                // However, for strictness, we should probably validate it via SafePath if we can.
                // BUT, CWD might not be our workspace root.
                return Ok(SafePath::from_trusted_root_path(p));
            }
        }
    }

    // 3. Workspace Resolution (Overlay: user -> system -> root)
    let sub = subdir.unwrap_or("");

    // Candidates for relative segments
    let segments = if sub.is_empty() {
        vec![
            format!("user/{input}"),
            format!("system/{input}"),
            input.to_string(),
        ]
    } else {
        vec![
            format!("user/{sub}/{input}"),
            format!("system/{sub}/{input}"),
            format!("{sub}/{input}"),
            input.to_string(),
        ]
    };

    for seg in segments {
        if let Ok(rel) = SafePath::try_from_str(&seg) {
            let combined = SafePath::from_trusted_root(root, &rel);
            if let Some(found) = check_path(combined.as_path()) {
                // Re-create from found path to ensure absolute/relative consistency
                return Ok(SafePath::from_trusted_root_path(found));
            }
        }
    }

    Err(format!(
        "Could not resolve path '{input}'. Checked absolute, CWD, and workspace '{subdir:?}' overlays."
    ))
}

pub fn parse_keyboard(s: &str) -> Result<String, String> {
    if s.len() > MAX_KEYBOARD_NAME_LEN {
        return Err(format!(
            "keyboard name must be <= {MAX_KEYBOARD_NAME_LEN} chars"
        ));
    }
    let s = s.trim();
    if s.is_empty() {
        return Err("keyboard name cannot be empty".into());
    }
    Ok(s.into())
}

pub fn parse_cost(s: &str) -> Result<String, String> {
    if s.len() > MAX_FILENAME_LEN {
        return Err(format!(
            "cost matrix filename must be <= {MAX_FILENAME_LEN} chars"
        ));
    }
    let s = s.trim();
    if s.is_empty() {
        return Err("cost matrix filename cannot be empty".into());
    }
    Ok(s.into())
}

pub fn parse_corpora(args: &[String]) -> Result<Vec<CorpusSource>, String> {
    if args.len() > MAX_CLI_CORPORA {
        return Err(format!(
            "Too many corpora sources (limit {MAX_CLI_CORPORA})"
        ));
    }
    args.iter().map(|s| CorpusSource::from_str(s)).collect()
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;

    #[test]
    fn test_parse_keyboard() {
        assert_eq!(parse_keyboard("  corne  ").unwrap(), "corne");
        assert!(parse_keyboard("").is_err());
    }

    #[test]
    fn test_parse_cost() {
        assert_eq!(parse_cost("weights.json").unwrap(), "weights.json");
        assert!(parse_cost("").is_err());
    }
}
