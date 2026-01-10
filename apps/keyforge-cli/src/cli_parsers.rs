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


use keyforge_model::config::CorpusSource;
use keyforge_model::KeyConstraint;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use keyforge_model::constants::{MAX_KEYBOARD_NAME_LEN, MAX_FILENAME_LEN};
use crate::constants::MAX_CLI_CORPORA;

pub fn parse_key_constraint(s: &str) -> Result<KeyConstraint, String> {
    KeyConstraint::from_str(s)
}

pub fn resolve_path(input: &str, subdir: Option<&str>, root: &Path) -> Result<PathBuf, String> {
    let input_path = Path::new(input);

    // 1. Absolute paths: Always allowed for CLI users.
    if input_path.is_absolute() {
        if input_path.exists() {
            return Ok(input_path.to_path_buf());
        } else {
            return Err(format!("Absolute path does not exist: {}", input));
        }
    }

    // 2. Explicit CWD-relative paths (./ or ../)
    if input.starts_with("./") || input.starts_with("../") {
        if let Ok(cwd) = std::env::current_dir() {
            let p = cwd.join(input);
            if p.exists() {
                return Ok(p);
            }
        }
    }

    // 3. Workspace Resolution (Overlay: user -> system -> root)
    let sub = subdir.unwrap_or("");

    // Check user/{subdir}/{input}
    let user_path = root.join("user").join(sub).join(input);
    if user_path.exists() {
        return Ok(user_path);
    }
    // Try with .json
    if !input.ends_with(".json") {
        let user_json = user_path.with_extension("json");
        if user_json.exists() {
            return Ok(user_json);
        }
    }

    // Check system/{subdir}/{input}
    let system_path = root.join("system").join(sub).join(input);
    if system_path.exists() {
        return Ok(system_path);
    }
    // Try with .json
    if !input.ends_with(".json") {
        let system_json = system_path.with_extension("json");
        if system_json.exists() {
            return Ok(system_json);
        }
    }

    // Check root/{subdir}/{input} (Legacy/Fallback)
    let root_sub_path = root.join(sub).join(input);
    if root_sub_path.exists() {
        return Ok(root_sub_path);
    }
    if !input.ends_with(".json") {
        let root_json = root_sub_path.with_extension("json");
        if root_json.exists() {
            return Ok(root_json);
        }
    }

    // 4. Root-relative fallback (Directly in data root)
    let root_direct = root.join(input);
    if root_direct.exists() {
        return Ok(root_direct);
    }
    if !input.ends_with(".json") {
        let root_direct_json = root_direct.with_extension("json");
        if root_direct_json.exists() {
            return Ok(root_direct_json);
        }
    }

    Err(format!(
        "Could not resolve path '{}'. Checked absolute, CWD, and workspace '{:?}' overlays.",
        input, subdir
    ))
}

pub fn parse_keyboard(s: &str) -> Result<String, String> {
    if s.len() > MAX_KEYBOARD_NAME_LEN {
        return Err(format!("keyboard name must be <= {} chars", MAX_KEYBOARD_NAME_LEN));
    }
    let s = s.trim();
    if s.is_empty() {
        return Err("keyboard name cannot be empty".into());
    }
    Ok(s.into())
}

pub fn parse_cost(s: &str) -> Result<String, String> {
    if s.len() > MAX_FILENAME_LEN {
        return Err(format!("cost matrix filename must be <= {} chars", MAX_FILENAME_LEN));
    }
    let s = s.trim();
    if s.is_empty() {
        return Err("cost matrix filename cannot be empty".into());
    }
    Ok(s.into())
}

pub fn parse_corpora(args: &[String]) -> Result<Vec<CorpusSource>, String> {
    if args.len() > MAX_CLI_CORPORA {
        return Err(format!("Too many corpora sources (limit {})", MAX_CLI_CORPORA));
    }
    args.iter().map(|s| CorpusSource::from_str(s)).collect()
}
