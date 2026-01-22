// libs/keyforge-infra/src/fs/listing.rs

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

use crate::asset::{
    ASSET_PATH_CORPORA, ASSET_PATH_KEYBOARDS, ASSET_PATH_KEYMAP_EXTRAS, ASSET_PATH_WEIGHTS,
};
use crate::error::{InfraError, InfraResult};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Registry of supported asset file extensions.
#[derive(Debug)]
pub struct ExtensionRegistry;

impl ExtensionRegistry {
    /// Standard extensions for compressed binary assets.
    pub const BINARY: &'static [&'static str] = &["mpk.zst", "zst", "mpk"];
    /// Standard extensions for human-readable text assets.
    pub const TEXT: &'static [&'static str] = &["json", "toml", "yaml"];

    /// Returns true if the path has a supported binary extension.
    #[must_use]
    pub fn is_binary(path: &Path) -> bool {
        Self::BINARY
            .iter()
            .any(|&ext| path.to_string_lossy().ends_with(ext))
    }

    /// Returns true if the path has a supported text extension.
    #[must_use]
    pub fn is_text(path: &Path) -> bool {
        Self::TEXT
            .iter()
            .any(|&ext| path.to_string_lossy().ends_with(ext))
    }
}

/// Helper to scan a specific sub-path and add stems to a set.
fn scan_dir(
    root: &Path,
    sub_path: &str,
    target_extensions: &[&str],
    results: &mut HashSet<String>,
) -> InfraResult<()> {
    let target = root.join(sub_path);
    if !target.exists() {
        return Ok(());
    }

    let entries = fs::read_dir(&target).map_err(InfraError::Io)?;
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_symlink() || !p.is_file() {
            continue;
        }

        let Some(filename) = p.file_name().and_then(|s| s.to_str()) else {
            continue;
        };

        for &ext in target_extensions {
            if filename.ends_with(ext) {
                let stem = filename
                    .strip_suffix(&format!(".{ext}"))
                    .unwrap_or(filename);

                // Further strip internal sub-extensions
                let final_stem = stem.strip_suffix(".mpk").unwrap_or(stem);
                results.insert(final_stem.to_string());
                break;
            }
        }
    }
    Ok(())
}

/// Discovers all available keyboard definitions in both the system library and user workspace.
///
/// # Errors
/// Returns `InfraError` if directory scanning fails.
pub fn list_keyboards(root: &Path) -> InfraResult<Vec<String>> {
    let mut names = HashSet::new();
    scan_dir(
        root,
        &format!("system/{ASSET_PATH_KEYBOARDS}"),
        ExtensionRegistry::BINARY,
        &mut names,
    )?;
    scan_dir(root, "user/keyboards", ExtensionRegistry::TEXT, &mut names)?;

    let mut sorted: Vec<String> = names.into_iter().collect();
    sorted.sort();
    Ok(sorted)
}

/// Discovers all available corpora by searching for directory-based bundles.
///
/// # Errors
/// Returns `InfraError` if directory scanning fails.
pub fn list_corpora(root: &Path) -> InfraResult<Vec<String>> {
    let mut ids = HashSet::new();

    for (scope, ext) in [("system", "mpk.zst"), ("user", "json")] {
        let base = root.join(scope).join(ASSET_PATH_CORPORA);
        if !base.exists() {
            continue;
        }

        let walker = walkdir::WalkDir::new(&base)
            .min_depth(1)
            .max_depth(3)
            .follow_links(true);

        for entry in walker.into_iter().filter_map(std::result::Result::ok) {
            if entry.path_is_symlink() {
                continue;
            }
            let p = entry.path();

            if p.file_name().and_then(|s| s.to_str()) == Some(&format!("1grams.{ext}")) {
                if let Some(parent) = p.parent() {
                    if let Ok(relative) = parent.strip_prefix(&base) {
                        let id = relative.to_string_lossy().replace('\\', "/");
                        if !id.is_empty() {
                            ids.insert(id);
                        }
                    }
                }
            }
        }
    }

    let mut sorted: Vec<String> = ids.into_iter().collect();
    sorted.sort();
    Ok(sorted)
}

/// Lists all available cost matrices (effort models).
///
/// # Errors
/// Returns `InfraError` if directory scanning fails.
pub fn list_cost_matrices(root: &Path) -> InfraResult<Vec<String>> {
    let mut names = HashSet::new();
    scan_dir(
        root,
        &format!("system/{ASSET_PATH_WEIGHTS}"),
        ExtensionRegistry::BINARY,
        &mut names,
    )?;
    scan_dir(root, "user/weights", ExtensionRegistry::TEXT, &mut names)?;

    let mut sorted: Vec<String> = names.into_iter().collect();
    sorted.sort();
    Ok(sorted)
}

/// Lists available keymap extras (e.g., custom symbols or macros).
///
/// # Errors
/// Returns `InfraError` if directory scanning fails.
pub fn list_keymap_extras(root: &Path) -> InfraResult<Vec<String>> {
    let mut names = HashSet::new();
    scan_dir(
        root,
        &format!("system/{ASSET_PATH_KEYMAP_EXTRAS}"),
        ExtensionRegistry::BINARY,
        &mut names,
    )?;
    scan_dir(
        root,
        "user/keymap_extras",
        ExtensionRegistry::TEXT,
        &mut names,
    )?;
    let mut sorted: Vec<String> = names.into_iter().collect();
    sorted.sort();
    Ok(sorted)
}
