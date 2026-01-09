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

use crate::Exporter;
use anyhow::Result;
use serde_json::json;

/// An exporter for the VIA keyboard configurator.
///
/// This generates a JSON file that can be imported into the VIA desktop app 
/// or web interface to update a compatible keyboard.
pub struct ViaExporter;

impl Exporter for ViaExporter {
    fn generate(&self, layout_name: &str, layers: &[Vec<String>]) -> Result<String> {
        if layers.is_empty() {
            return Err(anyhow::anyhow!("Layout cannot be empty"));
        }

        let mut all_mapped_layers = Vec::new();

        for keys in layers {
            // VIA "Import Keymap" expects a JSON object with a "layers" array.
            let mapped_keys: Vec<String> = keys
                .iter()
                .map(|k| {
                    let upper = k.to_uppercase();
                    match upper.as_str() {
                        "TRNS" | "_______" => "KC_TRNS".to_string(),
                        "NO" | "XXXXXXX" => "KC_NO".to_string(),
                        _ => {
                            if !upper.starts_with("KC_")
                                && !upper.contains('(')
                                && upper.chars().all(|c| c.is_alphanumeric() || c == '_')
                            {
                                format!("KC_{}", upper)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_via_generate_multi_layer() {
        let exporter = ViaExporter;
        let layers = vec![
            vec!["A".to_string(), "B".to_string()],
            vec!["TRNS".to_string(), "NO".to_string()],
        ];
        let result = exporter.generate("Test Layout", &layers).unwrap();
        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
        
        assert_eq!(json["name"], "Test Layout");
        assert_eq!(json["layers"].as_array().unwrap().len(), 2);
        assert_eq!(json["layers"][1][0], "KC_TRNS");
        assert_eq!(json["layers"][1][1], "KC_NO");
    }
}
