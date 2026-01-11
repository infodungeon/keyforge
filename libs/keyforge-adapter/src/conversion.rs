// libs/keyforge-adapter/src/conversion.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::error::{AdapterError, AdapterResult};
use keyforge_model::KeyCode;
use keyforge_model::constants::MAX_LAYOUT_DATA_LEN;
use keyforge_model::keycodes::KeycodeRegistry;
use keyforge_model::{config, geometry, KeyConstraint};

/// Converts a protocol-level corpus source into a domain-level source.
pub fn to_domain_corpus_source(s: &config::CorpusSource) -> config::CorpusSource {
    config::CorpusSource {
        id: s.id.clone(),
        weight: s.weight,
        hash: s.hash.clone(),
    }
}

/// Converts protocol-level keyboard geometry into a domain-level keyboard.
///
/// This resolves physical properties like home row positions and calculates 
/// internal indices for high-performance scoring.
pub fn to_domain_keyboard(geo: &geometry::KeyboardGeometry) -> AdapterResult<keyforge_model::Keyboard> {
    let keys = geo
        .keys
        .iter()
        .enumerate()
        .map(|(i, k)| {
            let mut kn = to_domain_keynode(k.clone());
            kn.index = i;
            kn.is_home = k.row.0 == geo.home_row;
            kn
        })
        .collect();

    keyforge_model::Keyboard::new(keys, geo.home_row)
        .map_err(|e| AdapterError::Validation(format!("Failed to create keyboard: {}", e)))
}

/// Converts protocol-level scoring weights into a domain-level evaluation rubric.
pub fn to_domain_rubric(w: &config::ScoringWeights) -> keyforge_model::Rubric {
    keyforge_model::Rubric {
        finger_effort: w.get_finger_penalty_scale(),
        travel_lat: w.weight_lateral_travel,
        travel_vert: w.weight_vertical_travel,
        sfb_base: w.penalty_sfb_base,
        sfb_lateral: w.penalty_sfb_lateral,
        sfb_lateral_weak: w.penalty_sfb_lateral_weak,
        sfb_diagonal: w.penalty_sfb_diagonal,
        sfb_long: w.penalty_sfb_long,
        threshold_sfb_long_row_diff: w.threshold_sfb_long_row_diff,
        penalty_scissor: w.penalty_scissor,
        threshold_scissor_row_diff: w.threshold_scissor_row_diff,
        redirect: w.penalty_redirect,
        roll_bonus: w.bonus_bigram_roll_in,
        trigram_coverage: w.trigram_coverage,
        trigram_limit: w.loader_trigram_limit,
    }
}

/// Converts protocol-level search parameters into domain-level search configuration.
pub fn to_domain_config(p: &config::SearchParams, seed: u64) -> keyforge_model::SearchConfig {
    keyforge_model::SearchConfig::Annealing {
        steps: p.search_steps,
        start_temp: p.temp_max,
        end_temp: p.temp_min,
        seed,
        patience: p.search_patience,
        reheats: p.reheats,
        reheat_factor: p.reheat_factor,
    }
}

/// Resolves a list of protocol constraints against a keycount and registry.
///
/// Returns a vector of optional keycodes where each `Some(code)` represents 
/// a pinned key at that index.
pub fn resolve_constraints(
    proto_constraints: &[KeyConstraint],
    key_count: usize,
    registry: &KeycodeRegistry,
) -> AdapterResult<Vec<Option<KeyCode>>> {
    let mut pins = vec![None; key_count];
    for c in proto_constraints {
        let idx = usize::from(c.index);
        if idx < key_count {
            // Resolve string key to u16 code
            if let Some(code) = registry.get_code(&c.key) {
                pins[idx] = Some(code);
            } else {
                return Err(AdapterError::UnknownToken(c.key.clone()));
            }
        } else {
            return Err(AdapterError::Validation(format!(
                "Constraint index {} out of bounds (max {})",
                idx, key_count
            )));
        }
    }
    Ok(pins)
}

/// Resolves a label-based cost matrix into an index-based override list.
pub fn resolve_cost_matrix(
    raw: &[(String, String, f32)],
    geo: &geometry::KeyboardGeometry,
) -> Vec<(usize, usize, f32)> {
    let mut overrides = Vec::new();
    let mut id_map = std::collections::HashMap::new();
    for (i, k) in geo.keys.iter().enumerate() {
        id_map.insert(k.label.clone(), i);
    }
    for (from, to, cost) in raw {
        if let (Some(&idx1), Some(&idx2)) = (id_map.get(from), id_map.get(to)) {
            overrides.push((idx1, idx2, *cost));
        }
    }
    overrides
}



/// Strict layout-string parsing.
///
/// - Unknown tokens are treated as errors.
/// - Intended for server-side verification and other trust boundaries.
pub fn parse_layout_string_strict(
    s: &str,
    size: usize,
    registry: &KeycodeRegistry,
) -> AdapterResult<keyforge_model::Layout> {
    if s.len() > MAX_LAYOUT_DATA_LEN {
        return Err(AdapterError::LayoutTooLong(MAX_LAYOUT_DATA_LEN));
    }

    let mut keys = Vec::with_capacity(size);
    let tokens: Vec<&str> = s.split_whitespace().collect();

    for token in tokens {
        if keys.len() >= size {
            break;
        }

        // 1. Try exact match first (e.g. "MO(1)")
        if let Some(code) = registry.get_code(token) {
            keys.push(code);
            continue;
        }

        // 2. Try stripping arguments safely (e.g. "MO(1)" -> "MO")
        let base_token = if token.ends_with(')') {
            if let Some(idx) = token.find('(') {
                &token[..idx]
            } else {
                token
            }
        } else {
            token
        };

        if let Some(code) = registry.get_code(base_token) {
            keys.push(code);
        } else {
            // Strict parsing: remove length-1 ASCII backdoor. Everything must be in the registry.
            return Err(AdapterError::UnknownToken(token.to_string()));
        }
    }

    while keys.len() < size {
        keys.push(KeyCode(0));
    }

    Ok(keyforge_model::Layout::new_unchecked(keys))
}

/// Permissive layout-string parsing.
///
/// - Unknown tokens are replaced with 0 (KC_NO).
/// - Intended for UI/CLI convenience.
pub fn parse_layout_string_permissive(
    s: &str,
    size: usize,
    registry: &KeycodeRegistry,
) -> keyforge_model::Layout {
    let mut keys = Vec::with_capacity(size);
    let tokens: Vec<&str> = s.split_whitespace().collect();

    for token in tokens {
        if keys.len() >= size {
            break;
        }

        if let Some(code) = registry.get_code(token) {
            keys.push(code);
            continue;
        }

        let base_token = if token.ends_with(')') {
            if let Some(idx) = token.find('(') {
                &token[..idx]
            } else {
                token
            }
        } else {
            token
        };

        if let Some(code) = registry.get_code(base_token) {
            keys.push(code);
        } else {
            keys.push(KeyCode(0));
        }
    }

    while keys.len() < size {
        keys.push(KeyCode(0));
    }

    keyforge_model::Layout::new_unchecked(keys)
}

/// Backwards-compatible alias.
///
/// Prefer `parse_layout_string_strict` or `parse_layout_string_permissive`.
pub fn parse_layout_string(
    s: &str,
    size: usize,
    registry: &KeycodeRegistry,
) -> AdapterResult<keyforge_model::Layout> {
    parse_layout_string_strict(s, size, registry)
}

// Convert Protocol -> Model
/// Converts a protocol-level key node into a domain-level node.
pub fn to_domain_keynode(k: geometry::KeyNode) -> keyforge_model::KeyNode {
    keyforge_model::KeyNode {
        index: k.index,
        label: k.label,
        x: k.x,
        y: k.y,
        w: k.w,
        h: k.h,
        r: k.r,
        rx: k.rx,
        ry: k.ry,
        hand: k.hand,
        finger: k.finger,
        row: k.row,
        col: k.col,
        is_home: k.is_home,
        is_stretch: k.is_stretch,
    }
}

