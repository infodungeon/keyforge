use keyforge_core::config::Config;

fn get_default_test_config() -> Config {
    Config::default()
}

#[test]
fn test_finger_penalty_parsing_defaults() {
    let config = get_default_test_config();
    let expected = [0.0, 1.0, 1.1, 1.3, 1.6];
    let result = config.weights.get_finger_penalty_scale();
    assert_eq!(result, expected);
}

#[test]
fn test_finger_penalty_parsing_custom() {
    let mut config = get_default_test_config();
    config.weights.finger_penalty_scale = "1.0,1.0,1.0,1.0,1.0".to_string();
    let expected = [1.0, 1.0, 1.0, 1.0, 1.0];
    let result = config.weights.get_finger_penalty_scale();
    assert_eq!(result, expected);
}

#[test]
#[should_panic(expected = "requires 5 values")]
fn test_finger_penalty_parsing_partial_panics() {
    let mut config = get_default_test_config();
    config.weights.finger_penalty_scale = "5.0, 5.0, 5.0".to_string();
    config.weights.get_finger_penalty_scale();
}

#[test]
#[should_panic(expected = "Invalid number")]
fn test_finger_penalty_parsing_garbage_panics() {
    let mut config = get_default_test_config();
    config.weights.finger_penalty_scale = "bad, data, here, 1.0, 1.0".to_string();
    config.weights.get_finger_penalty_scale();
}

#[test]
fn test_critical_bigram_parsing() {
    let mut config = get_default_test_config();
    config.defs.critical_bigrams = "th, he, in".to_string();
    let result = config.defs.get_critical_bigrams();
    assert_eq!(result.len(), 3);
    assert_eq!(result[0], [b't', b'h']);
}

#[test]
#[should_panic(expected = "Bad bigram")]
fn test_critical_bigram_parsing_invalid_panics() {
    let mut config = get_default_test_config();
    config.defs.critical_bigrams = "th, abc, t, he".to_string();
    config.defs.get_critical_bigrams();
}
