// ===== keyforge/src/api.rs =====
use crate::config::{Config, ScoringWeights};
use crate::geometry::KeyboardDefinition;
use crate::optimizer::mutation;
use crate::scorer::{ScoreDetails, Scorer};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Mutex;

/// The global state required to run KeyForge services.
pub struct KeyForgeState {
    pub scorer: Mutex<Option<Scorer>>,
    pub kb_def: Mutex<Option<KeyboardDefinition>>,
}

impl Default for KeyForgeState {
    fn default() -> Self {
        Self {
            scorer: Mutex::new(None),
            kb_def: Mutex::new(None),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ValidationResult {
    pub layout_name: String,
    pub score: ScoreDetails,
    pub geometry: crate::geometry::KeyboardGeometry,
    pub heatmap: Vec<f32>,
}

/// Service: Initialize the Scorer with data files.
pub fn load_dataset(
    state: &KeyForgeState,
    cost_path: &str,
    ngrams_path: &str,
    keyboard_path: &Option<String>,
    corpus_scale: Option<f32>,
) -> Result<String, String> {
    // Start with defaults
    let mut config = Config::default();

    // Override scale if provided
    if let Some(scale) = corpus_scale {
        config.weights.corpus_scale = scale;
    }

    let kb_path = keyboard_path
        .as_ref()
        .ok_or("Keyboard path is required".to_string())?;

    // Load the Keyboard Bundle
    let kb_def = KeyboardDefinition::load_from_file(kb_path)?;

    // 1. Resolve Weights Strategy (Matching main.rs logic)
    // Map keyboard types to weight filenames
    let weight_filename = match kb_def.meta.kb_type.as_str() {
        "ortho" | "column_staggered" => Some("ortho_split.json"),
        "row_staggered" => Some("row_stagger.json"),
        _ => None,
    };

    if let Some(name) = weight_filename {
        // In the API context, we assume data/weights is relative to the binary or CWD
        // For Tauri apps, paths might need strict handling, but for now we use relative.
        let weight_path = format!("data/weights/{}", name);
        if Path::new(&weight_path).exists() {
            println!("API: Auto-loading weights from {}", weight_path);
            config.weights = ScoringWeights::load_from_file(&weight_path);
        } else {
            println!(
                "API Warning: Weights file '{}' not found. Using defaults.",
                weight_path
            );
        }
    } else {
        println!(
            "API Info: No specific weights profile for type '{}'. Using defaults.",
            kb_def.meta.kb_type
        );
    }

    // Initialize Scorer
    let scorer = Scorer::new(cost_path, ngrams_path, &kb_def.geometry, config, false)?;

    // Store State
    let mut s_guard = state.scorer.lock().map_err(|e| e.to_string())?;
    *s_guard = Some(scorer);

    let mut k_guard = state.kb_def.lock().map_err(|e| e.to_string())?;
    *k_guard = Some(kb_def);

    Ok("Dataset Loaded Successfully".to_string())
}

/// Service: Validate a specific layout string with custom weights.
pub fn validate_layout(
    state: &KeyForgeState,
    layout_str: String,
    weights: Option<ScoringWeights>,
) -> Result<ValidationResult, String> {
    let mut guard = state.scorer.lock().map_err(|e| e.to_string())?;
    let scorer = guard
        .as_mut()
        .ok_or("Scorer not initialized. Load dataset first.")?;

    if let Some(w) = weights {
        scorer.weights = w;
    }

    let key_count = scorer.key_count;

    // Convert input string to bytes
    let mut layout_bytes = vec![0u8; key_count];
    for (i, c) in layout_str
        .to_lowercase()
        .chars()
        .take(key_count)
        .enumerate()
    {
        layout_bytes[i] = c as u8;
    }

    // This uses crate::optimizer::mutation
    let pos_map = mutation::build_pos_map(&layout_bytes);
    let details = scorer.score_debug(&pos_map, 10_000);

    // Heatmap Calculation
    let max_freq_val = scorer.char_freqs.iter().fold(0.0f32, |a, &b| a.max(b));

    let mut heatmap = Vec::with_capacity(key_count);
    for &char_byte in &layout_bytes {
        if char_byte == 0 {
            heatmap.push(0.0);
            continue;
        }
        let freq = scorer.char_freqs[char_byte as usize];
        let intensity = if max_freq_val > 0.0 {
            freq / max_freq_val
        } else {
            0.0
        };
        heatmap.push(intensity);
    }

    Ok(ValidationResult {
        layout_name: "Custom".to_string(),
        score: details,
        geometry: scorer.geometry.clone(),
        heatmap,
    })
}
