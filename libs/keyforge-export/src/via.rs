// Copyright (c) 2025 KeyForge Contributors
//
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

use crate::error::{ExportError, ExportResult};
use crate::Exporter;
use keyforge_model::constants::{DEFAULT_NO_OP, DEFAULT_TRANSPARENT};
use serde_json::json;

/// An exporter for the VIA keyboard configurator.
///
/// This generates a JSON file that can be imported into the VIA desktop app
/// or web interface to update a compatible keyboard.
#[derive(Debug)]
pub struct ViaExporter;

impl Exporter for ViaExporter {
    fn generate(
        &self,
        layout_name: &str,
        layers: &[Vec<String>],
        _registry: Option<&keyforge_model::keycodes::KeycodeRegistry>,
    ) -> ExportResult<String> {
        if layers.is_empty() {
            return Err(ExportError::InvalidLayout("Layers cannot be empty".into()));
        }

        let mut all_mapped_layers = Vec::new();

        for keys in layers {
            // VIA "Import Keymap" expects a JSON object with a "layers" array.
            let mapped_keys: Vec<String> = keys
                .iter()
                .map(|k| {
                    let upper = k.to_uppercase();
                    match upper.as_str() {
                        "TRNS" | DEFAULT_TRANSPARENT => "KC_TRNS".to_string(),
                        "NO" | DEFAULT_NO_OP => "KC_NO".to_string(),
                        _ => {
                            if !upper.starts_with("KC_")
                                && !upper.contains('(')
                                && upper.chars().all(|c| c.is_alphanumeric() || c == '_')
                            {
                                format!("KC_{upper}")
                            } else {
                                upper
                            }
                        }
                    }
                })
                .collect();
            all_mapped_layers.push(mapped_keys);
        }

        let json_output = json!({
            "name": layout_name,
            "layers": all_mapped_layers
        });

        Ok(serde_json::to_string_pretty(&json_output)?)
    }
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;

    #[test]
    fn test_via_generate_multi_layer() {
        let exporter = ViaExporter;
        let layers = vec![
            vec!["A".to_string(), "B".to_string()],
            vec!["TRNS".to_string(), "NO".to_string()],
        ];
        let result = exporter.generate("Test Layout", &layers, None).unwrap();
        let json: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(json["name"], "Test Layout");
        assert_eq!(json["layers"].as_array().unwrap().len(), 2);
        assert_eq!(json["layers"][1][0], "KC_TRNS");
        assert_eq!(json["layers"][1][1], "KC_NO");
    }

    #[test]
    fn test_via_generate_edge_cases() {
        let exporter = ViaExporter;

        // 1. Empty layers
        assert!(exporter.generate("fail", &[], None).is_err());

        // 2. Normalization branches
        let layers = vec![vec![
            "KC_A".to_string(),  // already starts with KC_
            "mo(1)".to_string(), // contains (
            ".".to_string(),     // non-alphanumeric
        ]];
        let result = exporter.generate("Test", &layers, None).unwrap();
        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(json["layers"][0][0], "KC_A");
        assert_eq!(json["layers"][0][1], "MO(1)");
        assert_eq!(json["layers"][0][2], ".");
    }
}
