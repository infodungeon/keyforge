use crate::error::{InfraError, InfraResult};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Helper to scan a specific sub-path and add stems to a set.
/// Extension is the target suffix (e.g., "mpk.zst").
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

        let filename = match p.file_name().and_then(|s| s.to_str()) {
            Some(f) => f,
            None => continue,
        };

        if filename.ends_with(extension) {
            let stem = filename
                .strip_suffix(&format!(".{}", extension))
                .unwrap_or(filename);
            // Handle compound stems like "corne.mpk"
            let final_stem = stem.strip_suffix(".mpk").unwrap_or(stem);
            results.insert(final_stem.to_string());
        }
    }
    Ok(())
}

pub fn list_keyboards(root: &Path) -> InfraResult<Vec<String>> {
    let mut names = HashSet::new();
    // System: Binary Only
    scan_dir(root, "system/keyboards", "mpk.zst", &mut names)?;
    // User: Support JSON for development/customization
    scan_dir(root, "user/keyboards", "json", &mut names)?;

    let mut sorted: Vec<String> = names.into_iter().collect();
    sorted.sort();
    Ok(sorted)
}

pub fn list_corpora(root: &Path) -> InfraResult<Vec<String>> {
    let mut ids = HashSet::new();

    for (scope, ext) in [("system", "mpk.zst"), ("user", "json")] {
        let base = root.join(scope).join("corpora");
        if !base.exists() {
            continue;
        }

        let walker = walkdir::WalkDir::new(&base)
            .min_depth(1)
            .max_depth(3) // Extra depth for category/id
            .follow_links(false);

        for entry in walker.into_iter().filter_map(|e| e.ok()) {
            if entry.path_is_symlink() {
                continue;
            }
            let p = entry.path();

            // Check for the anchor file (1grams) in either binary or json format
            if p.file_name().and_then(|s| s.to_str()) == Some(&format!("1grams.{}", ext)) {
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

pub fn list_cost_matrices(root: &Path) -> InfraResult<Vec<String>> {
    let mut names = HashSet::new();
    scan_dir(root, "system/weights", "mpk.zst", &mut names)?;
    scan_dir(root, "user/weights", "json", &mut names)?;

    for excluded in ["ortho_split", "row_stagger", "testing"] {
        names.remove(excluded);
    }

    let mut sorted: Vec<String> = names.into_iter().collect();
    sorted.sort();
    Ok(sorted)
}

pub fn list_keymap_extras(root: &Path) -> InfraResult<Vec<String>> {
    let mut names = HashSet::new();
    scan_dir(root, "system/keymap_extras", "mpk.zst", &mut names)?;
    scan_dir(root, "user/keymap_extras", "json", &mut names)?;
    let mut sorted: Vec<String> = names.into_iter().collect();
    sorted.sort();
    Ok(sorted)
}
