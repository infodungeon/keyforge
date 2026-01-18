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

use crate::asset::{ASSET_PATH_CORPORA, ASSET_PATH_KEYBOARDS, ASSET_PATH_KEYMAP_EXTRAS, ASSET_PATH_WEIGHTS};
use crate::error::{InfraError, InfraResult};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Helper to scan a specific sub-path and add stems to a set.
/// Extension is the target suffix (e.g., "mpk.zst" or "json").
fn scan_dir(
    root: &Path,
    sub_path: &str,
    extension: &str,
    results: &mut HashSet<String>,
) -> InfraResult<()> {
    let target = root.join(sub_path);
    // eprintln!("DEBUG: Scanning {:?} for extension '{}'", target, extension);
    
    if !target.exists() {
        return Ok(());
    }

    let entries = fs::read_dir(&target).map_err(InfraError::Io)?;
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_symlink() || !p.is_file() {
            continue;
        }

        let filename = match p.file_name().and_then(|s| s.to_str()) {
            Some(f) => f,
            None => continue,
        };

        if filename.ends_with(extension) {
            let stem = filename
                .strip_suffix(&format!(".{extension}"))
                .unwrap_or(filename);
            
            // NOTE: Previous logic arbitrarily stripped ".mpk" here. 
            // We now rely on consistent naming. If "foo.mpk.zst" is scanned with ext "zst",
            // stem is "foo.mpk". If scanned with "mpk.zst", stem is "foo".
            // To maintain compatibility with "mpk" files being the Stem Identity, if the result ends in .mpk, we strip it.
            let final_stem = stem.strip_suffix(".mpk").unwrap_or(stem);
            results.insert(final_stem.to_string());
        }
    }
    Ok(())
}

/// Discovers all available keyboard definitions in both the system library and user workspace.
///
/// Returns a sorted list of unique keyboard identifiers (file stems).
pub fn list_keyboards(root: &Path) -> InfraResult<Vec<String>> {
    let mut names = HashSet::new();
    // System: Binary Only - Updated to new structure
    scan_dir(root, &format!("system/{ASSET_PATH_KEYBOARDS}"), "mpk.zst", &mut names)?;
    // User: Support JSON for development/customization
    scan_dir(root, "user/keyboards", "json", &mut names)?;

    let mut sorted: Vec<String> = names.into_iter().collect();
    sorted.sort();
    Ok(sorted)
}

/// Discovers all available corpora by searching for directory-based bundles.
///
/// A corpus is considered present if it contains a `1grams` anchor file.
/// Returns a sorted list of unique corpus identifiers.
pub fn list_corpora(root: &Path) -> InfraResult<Vec<String>> {
    let mut ids = HashSet::new();

    for (scope, ext) in [("system", "mpk.zst"), ("user", "json")] {
        let base = root.join(scope).join(ASSET_PATH_CORPORA);
        if !base.exists() {
            continue;
        }

        let walker = walkdir::WalkDir::new(&base)
            .min_depth(1)
            .max_depth(3) // Extra depth for category/id
            .follow_links(true); // Follow symlinks

        for entry in walker.into_iter().filter_map(std::result::Result::ok) {
            if entry.path_is_symlink() {
                continue;
            }
            let p = entry.path();

            // Check for the anchor file (1grams) in either binary or json format
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
pub fn list_cost_matrices(root: &Path) -> InfraResult<Vec<String>> {
    let mut names = HashSet::new();
    // System: Weights are now in system/weights
    scan_dir(root, &format!("system/{ASSET_PATH_WEIGHTS}"), "mpk.zst", &mut names)?;
    scan_dir(root, "user/weights", "json", &mut names)?;

    let mut sorted: Vec<String> = names.into_iter().collect();
    sorted.sort();
    Ok(sorted)
}


/// Lists available keymap extras (e.g., custom symbols or macros).
pub fn list_keymap_extras(root: &Path) -> InfraResult<Vec<String>> {
    let mut names = HashSet::new();
    scan_dir(root, &format!("system/{ASSET_PATH_KEYMAP_EXTRAS}"), "mpk.zst", &mut names)?;
    scan_dir(root, "user/keymap_extras", "json", &mut names)?;
    let mut sorted: Vec<String> = names.into_iter().collect();
    sorted.sort();
    Ok(sorted)
}
