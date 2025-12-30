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

pub struct ViaExporter;

impl Exporter for ViaExporter {
    fn generate(&self, layout_name: &str, keys: &[String]) -> Result<String> {
        if keys.is_empty() {
            return Err(anyhow::anyhow!("Layout cannot be empty"));
        }

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

        let json_output = json!({
            "name": layout_name,
            "layers": [
                mapped_keys
            ]
        });

        Ok(serde_json::to_string_pretty(&json_output)?)
    }
}
