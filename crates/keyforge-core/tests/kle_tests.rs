use keyforge_core::geometry::kle::parse_kle_json;

#[test]
fn test_parse_simple_kle() {
    // [ "Q", {x:1}, "W", {x:0.5}, "E" ]
    // Q: x=0
    // W: x=2 (1u Q width + 1u shift)
    // E: x=3.5 (1u W width + 0.5u shift)

    let json = r#"[
        ["Q", {"x":1}, "W", {"x":0.5}, "E"]
    ]"#;

    let geom = parse_kle_json(json).expect("Failed to parse KLE");
    let keys = geom.keys;

    assert_eq!(keys.len(), 3);

    // Q
    assert_eq!(keys[0].id, "Q");
    assert_eq!(keys[0].x, 0.0);

    // W
    assert_eq!(keys[1].id, "W");
    assert_eq!(keys[1].x, 2.0);

    // E
    assert_eq!(keys[2].id, "E");
    assert_eq!(keys[2].x, 3.5);
}

#[test]
fn test_parse_multiline_kle() {
    // Row 0: Q(0,0), W(1,0) -> Cursor Y increments to 1.
    // Row 1: Modifier {y:0.5} -> Y += 0.5 -> Y = 1.5
    // Keys A, S take this Y.

    let json = r#"[
        ["Q", "W"],
        [{"y": 0.5}, "A", "S"]
    ]"#;

    let geom = parse_kle_json(json).expect("Failed to parse KLE");
    let keys = geom.keys;

    assert_eq!(keys.len(), 4);

    // A
    assert_eq!(keys[2].id, "A");
    assert_eq!(keys[2].y, 1.5);

    // S
    assert_eq!(keys[3].id, "S");
    assert_eq!(keys[3].y, 1.5);
}
