use crate::error::{AdapterError, AdapterResult};
use keyforge_model::keycodes::KeycodeRegistry;
use keyforge_model::{geometry, KeyCode, KeyConstraint};

/// Converts protocol-level keyboard geometry into a domain-level keyboard.
///
/// This resolves physical properties like home row positions and calculates
/// internal indices for high-performance scoring.
/// Converts a domain geometry to a model keyboard.
///
/// # Errors
///
/// Returns an `AdapterError::Validation` if the geometry is invalid or the key set is empty.
pub fn to_domain_keyboard(
    geo: &geometry::KeyboardGeometry,
) -> AdapterResult<keyforge_model::Keyboard> {
    let keys = geo
        .keys
        .iter()
        .enumerate()
        .map(|(i, k)| {
            let mut kn = to_domain_keynode(k);
            kn.index = i;
            // Task-adap-rev-001: Only override is_home if it's false in input
            if !k.is_home {
                kn.is_home = k.row == geo.home_row;
            }
            kn
        })
        .collect();

    keyforge_model::Keyboard::new(keys, geo.home_row, String::new())
        .map_err(|e| AdapterError::Validation(format!("Failed to create keyboard: {e}")))
}

/// Converts a protocol-level key node into a domain-level node.
#[must_use]
pub fn to_domain_keynode(k: &geometry::KeyNode) -> keyforge_model::KeyNode {
    keyforge_model::KeyNode {
        index: k.index,
        label: k.label.clone(),
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
/// Resolves constraints into keycodes.
///
/// # Errors
///
/// Returns an `AdapterError::UnknownToken` if a constraint refers to a keycode not in the registry,
/// or `AdapterError::Validation` if a constraint index is out of bounds.
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
                "Constraint index {idx} out of bounds (max {key_count})"
            )));
        }
    }
    Ok(pins)
}

/// Resolves a label-based cost matrix into an index-based override list.
#[must_use]
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

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;
    use keyforge_model::geometry::{KeyNode, KeyboardGeometry};
    use keyforge_model::types::{ColIndex, FingerIndex, HandIndex, KeyIndex, RowIndex};

    #[test]
    fn test_to_domain_keynode_conversion() {
        let proto_key = KeyNode {
            index: 0,
            label: "A".to_string(),
            x: 10.0,
            y: 20.0,
            hand: HandIndex(0),
            finger: FingerIndex(1),
            row: RowIndex(0),
            col: ColIndex(0),
            is_home: true,
            ..Default::default()
        };

        let domain_key = to_domain_keynode(&proto_key);
        assert_eq!(domain_key.index, proto_key.index);
        assert_eq!(domain_key.label, proto_key.label);
    }

    #[test]
    fn test_to_domain_keyboard_conversion() {
        let proto_geo = KeyboardGeometry {
            keys: vec![
                KeyNode {
                    index: 0,
                    label: "A".to_string(),
                    hand: HandIndex(0),
                    finger: FingerIndex(1),
                    row: RowIndex(0),
                    col: ColIndex(0),
                    is_home: true,
                    ..Default::default()
                },
                KeyNode {
                    index: 1,
                    label: "B".to_string(),
                    hand: HandIndex(1),
                    finger: FingerIndex(2),
                    row: RowIndex(0),
                    col: ColIndex(1),
                    is_home: false,
                    ..Default::default()
                },
            ],
            prime_slots: vec![KeyIndex(0)],
            med_slots: vec![KeyIndex(1)],
            low_slots: vec![],
            home_row: RowIndex(0),
        };

        let domain_keyboard = to_domain_keyboard(&proto_geo).expect("Failed to convert keyboard");
        assert_eq!(domain_keyboard.count(), 2);
    }

    #[test]
    fn test_resolve_constraints() {
        let mut reg = KeycodeRegistry::new_with_defaults();
        reg.definitions
            .push(keyforge_model::keycodes::KeycodeDefinition {
                code: KeyCode(10),
                id: "A".into(),
                label: "a".into(),
                aliases: vec![],
            });
        reg.rebuild_maps();

        let constraints = vec![KeyConstraint {
            index: KeyIndex(0),
            key: "A".into(),
        }];
        let pins = resolve_constraints(&constraints, 2, &reg).unwrap();
        assert_eq!(pins[0], Some(KeyCode(10)));
        assert_eq!(pins[1], None);

        // Fail: Unknown token
        let constraints = vec![KeyConstraint {
            index: KeyIndex(0),
            key: "UNKNOWN".into(),
        }];
        assert!(resolve_constraints(&constraints, 2, &reg).is_err());

        // Fail: Out of bounds
        let constraints = vec![KeyConstraint {
            index: KeyIndex(5),
            key: "A".into(),
        }];
        assert!(resolve_constraints(&constraints, 2, &reg).is_err());
    }

    #[test]
    fn test_resolve_cost_matrix() {
        let proto_geo = KeyboardGeometry {
            keys: vec![
                KeyNode {
                    label: "A".into(),
                    ..Default::default()
                },
                KeyNode {
                    label: "B".into(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let raw = vec![("A".to_string(), "B".to_string(), 10.0)];
        let overrides = resolve_cost_matrix(&raw, &proto_geo);
        assert_eq!(overrides.len(), 1);
        assert_eq!(overrides[0], (0, 1, 10.0));
    }
}
