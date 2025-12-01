use crate::config::{Config, ScoringWeights};
use crate::geometry::KeyboardDefinition;
use crate::optimizer::mutation;
use crate::scorer::{ScoreDetails, Scorer, ScorerBuilder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use tracing::{info, warn};

pub struct KeyForgeSession {
    pub scorer: Scorer,
    pub kb_def: KeyboardDefinition,
}

pub struct KeyForgeState {
    pub sessions: Mutex<HashMap<String, KeyForgeSession>>,
}

impl Default for KeyForgeState {
    fn default() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
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

/// Service: Initialize the Scorer with data files into a specific Session ID.
pub fn load_dataset(
    state: &KeyForgeState,
    session_id: &str,
    cost_path: &str,
    ngrams_path: &str,
    keyboard_path: &Option<String>,
    corpus_scale: Option<f32>,
    data_root: Option<&str>,
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

    // 1. Resolve Weights Strategy
    // Map keyboard types to weight filenames
    let weight_filename = match kb_def.meta.kb_type.as_str() {
        "ortho" | "column_staggered" => Some("ortho_split.json"),
        "row_staggered" => Some("row_stagger.json"),
        _ => None,
    };

    if let Some(name) = weight_filename {
        // Construct path relative to data_root if provided, else use relative CWD
        let weight_path = if let Some(root) = data_root {
            format!("{}/data/weights/{}", root, name)
        } else {
            format!("data/weights/{}", name)
        };

        if Path::new(&weight_path).exists() {
            info!("API: Auto-loading weights from {}", weight_path);
            config.weights = ScoringWeights::load_from_file(&weight_path);
        } else {
            warn!(
                "API Warning: Weights file '{}' not found. Using defaults.",
                weight_path
            );
        }
    } else {
        info!(
            "API Info: No specific weights profile for type '{}'. Using defaults.",
            kb_def.meta.kb_type
        );
    }

    // 2. Initialize Scorer using Builder Pattern
    let scorer = ScorerBuilder::new()
        .with_weights(config.weights)
        .with_defs(config.defs)
        .with_geometry(kb_def.geometry.clone())
        .with_costs_from_file(cost_path)
        .map_err(|e| format!("Failed to load Cost Matrix: {}", e))?
        .with_ngrams_from_file(ngrams_path)
        .map_err(|e| format!("Failed to load N-grams: {}", e))?
        .build()
        .map_err(|e| format!("Scorer Initialization Failed: {}", e))?;

    // 3. Store in Session Map
    let mut sessions = state.sessions.lock().map_err(|e| e.to_string())?;

    let session = KeyForgeSession { scorer, kb_def };
    sessions.insert(session_id.to_string(), session);

    Ok(format!("Session '{}' Loaded Successfully", session_id))
}

pub fn validate_layout(
    state: &KeyForgeState,
    session_id: &str,
    layout_str: String,
    weights: Option<ScoringWeights>,
) -> Result<ValidationResult, String> {
    let mut sessions = state.sessions.lock().map_err(|e| e.to_string())?;

    let session = sessions.get_mut(session_id).ok_or_else(|| {
        format!(
            "Session '{}' not found. Please load dataset first.",
            session_id
        )
    })?;

    if let Some(w) = weights {
        session.scorer.weights = w;
    }

    let key_count = session.scorer.key_count;

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

    // RENAMED from score_debug
    let details = session.scorer.score_details(&pos_map, 10_000);

    // Heatmap Calculation
    let max_freq_val = session
        .scorer
        .char_freqs
        .iter()
        .fold(0.0f32, |a, &b| a.max(b));

    let mut heatmap = Vec::with_capacity(key_count);
    for &char_byte in &layout_bytes {
        if char_byte == 0 {
            heatmap.push(0.0);
            continue;
        }
        let freq = session.scorer.char_freqs[char_byte as usize];
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
        geometry: session.scorer.geometry.clone(),
        heatmap,
    })
}
