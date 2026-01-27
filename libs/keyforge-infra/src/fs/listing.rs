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
            if let Some(ext) = entry.path().extension() {
                let ext_str = ext.to_string_lossy().to_string();
                if extensions.contains(&ext_str) {
                    results.push(entry.into_path());
                }
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
    let dir = root.join("system/corpora");
    list_files(&dir, &["json".to_string(), "txt".to_string()])
}

/// Lists all available cost matrices.
///
/// # Errors
///
/// Returns `InfraError` if the weights directory is unreachable.
pub fn list_cost_matrices(root: &Path) -> InfraResult<Vec<PathBuf>> {
    let dir = root.join("system/weights");
    list_files(&dir, &["json".to_string()])
}

/// Lists all available keyboard models.
///
/// # Errors
///
/// Returns `InfraError` if the keyboards directory is unreachable.
pub fn list_keyboards(root: &Path) -> InfraResult<Vec<PathBuf>> {
    let dir = root.join("system/keyboards");
    list_files(&dir, &["json".to_string()])
}

/// Lists all available keymap extra configurations.
///
/// # Errors
///
/// Returns `InfraError` if the extras directory is unreachable.
pub fn list_keymap_extras(root: &Path) -> InfraResult<Vec<PathBuf>> {
    let dir = root.join("system/keymap_extras");
    list_files(&dir, &["json".to_string()])
}
