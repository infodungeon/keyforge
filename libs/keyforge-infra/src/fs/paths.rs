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

use crate::config::CommonConfig;
use std::path::PathBuf;

/// Resolves the absolute path to the `KeyForge` workspace root.
///
/// # Errors
/// Returns an `InfraError` if the root cannot be resolved or is invalid.
pub fn resolve_root(override_path: Option<PathBuf>) -> crate::error::InfraResult<PathBuf> {
    let config = CommonConfig {
        data_dir: override_path,
        ..CommonConfig::default()
    };

    let root = config.resolve_data_dir();

    // If it's a relative path (like "." or custom), we want to canonicalize it if it exists.
    if root.exists() {
        return Ok(root.canonicalize()?);
    }

    Ok(root)
}
