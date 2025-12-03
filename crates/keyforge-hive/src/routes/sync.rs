use axum::Json;
use keyforge_core::util::calculate_file_hash;
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;
use tracing::info;
use walkdir::WalkDir;

#[derive(Serialize)]
pub struct FileManifest {
    pub files: HashMap<String, String>,
}

pub async fn get_manifest() -> Json<FileManifest> {
    let mut files = HashMap::new();
    // Look for data in the workspace root
    let possible_paths = vec!["data", "../data", "../../data"];

    let mut data_root = "data";
    for p in &possible_paths {
        if Path::new(p).exists() {
            data_root = p;
            break;
        }
    }

    info!("📂 Scannning for manifest at: '{}'", data_root);

    // Security: Disable following links to prevent symlink attacks escaping data dir
    let walker = WalkDir::new(data_root).follow_links(false);

    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let path = entry.path();

            // Extra safety check: Path must start with data_root
            if !path.starts_with(data_root) {
                continue;
            }

            if let Ok(hash) = calculate_file_hash(path) {
                // Normalize path to be relative to data_root for the client
                let full_path_str = path.to_string_lossy().replace("\\", "/");

                // Strip the data_root prefix
                let clean_key = if let Some(idx) = full_path_str.find("/data/") {
                    full_path_str[idx + 6..].to_string()
                } else if let Some(stripped) = full_path_str.strip_prefix("data/") {
                    stripped.to_string()
                } else {
                    full_path_str.to_string()
                };

                files.insert(clean_key, hash);
            }
        }
    }

    info!("📊 Generated manifest with {} files.", files.len());
    Json(FileManifest { files })
}
