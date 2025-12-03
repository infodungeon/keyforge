use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct BenchmarkEntry {
    pub layout: String,
    pub effort: f32,
    pub distance: f32,
    pub sfb: f32,
    pub lateral_stretch: f32,
    pub pinky_scissors: f32,
    pub tri_redirect: f32,
    pub roll_in: f32,
    pub roll_out: f32,
    pub skip_bigrams: f32,
}

impl Default for BenchmarkEntry {
    fn default() -> Self {
        Self {
            layout: "Unknown".to_string(),
            effort: 0.0,
            distance: 0.0,
            sfb: 0.0,
            lateral_stretch: 0.0,
            pinky_scissors: 0.0,
            tri_redirect: 0.0,
            roll_in: 0.0,
            roll_out: 0.0,
            skip_bigrams: 0.0,
        }
    }
}

pub fn load() -> Option<Vec<BenchmarkEntry>> {
    let path = "data/benchmarks/cyanophage.json";

    if !Path::new(path).exists() {
        eprintln!("⚠️  Notice: Benchmark file '{}' not found.", path);
        eprintln!("    (The 'Reality Check' table will be skipped.)");
        return None;
    }

    match fs::read_to_string(path) {
        Ok(content) => match serde_json::from_str(&content) {
            Ok(data) => Some(data),
            Err(e) => {
                eprintln!("❌ Error parsing benchmark JSON: {}", e);
                None
            }
        },
        Err(e) => {
            eprintln!("❌ Error reading benchmark file: {}", e);
            None
        }
    }
}