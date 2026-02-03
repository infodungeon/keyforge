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
            let path = entry.path();
            let path_str = path.to_string_lossy();
            
            for ext in extensions {
                if path_str.ends_with(ext) {
                    results.push(entry.into_path());
                    break;
                }
            }
        }
    }
    Ok(results)
}

enum IdStrategy {
    /// ID is the relative path minus extension (e.g. "models/sys").
    RelativePath,
    /// ID is the filename stem, ignoring directory structure (e.g. "sys").
    /// Used if the asset ID is unique by filename regardless of folder.
    FilenameStem,
    /// ID is the parent directory relative to root (e.g. "en/std" for "en/std/1grams.json").
    ParentDir,
}

fn list_assets(root: &Path, category: &str, extensions: &[&str], strategy: IdStrategy) -> InfraResult<Vec<String>> {
    let mut assets = Vec::new();
    let ext_strings: Vec<String> = extensions.iter().map(|s| s.to_string()).collect();

    let process_path = |path: PathBuf, base: &Path| -> Option<String> {
        let rel = path.strip_prefix(base).ok()?;
        match strategy {
            IdStrategy::RelativePath => {
                let rel_str = rel.to_string_lossy();
                let mut id = rel_str.to_string();
                for ext in extensions {
                    if id.ends_with(ext) {
                        id = id.trim_end_matches(ext).trim_end_matches('.').to_string();
                        break;
                    }
                }
                Some(id)
            }
            IdStrategy::FilenameStem => {
                // Extracts "sys" from "models/sys.mpk.zst"
                // Note: file_stem() handles one extension. "sys.mpk.zst" -> "sys.mpk".
                // We need to handle multiple extensions.
                let filename = path.file_name()?.to_string_lossy();
                let mut id = filename.to_string();
                for ext in extensions {
                    if id.ends_with(ext) {
                        id = id.trim_end_matches(ext).trim_end_matches('.').to_string();
                        break;
                    }
                }
                Some(id)
            }
            IdStrategy::ParentDir => {
                let parent = rel.parent()?;
                Some(parent.to_string_lossy().to_string())
            }
        }
    };

    // 1. System Assets
    let sys_path = root.join("system").join(category);
    let sys_files = list_files(&sys_path, &ext_strings)?;
    for path in sys_files {
        if let Some(id) = process_path(path, &sys_path) {
            assets.push(id);
        }
    }

    // 2. User Assets
    let user_path = root.join("user").join(category);
    let user_files = list_files(&user_path, &ext_strings)?;
    for path in user_files {
        if let Some(id) = process_path(path, &user_path) {
            assets.push(id);
        }
    }

    Ok(assets)
}

/// Lists all available corpora.
pub fn list_corpora(root: &Path) -> InfraResult<Vec<String>> {
    // Corpora are defined by the folder containing 1grams/etc.
    list_assets(root, "corpora", &["1grams.json", "1grams.mpk.zst"], IdStrategy::ParentDir)
}

/// Lists all available cost matrices.
pub fn list_cost_matrices(root: &Path) -> InfraResult<Vec<String>> {
    list_assets(root, "weights", &["json", "mpk", "mpk.zst"], IdStrategy::FilenameStem)
}

/// Lists all available keyboard models.
pub fn list_keyboards(root: &Path) -> InfraResult<Vec<String>> {
    // Test expects "sys" from "models/sys.mpk.zst", so FilenameStem.
    list_assets(root, "keyboards", &["json", "mpk", "mpk.zst"], IdStrategy::FilenameStem)
}

/// Lists all available keymap extra configurations.
pub fn list_keymap_extras(root: &Path) -> InfraResult<Vec<String>> {
    list_assets(root, "keymap_extras", &["json", "mpk", "mpk.zst"], IdStrategy::FilenameStem)
}
