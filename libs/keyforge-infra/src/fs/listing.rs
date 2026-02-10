// libs/keyforge-infra/src/fs/listing.rs

use crate::error::{InfraError, InfraResult};
use keyforge_boundary::SafePath;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

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

<<<<<<< HEAD
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
=======
/// Helper to scan a specific sub-path and add stems to a set.
fn scan_dir(
    root: &SafePath,
    sub_path: &str,
    target_extensions: &[&str],
    results: &mut HashSet<String>,
) -> InfraResult<()> {
    let target = root.as_path().join(sub_path);
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
pub fn list_keyboards(root: &SafePath) -> InfraResult<Vec<String>> {
    let mut names = HashSet::new();
    scan_dir(
        root,
        "system/keyboards/models",
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
pub fn list_corpora(root: &SafePath) -> InfraResult<Vec<String>> {
    let mut ids = HashSet::new();

    for (scope, ext) in [("system", "mpk.zst"), ("user", "json")] {
        let base = root.as_path().join(scope).join("corpora");
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
pub fn list_cost_matrices(root: &SafePath) -> InfraResult<Vec<String>> {
    let mut names = HashSet::new();
    scan_dir(
        root,
        "system/weights",
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
pub fn list_keymap_extras(root: &SafePath) -> InfraResult<Vec<String>> {
    let mut names = HashSet::new();
    scan_dir(
        root,
        "system/keymap_extras",
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
>>>>>>> master
}
