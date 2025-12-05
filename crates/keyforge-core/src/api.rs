// ===== keyforge/crates/keyforge-core/src/api.rs =====
use crate::config::{Config, ScoringWeights};
use crate::geometry::KeyboardDefinition;
use crate::keycodes::KeycodeRegistry;
use crate::layouts::layout_string_to_u16;
use crate::optimizer::mutation;
use crate::scorer::{ScoreDetails, Scorer};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::RwLock;
use tracing::{info, warn};

pub struct KeyForgeSession {
    pub scorer: Scorer,
    pub kb_def: KeyboardDefinition,
    pub registry: KeycodeRegistry,
}

pub struct KeyForgeState {
    pub sessions: RwLock<HashMap<String, KeyForgeSession>>,
}

impl Default for KeyForgeState {
    fn default() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
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
    pub penalty_map: Vec<f32>,
}

pub fn load_dataset(
    state: &KeyForgeState,
    session_id: &str,
    cost_path: &str,
    corpus_dir: &str, // RENAMED from ngrams_path
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

    let root_path_str = data_root.unwrap_or(".");
    let root_path = Path::new(root_path_str);

    let weight_filename = match kb_def.meta.kb_type.as_str() {
        "ortho" | "column_staggered" => Some("ortho_split.json"),
        "row_staggered" => Some("row_stagger.json"),
        _ => None,
    };

    if let Some(name) = weight_filename {
        let direct_path = root_path.join("weights").join(name);
        let repo_path = root_path.join("data").join("weights").join(name);

        let final_path = if direct_path.exists() {
            Some(direct_path)
        } else if repo_path.exists() {
            Some(repo_path)
        } else {
            None
        };

        if let Some(p) = final_path {
            info!("API: Auto-loading weights from {:?}", p);
            config.weights = ScoringWeights::load_from_file(&p);
        } else {
            warn!(
                "API Warning: Weights file '{}' not found. Using defaults.",
                name
            );
        }
    }

    let direct_kc = root_path.join("keycodes.json");
    let repo_kc = root_path.join("data").join("keycodes.json");

    let registry = if direct_kc.exists() {
        KeycodeRegistry::load_from_file(&direct_kc)?
    } else if repo_kc.exists() {
        KeycodeRegistry::load_from_file(&repo_kc)?
    } else {
        warn!("API: keycodes.json not found, using built-in defaults.");
        KeycodeRegistry::new_with_defaults()
    };

    // Scorer now accepts directory
    let scorer = Scorer::new(cost_path, corpus_dir, &kb_def.geometry, config, false)
        .map_err(|e| format!("Scorer Initialization Failed: {}", e))?;

    {
        let mut sessions = state.sessions.write().map_err(|e| e.to_string())?;
        sessions.insert(
            session_id.to_string(),
            KeyForgeSession {
                scorer,
                kb_def,
                registry,
            },
        );
    }

    Ok(format!("Session '{}' Loaded Successfully", session_id))
}

pub fn validate_layout(
    state: &KeyForgeState,
    session_id: &str,
    layout_str: String,
    weights: Option<ScoringWeights>,
) -> Result<ValidationResult, String> {
    let sessions = state.sessions.read().map_err(|e| e.to_string())?;

    let session = sessions.get(session_id).ok_or_else(|| {
        format!(
            "Session '{}' not found. Please load dataset first.",
            session_id
        )
    })?;

    let mut scorer_ref = session.scorer.clone();
    if let Some(w) = weights {
        scorer_ref.weights = w;
    }

    let key_count = scorer_ref.key_count;
    let layout_codes = layout_string_to_u16(&layout_str, key_count, &session.registry);
    let pos_map = mutation::build_pos_map(&layout_codes);
    let details = scorer_ref.score_details(&pos_map, 10_000);

    let max_freq_val = scorer_ref.char_freqs.iter().fold(0.0f32, |a, &b| a.max(b));
    let mut heatmap = Vec::with_capacity(key_count);

    for &code in &layout_codes {
        if code == 0 || code >= 256 {
            heatmap.push(0.0);
            continue;
        }
        let byte = code as u8;
        let mut freq = scorer_ref.char_freqs[byte as usize];

        if freq == 0.0 {
            if byte.is_ascii_uppercase() {
                freq = scorer_ref.char_freqs[byte.to_ascii_lowercase() as usize];
            } else if byte.is_ascii_lowercase() {
                freq = scorer_ref.char_freqs[byte.to_ascii_uppercase() as usize];
            }
        }

        let intensity = if max_freq_val > 0.0 {
            freq / max_freq_val
        } else {
            0.0
        };
        heatmap.push(intensity);
    }

    let raw_costs = scorer_ref.get_element_costs(&pos_map);
    let max_cost = raw_costs.iter().fold(0.0f32, |a, &b| a.max(b));

    let mut penalty_map = Vec::with_capacity(key_count);
    for &cost in &raw_costs {
        let intensity = if max_cost > 0.0 { cost / max_cost } else { 0.0 };
        penalty_map.push(intensity);
    }

    Ok(ValidationResult {
        layout_name: "Custom".to_string(),
        score: details,
        geometry: scorer_ref.geometry.clone(),
        heatmap,
        penalty_map,
    })
}
