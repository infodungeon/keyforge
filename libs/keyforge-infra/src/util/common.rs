// Copyright (c) 2025 KeyForge Contributors
//
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
use crate::error::InfraResult;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn calculate_file_hash<P: AsRef<Path>>(path: P) -> InfraResult<String> {
    let mut file = File::open(path).map_err(crate::error::InfraError::Io)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 4096];

    loop {
        let n = file
            .read(&mut buffer)
            .map_err(crate::error::InfraError::Io)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    Ok(hex::encode(hasher.finalize()))
}

use keyforge_core::loader::RawCostData;
use keyforge_protocol::UserStatsStore;

pub fn generate_cost_profile(_store: &UserStatsStore) -> String {
    let data = RawCostData { entries: vec![] };
    serde_json::to_string(&data).unwrap_or_else(|_| "{\"entries\":[]}".to_string())
}

use crate::error::InfraError;
use keyforge_model::constants::MAX_INPUT_FILE_SIZE;
use keyforge_model::keycodes::{KeycodeDefinition, KeycodeRegistry};

pub fn load_keycode_registry(path: &Path) -> InfraResult<KeycodeRegistry> {
    let content = crate::fs::io::read_to_string_limited(path, MAX_INPUT_FILE_SIZE)?;

    let mut deserializer = serde_json::Deserializer::from_str(&content);

    use serde::Deserialize;
    let defs: Vec<KeycodeDefinition> =
        Vec::deserialize(&mut deserializer).map_err(InfraError::Serde)?;
    Ok(KeycodeRegistry::new(defs))
}

/// Aggressively sanitizes filenames to prevent traversal or shell issues.
/// Allowlist: Alphanumeric, dot, underscore, hyphen.
/// Replaces everything else with underscore.
pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}
