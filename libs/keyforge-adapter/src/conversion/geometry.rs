use crate::error::{AdapterError, AdapterResult};
use keyforge_model::{KeyCode, KeyConstraint, geometry};
use keyforge_model::keycodes::KeycodeRegistry;

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
