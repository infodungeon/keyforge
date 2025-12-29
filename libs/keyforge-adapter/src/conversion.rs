use keyforge_model::{self, KeyNode};
use keyforge_protocol::constants::MAX_LAYOUT_DATA_LEN;
use keyforge_protocol::keycodes::KeycodeRegistry;
use keyforge_protocol::{config, geometry, KeyConstraint};

pub fn to_domain_keyboard(geo: &geometry::KeyboardGeometry) -> keyforge_model::Keyboard {
    let keys = geo
        .keys
        .iter()
        .enumerate()
        .map(|(i, k)| KeyNode {
            id: i,
            label: k.id.clone(),
            hand: k.hand,
            finger: k.finger,
            row: k.row,
            col: k.col,
            x: k.x,
            y: k.y,
            is_home: k.row == geo.home_row,
        })
        .collect();

    keyforge_model::Keyboard::new(keys, geo.home_row)
}

pub fn to_domain_rubric(w: &config::ScoringWeights) -> keyforge_model::Rubric {
    let finger_scales = w.get_finger_penalty_scale();
    keyforge_model::Rubric {
        sfb_base: w.penalty_sfb_base,
        sfb_lateral: w.penalty_sfb_lateral,
        travel_lat: w.weight_lateral_travel,
        travel_vert: w.weight_vertical_travel,
        finger_effort: finger_scales,
        redirect: w.penalty_redirect,
        roll_bonus: w.bonus_bigram_roll_in,
        trigram_coverage: w.trigram_coverage,
        trigram_limit: w.loader_trigram_limit,
    }
}

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

pub fn resolve_constraints(
    proto_constraints: &[KeyConstraint],
    key_count: usize,
    registry: &KeycodeRegistry,
) -> Result<Vec<Option<u16>>, String> {
    let mut pins = vec![None; key_count];
    for c in proto_constraints {
        let idx = c.index as usize;
        if idx < key_count {
            // Resolve string key to u16 code
            if let Some(code) = registry.get_code(&c.key) {
                pins[idx] = Some(code);
            } else {
                // Try parsing as number if lookup fails (backward compatibility/direct ID)
                if let Ok(code) = c.key.parse::<u16>() {
                    pins[idx] = Some(code);
                } else {
                    return Err(format!("Unknown key in constraint: {}", c.key));
                }
            }
        } else {
            return Err(format!(
                "Constraint index {} out of bounds (max {})",
                idx, key_count
            ));
        }
    }
    Ok(pins)
}

pub fn resolve_cost_matrix(
    raw_data: &[(String, String, f32)],
    geo: &geometry::KeyboardGeometry,
) -> Vec<(usize, usize, f32)> {
    let mut overrides = Vec::new();
    let mut id_map = std::collections::HashMap::new();
    for (i, k) in geo.keys.iter().enumerate() {
        id_map.insert(k.id.clone(), i);
    }
    for (id1, id2, cost) in raw_data {
        if let (Some(&idx1), Some(&idx2)) = (id_map.get(id1), id_map.get(id2)) {
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
) -> Result<keyforge_model::Layout, String> {
    if s.len() > MAX_LAYOUT_DATA_LEN {
        return Err(format!(
            "Layout string exceeds maximum length of {}",
            MAX_LAYOUT_DATA_LEN
        ));
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

        // 2. Try stripping arguments (e.g. "MO(1)" -> "MO")
        let base_token = if let Some(idx) = token.find('(') {
            &token[..idx]
        } else {
            token
        };

        if let Some(code) = registry.get_code(base_token) {
            keys.push(code);
        } else {
            if token.len() == 1 {
                let c = token.chars().next().unwrap();
                if c.is_ascii() {
                    keys.push(c as u16);
                    continue;
                }
            }
            // Strict parsing: Don't silently insert 0 for unknown tokens
            return Err(format!("Unknown key token: {}", token));
        }
    }

    while keys.len() < size {
        keys.push(0);
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

        let base_token = if let Some(idx) = token.find('(') {
            &token[..idx]
        } else {
            token
        };

        if let Some(code) = registry.get_code(base_token) {
            keys.push(code);
        } else if token.len() == 1 {
            if let Some(c) = token.chars().next() {
                if c.is_ascii() {
                    keys.push(c as u16);
                } else {
                    keys.push(0);
                }
            } else {
                keys.push(0);
            }
        } else {
            keys.push(0);
        }
    }

    while keys.len() < size {
        keys.push(0);
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
) -> Result<keyforge_model::Layout, String> {
    parse_layout_string_strict(s, size, registry)
}
