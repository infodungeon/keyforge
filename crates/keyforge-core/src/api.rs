use crate::config::{Config, ScoringWeights};
use crate::geometry::KeyboardDefinition;
use crate::keycodes::KeycodeRegistry;
use crate::layouts::layout_string_to_u16; // CHANGED
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
    pub registry: KeycodeRegistry,
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
    let mut config = Config::default();

    if let Some(scale) = corpus_scale {
        config.weights.corpus_scale = scale;
    }

    let kb_path = keyboard_path
        .as_ref()
        .ok_or("Keyboard path is required".to_string())?;

    let kb_def = KeyboardDefinition::load_from_file(kb_path)?;

    // Resolve Root Path
    let root_path_str = data_root.unwrap_or(".");
    let root_path = Path::new(root_path_str);

    // 1. Resolve Weights Strategy
    let weight_filename = match kb_def.meta.kb_type.as_str() {
        "ortho" | "column_staggered" => Some("ortho_split.json"),
        "row_staggered" => Some("row_stagger.json"),
        _ => None,
    };

    if let Some(name) = weight_filename {
        // Try direct path (Tauri style: root IS data dir)
        let direct_path = root_path.join("weights").join(name);
        // Try repo path (CLI style: root HAS data dir)
        let repo_path = root_path.join("data").join("weights").join(name);

        let final_path = if direct_path.exists() {
            Some(direct_path.clone())
        } else if repo_path.exists() {
            Some(repo_path.clone())
        } else {
            None
        };

        if let Some(p) = final_path {
            info!("API: Auto-loading weights from {:?}", p);
            config.weights = ScoringWeights::load_from_file(&p);
        } else {
            warn!(
                "API Warning: Weights file '{}' not found in '{:?}' or '{:?}'. Using defaults.",
                name, direct_path, repo_path
            );
        }
    } else {
        info!(
            "API Info: No specific weights profile for type '{}'. Using defaults.",
            kb_def.meta.kb_type
        );
    }

    // 2. Load Keycode Registry
    // Try direct path first, then repo path
    let direct_kc = root_path.join("keycodes.json");
    let repo_kc = root_path.join("data").join("keycodes.json");

    let registry = if direct_kc.exists() {
        info!("API: Loading keycodes from {:?}", direct_kc);
        KeycodeRegistry::load_from_file(&direct_kc)?
    } else if repo_kc.exists() {
        info!("API: Loading keycodes from {:?}", repo_kc);
        KeycodeRegistry::load_from_file(&repo_kc)?
    } else {
        warn!(
            "API: keycodes.json not found at {:?} or {:?}, using built-in defaults.",
            direct_kc, repo_kc
        );
        KeycodeRegistry::new_with_defaults()
    };

    // 3. Initialize Scorer using Builder Pattern
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

    // 4. Store in Session Map
    let mut sessions = state.sessions.lock().map_err(|e| e.to_string())?;

    let session = KeyForgeSession {
        scorer,
        kb_def,
        registry,
    };
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

    // CHANGED: Use u16 parser
    let layout_codes = layout_string_to_u16(&layout_str, key_count, &session.registry);

    // This uses crate::optimizer::mutation
    let pos_map = mutation::build_pos_map(&layout_codes);

    let details = session.scorer.score_details(&pos_map, 10_000);

    // Heatmap Calculation
    let max_freq_val = session
        .scorer
        .char_freqs
        .iter()
        .fold(0.0f32, |a, &b| a.max(b));

    let mut heatmap = Vec::with_capacity(key_count);
    for &code in &layout_codes {
        // Skip KC_NO or custom high-byte codes (macros)
        if code == 0 || code >= 256 {
            heatmap.push(0.0);
            continue;
        }

        let byte = code as u8;
        let mut freq = session.scorer.char_freqs[byte as usize];

        // Handle case folding for heatmap
        if freq == 0.0 {
            if byte.is_ascii_uppercase() {
                freq = session.scorer.char_freqs[byte.to_ascii_lowercase() as usize];
            } else if byte.is_ascii_lowercase() {
                freq = session.scorer.char_freqs[byte.to_ascii_uppercase() as usize];
            }
        }

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
