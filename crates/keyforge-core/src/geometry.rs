// Re-export data types so they appear to be part of this module for internal core use
pub use crate::protocol::geometry::*;

use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub mod kle;

// Trait to extend the Protocol struct with IO logic
pub trait KeyboardLoader {
    fn load_from_file<P: AsRef<Path>>(path: P) -> Result<KeyboardDefinition, String>;
}

impl KeyboardLoader for KeyboardDefinition {
    fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("❌ Failed to read keyboard file: {}", e))?;

        // 1. Try standard KeyForge JSON format
        if let Ok(mut def) = serde_json::from_str::<KeyboardDefinition>(&content) {
            def.geometry.calculate_origins();
            return Ok(def);
        }

        // 2. Try KLE Format
        if let Ok(geom) = kle::parse_kle_json(&content) {
            let name = path
                .as_ref()
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .to_string();

            return Ok(KeyboardDefinition {
                meta: KeyboardMeta {
                    name,
                    author: "Imported from KLE".to_string(),
                    kb_type: "imported".to_string(),
                    ..Default::default()
                },
                geometry: geom,
                layouts: HashMap::new(),
            });
        }

        Err("❌ Failed to parse keyboard JSON".to_string())
    }
}
