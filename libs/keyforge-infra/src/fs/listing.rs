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

/// Helper to scan a specific sub-path and add stems to a set.
/// Extension is the target suffix (e.g., "mpk.zst" or "json").
fn scan_dir(
    root: &Path,
    sub_path: &str,
    extension: &str,
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

        if filename.ends_with(extension) {
            let stem = filename
                .strip_suffix(&format!(".{extension}"))
                .unwrap_or(filename);

            let final_stem = stem.strip_suffix(".mpk").unwrap_or(stem);
            results.insert(final_stem.to_string());
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
        "mpk.zst",
        &mut names,
    )?;
    scan_dir(root, "user/keyboards", "json", &mut names)?;

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
        "mpk.zst",
        &mut names,
    )?;
    scan_dir(root, "user/weights", "json", &mut names)?;

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
        "mpk.zst",
        &mut names,
    )?;
    scan_dir(root, "user/keymap_extras", "json", &mut names)?;
    let mut sorted: Vec<String> = names.into_iter().collect();
    sorted.sort();
    Ok(sorted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_listing_filters() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        
        // 1. Keyboards
        let sys_kb = root.join("system/keyboards/models");
        let user_kb = root.join("user/keyboards");
        fs::create_dir_all(&sys_kb).unwrap();
        fs::create_dir_all(&user_kb).unwrap();
        fs::write(sys_kb.join("sys.mpk.zst"), "").unwrap();
        fs::write(user_kb.join("user.json"), "").unwrap();
        
        let list = list_keyboards(root).unwrap();
        assert_eq!(list.len(), 2);
        assert!(list.contains(&"sys".into()));
        assert!(list.contains(&"user".into()));

        // 2. Corpora
        let sys_corp = root.join("system/corpora/en/std");
        let user_corp = root.join("user/corpora/custom");
        fs::create_dir_all(&sys_corp).unwrap();
        fs::create_dir_all(&user_corp).unwrap();
        fs::write(sys_corp.join("1grams.mpk.zst"), "").unwrap();
        fs::write(user_corp.join("1grams.json"), "").unwrap();
        
        let list = list_corpora(root).unwrap();
        assert_eq!(list.len(), 2);
        assert!(list.contains(&"en/std".into()));
        assert!(list.contains(&"custom".into()));

        // 3. Cost Matrices
        let sys_cm = root.join("system/weights");
        let user_cm = root.join("user/weights");
        fs::create_dir_all(&sys_cm).unwrap();
        fs::create_dir_all(&user_cm).unwrap();
        fs::write(sys_cm.join("cm_sys.mpk.zst"), "").unwrap();
        fs::write(user_cm.join("cm_user.json"), "").unwrap();
        
        let list = list_cost_matrices(root).unwrap();
        assert!(list.contains(&"cm_sys".into()));
        assert!(list.contains(&"cm_user".into()));

        // 4. Keymap Extras
        let sys_extra = root.join("system/keymap_extras");
        let user_extra = root.join("user/keymap_extras");
        fs::create_dir_all(&sys_extra).unwrap();
        fs::create_dir_all(&user_extra).unwrap();
        fs::write(sys_extra.join("extra_sys.mpk.zst"), "").unwrap();
        fs::write(user_extra.join("extra_user.json"), "").unwrap();
        
        let list = list_keymap_extras(root).unwrap();
        assert!(list.contains(&"extra_sys".into()));
        assert!(list.contains(&"extra_user".into()));
    }

    #[test]
    fn test_scan_dir_edge_cases() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        let dir = root.join("test");
        fs::create_dir_all(&dir).unwrap();
        
        // Non-file entries
        fs::create_dir(dir.join("subdir")).unwrap();
        
        let mut results = HashSet::new();
        scan_dir(root, "test", "json", &mut results).unwrap();
        assert!(results.is_empty());

        // File without extension
        fs::write(dir.join("noext"), "").unwrap();
        scan_dir(root, "test", "json", &mut results).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_listing_empty_root() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        
        assert!(list_keyboards(root).unwrap().is_empty());
        assert!(list_corpora(root).unwrap().is_empty());
        assert!(list_cost_matrices(root).unwrap().is_empty());
        assert!(list_keymap_extras(root).unwrap().is_empty());
    }
}
