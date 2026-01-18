// libs/keyforge-physics/src/analysis/fingerprint.rs

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

use keyforge_model::{Layout, KeyCode};
use std::collections::HashMap;
use std::sync::OnceLock;

/// Represents the identity of a layout based on its similarity to standard layouts.
#[derive(Debug, Clone)]
pub struct LayoutIdentity {
    /// The name of the standard layout (e.g., "Qwerty", "Colemak").
    pub name: String,
    /// A similarity score from 0.0 to 1.0.
    pub similarity: f32,
    /// The Hamming distance (number of mismatched keys) from the standard.
    pub distance: usize,
}

#[derive(Debug)]
pub struct Fingerprinter;

static STANDARDS: OnceLock<HashMap<String, Vec<KeyCode>>> = OnceLock::new();

impl Default for Fingerprinter {
    fn default() -> Self {
        Self
    }
}

impl Fingerprinter {
    fn get_standards() -> &'static HashMap<String, Vec<KeyCode>> {
        STANDARDS.get_or_init(|| {
            let mut standards = HashMap::new();
            standards.insert("Qwerty".into(), to_codes("qwertyuiopasdfghjkl;zxcvbnm,./"));
            standards.insert("Colemak".into(), to_codes("qwfpgjluy;arstdhneiozxcvbkm,./"));
            standards.insert("Dvorak".into(), to_codes("',.pyfgcrlaoeuidhtns;qjkxbmwvz"));
            standards
        })
    }

    pub fn identify(layout: &Layout) -> Option<LayoutIdentity> {
        let standards = Self::get_standards();
        let mut best: Option<LayoutIdentity> = None;

        for (name, std_keys) in standards {
            let len = std_keys.len().min(layout.keys.len());
            if len == 0 {
                continue;
            }

            let mut matches = 0;
            for (i, &std_code) in std_keys.iter().enumerate().take(len) {
                if layout.keys[i] == std_code {
                    matches += 1;
                }
            }

            #[allow(clippy::cast_precision_loss)]
            let similarity = matches as f32 / len as f32;
            let distance = len - matches;

            if best.as_ref().is_none_or(|b| similarity > b.similarity) {
                best = Some(LayoutIdentity {
                    name: name.clone(),
                    similarity,
                    distance,
                });
            }
        }

        if let Some(b) = best {
            if b.similarity > 0.2 {
                return Some(b);
            }
        }
        None
    }
}

fn to_codes(s: &str) -> Vec<KeyCode> {
    s.chars().map(|c| KeyCode(c as u16)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fingerprint_identification() {
        let qwerty_str = "qwertyuiopasdfghjkl;zxcvbnm,./";
        let keys: Vec<KeyCode> = qwerty_str.chars().map(|c| KeyCode(c as u16)).collect();
        let layout = Layout::new_unchecked(keys);

        let id = Fingerprinter::identify(&layout);
        assert!(id.is_some());
        let id = id.unwrap();
        assert_eq!(id.name, "Qwerty");
        assert!(id.similarity > 0.9);
    }
}
