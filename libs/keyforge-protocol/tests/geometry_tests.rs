use keyforge_protocol::geometry::{KeyNode, KeyboardGeometry};
use keyforge_protocol::types::{KeyIndex, HandIndex, FingerIndex};
use keyforge_protocol::Validator;

#[test]
fn test_geometry_valid() {
    let geom = KeyboardGeometry {
        keys: vec![KeyNode::default()],
        prime_slots: vec![KeyIndex(0)],
        med_slots: vec![],
        low_slots: vec![],
        home_row: 1,
    };
    assert!(geom.validate().is_ok());
}

#[test]
fn test_geometry_empty_keys() {
    let geom = KeyboardGeometry {
        keys: vec![],
        prime_slots: vec![],
        med_slots: vec![],
        low_slots: vec![],
        home_row: 1,
    };
    assert!(geom.validate().is_err());
}

#[test]
fn test_geometry_too_many_keys() {
    let keys = vec![KeyNode::default(); 201];
    let geom = KeyboardGeometry {
        keys,
        prime_slots: vec![],
        med_slots: vec![],
        low_slots: vec![],
        home_row: 1,
    };
    assert!(geom.validate().is_err());
}

#[test]
fn test_geometry_overlapping_slots() {
    let geom = KeyboardGeometry {
        keys: vec![KeyNode::default()],
        prime_slots: vec![KeyIndex(0)],
        med_slots: vec![KeyIndex(0)], // Overlap
        low_slots: vec![],
        home_row: 1,
    };
    assert!(geom.validate().is_err());
}

#[test]
fn test_geometry_incomplete_slots() {
    let geom = KeyboardGeometry {
        keys: vec![KeyNode::default(), KeyNode::default()],
        prime_slots: vec![KeyIndex(0)],
        med_slots: vec![],
        low_slots: vec![], // Index 1 missing
        home_row: 1,
    };
    assert!(geom.validate().is_err());
}

#[test]
fn test_geometry_slot_out_of_bounds() {
    let geom = KeyboardGeometry {
        keys: vec![KeyNode::default()],
        prime_slots: vec![KeyIndex(1)], // Out of bounds (len is 1)
        med_slots: vec![],
        low_slots: vec![],
        home_row: 1,
    };
    assert!(geom.validate().is_err());
}

#[test]
fn test_geometry_invalid_dimensions() {
    let mut key = KeyNode::default();
    key.w = 0.0;
    let geom = KeyboardGeometry {
        keys: vec![key],
        prime_slots: vec![KeyIndex(0)],
        med_slots: vec![],
        low_slots: vec![],
        home_row: 1,
    };
    assert!(geom.validate().is_err());
}

#[test]
fn test_geometry_invalid_hand() {
    let mut key = KeyNode::default();
    key.hand = HandIndex(2); // Max is 1
    let geom = KeyboardGeometry {
        keys: vec![key],
        prime_slots: vec![KeyIndex(0)],
        med_slots: vec![],
        low_slots: vec![],
        home_row: 1,
    };
    assert!(geom.validate().is_err());
}

#[test]
fn test_geometry_invalid_finger() {
    let mut key = KeyNode::default();
    key.finger = FingerIndex(5); // Max is 4
    let geom = KeyboardGeometry {
        keys: vec![key],
        prime_slots: vec![KeyIndex(0)],
        med_slots: vec![],
        low_slots: vec![],
        home_row: 1,
    };
    assert!(geom.validate().is_err());
}
