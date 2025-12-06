use keyforge_core::keycodes::KeycodeRegistry;
use keyforge_core::layouts::layout_string_to_u16;

#[test]
fn test_standard_ascii_parsing() {
    let reg = KeycodeRegistry::new_with_defaults();
    let input = "ABC";
    let codes = layout_string_to_u16(input, 3, &reg);
    // Expect lowercase ASCII codes (97='a', 98='b', 99='c')
    assert_eq!(codes, vec![97u16, 98u16, 99u16]);
}

#[test]
fn test_token_parsing() {
    let reg = KeycodeRegistry::new_with_defaults();
    let input = "A [ENT] B [TAB]";
    let codes = layout_string_to_u16(input, 4, &reg);
    // 97='a', 10=Enter, 98='b', 9=Tab
    assert_eq!(codes[0], 97);
    assert_eq!(codes[1], 10);
    assert_eq!(codes[2], 98);
    assert_eq!(codes[3], 9);
}

#[test]
fn test_custom_token_stability() {
    let reg = KeycodeRegistry::new_with_defaults();
    let input1 = "[MOO]";
    let codes1 = layout_string_to_u16(input1, 1, &reg);

    let input2 = "[MOO]";
    let codes2 = layout_string_to_u16(input2, 1, &reg);

    assert_eq!(codes1, codes2);
    assert!(
        codes1[0] >= 0xE000,
        "Custom token should be in dynamic range"
    );
}

#[test]
fn test_mixed_aliasing() {
    let reg = KeycodeRegistry::new_with_defaults();
    let c1 = layout_string_to_u16("[CTRL]", 1, &reg);
    let c2 = layout_string_to_u16("[LCTRL]", 1, &reg);
    assert_eq!(c1[0], 128);
    assert_eq!(c2[0], 128);
}

#[test]
fn test_padding_and_truncation() {
    let reg = KeycodeRegistry::new_with_defaults();
    let input = "AB";
    let codes = layout_string_to_u16(input, 4, &reg);
    // 97='a', 98='b'
    assert_eq!(codes, vec![97, 98, 0, 0]);

    let input_long = "ABCDE";
    let codes_trunc = layout_string_to_u16(input_long, 2, &reg);
    assert_eq!(codes_trunc, vec![97, 98]);
}

#[test]
fn test_qmk_spaced_parsing() {
    let reg = KeycodeRegistry::new_with_defaults();
    let input = "KC_A KC_B KC_ENT";
    let codes = layout_string_to_u16(input, 3, &reg);
    // KC_A maps to 'a' (97), KC_B to 'b' (98)
    assert_eq!(codes, vec![97, 98, 10]);
}
