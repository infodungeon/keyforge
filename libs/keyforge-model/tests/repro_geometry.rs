use keyforge_model::geometry::KeyboardDefinition;

#[test]
fn test_szr35_deserialization() {
    let json = r#"{
  "meta": {
    "name": "SZR35",
    "author": "KeyForge",
    "version": "1.0",
    "notes": "36-key Split Column-Staggered (3x5+3).",
    "type": "split_column_staggered"
  },
  "geometry": {
    "keys": [
      {"id": "KeyQ", "x": 0, "y": 0.5, "hand": 0, "finger": 4, "row": 0, "col": 0},
      {"id": "KeyW", "x": 1, "y": 0.25, "hand": 0, "finger": 3, "row": 0, "col": 1}
    ],
    "prime_slots": [0, 1],
    "med_slots": [],
    "low_slots": [],
    "home_row": 1
  },
  "layouts": {}
}"#;

    let def: KeyboardDefinition = serde_json::from_str(json).expect("Failed to deserialize");
    assert_eq!(def.geometry.keys.len(), 2, "Should have 2 keys");
    assert_eq!(def.geometry.keys[0].label, "KeyQ", "Label should be KeyQ");
}
