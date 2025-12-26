#![allow(clippy::unnecessary_map_or)]
use keyforge_model::Layout;
use std::collections::HashMap;
use std::sync::OnceLock;

pub struct LayoutIdentity {
    pub name: String,
    pub similarity: f32, // 0.0 to 1.0
    pub distance: usize, // Hamming distance
}

pub struct Fingerprinter;

static STANDARDS: OnceLock<HashMap<String, Vec<u16>>> = OnceLock::new();

impl Default for Fingerprinter {
    fn default() -> Self {
        Self
    }
}

impl Fingerprinter {
    fn get_standards() -> &'static HashMap<String, Vec<u16>> {
        STANDARDS.get_or_init(|| {
            let mut standards = HashMap::new();
            standards.insert("Qwerty".into(), to_codes("qwertyuiopasdfghjkl;zxcvbnm,./"));
            standards.insert("Colemak".into(), to_codes("qwfpgjluy;arstdhneiozxcvbkm,./"));
            standards.insert("Dvorak".into(), to_codes("',.pyfgcrlaoeuidhtns;qjkxbmwvz"));
            standards
        })
    }

    pub fn identify(&self, layout: &Layout) -> Option<LayoutIdentity> {
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

            let similarity = matches as f32 / len as f32;
            let distance = len - matches;

            if best.as_ref().map_or(true, |b| similarity > b.similarity) {
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

fn to_codes(s: &str) -> Vec<u16> {
    s.chars().map(|c| c as u16).collect()
}
