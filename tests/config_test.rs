use keyforge::config::{Config, LayoutDefinitions, ScoringWeights, SearchParams};

// Helper to create a default config for testing without relying on Clap parsing
fn get_default_test_config() -> Config {
    Config {
        search: SearchParams {
            search_epochs: 1,
            search_steps: 1,
            search_patience: 1,
            search_patience_threshold: 0.1,
            temp_min: 0.1,
            temp_max: 1.0,
            opt_limit_fast: 100,
            opt_limit_slow: 100,
        },
        weights: ScoringWeights {
            // SFR
            penalty_sfr_weak_finger: 20.0,
            penalty_sfr_bad_row: 25.0,
            penalty_sfr_lat: 40.0,

            // SFB
            penalty_sfb_base: 50.0,
            penalty_sfb_lateral: 35.0,
            penalty_sfb_lateral_weak: 160.0,
            penalty_sfb_outward_adder: 5.0,
            penalty_sfb_diagonal: 70.0,
            penalty_sfb_long: 90.0,
            penalty_sfb_bottom: 110.0,
            weight_weak_finger_sfb: 2.7,

            // Other
            penalty_scissor: 25.0,
            penalty_ring_pinky: 1.3,
            penalty_lateral: 35.0,

            // Flow
            penalty_skip: 20.0,
            penalty_redirect: 15.0,
            penalty_hand_run: 5.0,
            bonus_inward_roll: 60.0,

            // Tier
            penalty_high_in_med: 5.0,
            penalty_high_in_low: 20.0,
            penalty_med_in_prime: 2.0,
            penalty_med_in_low: 10.0,
            penalty_low_in_prime: 15.0,
            penalty_low_in_med: 2.0,

            // System
            penalty_imbalance: 200.0,
            max_hand_imbalance: 0.55,
            weight_geo_dist: 10.0,
            weight_finger_effort: 0.5,
            corpus_scale: 1.0,
            default_cost_ms: 120.0,
            finger_penalty_scale: "0.0,1.0,1.1,1.3,1.6".to_string(),
            finger_repeat_scale: "1.0,1.0,1.0,1.2,1.5".to_string(),
        },
        defs: LayoutDefinitions {
            tier_high_chars: "etaoinshr".to_string(),
            tier_med_chars: "ldcumwfgypb.,".to_string(),
            tier_low_chars: "vkjxqz/;".to_string(),
            critical_bigrams: "th,he".to_string(),
            finger_repeat_scale: "1.0,1.0,1.0,1.2,1.5".to_string(),
        },
    }
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
#[should_panic(expected = "is not 2 chars")]
fn test_critical_bigram_parsing_invalid_panics() {
    let mut config = get_default_test_config();
    config.defs.critical_bigrams = "th, abc, t, he".to_string();
    config.defs.get_critical_bigrams();
}
