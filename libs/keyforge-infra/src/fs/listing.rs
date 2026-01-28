// libs/keyforge-infra/src/fs/listing.rs

use crate::error::InfraResult;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Lists files in a directory filtering by extension.
///
/// # Errors
///
/// Returns `InfraError` if the directory cannot be read.
pub fn list_files(dir: &Path, extensions: &[String]) -> InfraResult<Vec<PathBuf>> {
    let mut results = Vec::new();
    if !dir.exists() {
        return Ok(results);
    }

    for entry in WalkDir::new(dir).into_iter().filter_map(Result::ok) {
        if entry.file_type().is_file() {
            let filename = entry.file_name().to_string_lossy();
            if extensions.iter().any(|ext| filename.ends_with(ext)) {
                results.push(entry.into_path());
            }
        }
    }
    Ok(results)
}

/// Lists all available corpora in the system root.
///
/// # Errors
///
/// Returns `InfraError` if the corpora directory is unreachable.
pub fn list_corpora(root: &Path) -> InfraResult<Vec<PathBuf>> {
    let mut results = Vec::new();
    let exts = vec!["json".to_string(), "mpk.zst".to_string()];
    results.extend(list_files(&root.join("system/corpora"), &exts)?);
    results.extend(list_files(&root.join("user/corpora"), &exts)?);
    Ok(results)
}

/// Lists all available cost matrices.
///
/// # Errors
///
/// Returns `InfraError` if the weights directory is unreachable.
pub fn list_cost_matrices(root: &Path) -> InfraResult<Vec<PathBuf>> {
    let mut results = Vec::new();
    let exts = vec!["json".to_string(), "mpk.zst".to_string()];
    results.extend(list_files(&root.join("system/weights"), &exts)?);
    results.extend(list_files(&root.join("user/weights"), &exts)?);
    Ok(results)
}

/// Lists all available keyboard models.
///
/// # Errors
///
/// Returns `InfraError` if the keyboards directory is unreachable.
pub fn list_keyboards(root: &Path) -> InfraResult<Vec<PathBuf>> {
    let mut results = Vec::new();
    let exts = vec!["json".to_string(), "mpk.zst".to_string()];
    results.extend(list_files(&root.join("system/keyboards"), &exts)?);
    results.extend(list_files(&root.join("user/keyboards"), &exts)?);
    Ok(results)
}

/// Lists all available keymap extra configurations.
///
/// # Errors
///
/// Returns `InfraError` if the extras directory is unreachable.
pub fn list_keymap_extras(root: &Path) -> InfraResult<Vec<PathBuf>> {
    let mut results = Vec::new();
    let exts = vec!["json".to_string(), "mpk.zst".to_string()];
    results.extend(list_files(&root.join("system/keymap_extras"), &exts)?);
    results.extend(list_files(&root.join("user/keymap_extras"), &exts)?);
    Ok(results)
}

/// Lists all available layout catalogs.
///
/// # Errors
///
/// Returns `InfraError` if the layouts directory is unreachable.
pub fn list_layouts(root: &Path) -> InfraResult<Vec<PathBuf>> {
    let mut results = Vec::new();
    let exts = vec!["json".to_string(), "mpk.zst".to_string()];
    results.extend(list_files(&root.join("system/layouts"), &exts)?);
    results.extend(list_files(&root.join("user/layouts"), &exts)?);
    Ok(results)
}
