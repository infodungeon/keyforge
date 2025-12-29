use keyforge_protocol::config::{
    Config, CorpusSource, LayoutDefinitions, ScoringWeights, SearchParams,
};
use keyforge_protocol::geometry::{KeyNode, KeyboardDefinition, KeyboardGeometry, KeyboardMeta};
use keyforge_protocol::job::{JobIdError, JobIdentifier};
use keyforge_protocol::parsing::{parse_key, KeyAction};
use keyforge_protocol::protocol::{
    BiometricSample, CostMatrixSource, JobConfig, JobQueueResponse, JobRequest, JobResponse,
    JobStatus, KeyConstraint, NodeRequest, NodeResponse, PopulationResponse, ResultSubmission,
    SystemMetrics, TuningProfile, UserStatsStore,
};
use keyforge_protocol::Validator;
use std::str::FromStr;

// =============================================================================
// 1. CONFIGURATION (config.rs)
// =============================================================================

#[test]
fn test_config_defaults() {
    let c = Config::default();
    assert!(c.search.validate().is_ok());
    assert!(c.weights.validate().is_ok());
}

#[test]
fn test_corpus_source() {
    // Default
    let c = CorpusSource::default();
    assert_eq!(c.id, "text/en_std");
    assert_eq!(c.weight, 1.0);

    // FromStr Valid
    let c1 = CorpusSource::from_str("rust").unwrap();
    assert_eq!(c1.id, "rust");
    assert_eq!(c1.weight, 1.0);

    let c2 = CorpusSource::from_str("rust:0.5").unwrap();
    assert_eq!(c2.id, "rust");
    assert_eq!(c2.weight, 0.5);

    // FromStr Errors
    assert!(CorpusSource::from_str("rust:invalid").is_err());
    assert!(CorpusSource::from_str("rust:0.0").is_err()); // Epsilon check
    assert!(CorpusSource::from_str("rust:-1.0").is_err());
    assert!(CorpusSource::from_str("rust:NaN").is_err());
}

#[test]
fn test_search_params_validation_exhaustive() {
    let valid = SearchParams::default();

    // Epochs
    let mut p = valid;
    p.search_epochs = 0;
    assert!(p.validate().is_err());
    p.search_epochs = 2_000_000;
    assert!(p.validate().is_err());

    // Steps
    let mut p = valid;
    p.search_steps = 0;
    assert!(p.validate().is_err());
    p.search_steps = 10_000_000;
    assert!(p.validate().is_err());

    // Opt Limits
    let mut p = valid;
    p.opt_limit_fast = 0;
    assert!(p.validate().is_err());
    p.opt_limit_fast = 20_000;
    assert!(p.validate().is_err());

    let mut p = valid;
    p.opt_limit_slow = 50; // < fast (100)
    assert!(p.validate().is_err());

    // Temp
    let mut p = valid;
    p.temp_min = -1.0;
    assert!(p.validate().is_err());

    let mut p = valid;
    p.temp_max = -1.0;
    assert!(p.validate().is_err());

    let mut p = valid;
    p.temp_max = 2000.0;
    assert!(p.validate().is_err());

    let mut p = valid;
    p.temp_min = 0.00001; // Underflow
    assert!(p.validate().is_err());

    let mut p = valid;
    p.temp_min = 30.0; // > max (20.0)
    assert!(p.validate().is_err());

    // Patience
    let mut p = valid;
    p.search_patience_threshold = -0.1;
    assert!(p.validate().is_err());
    p.search_patience_threshold = 1.1;
    assert!(p.validate().is_err());
}

#[test]
fn test_scoring_weights_validation_exhaustive() {
    let valid = ScoringWeights::default();

    // Trigram Limit
    let mut w = valid.clone();
    w.loader_trigram_limit = 100_000;
    assert!(w.validate().is_err());

    // Negative Penalties
    let mut w = valid.clone();
    w.penalty_sfb_base = -1.0;
    assert!(w.validate().is_err());

    let mut w = valid.clone();
    w.penalty_scissor = -1.0;
    assert!(w.validate().is_err());

    // Overflow
    let mut w = valid.clone();
    w.penalty_sfb_base = 2e8;
    assert!(w.validate().is_err());

    let mut w = valid.clone();
    w.penalty_scissor = 2e8;
    assert!(w.validate().is_err());

    let mut w = valid.clone();
    w.penalty_redirect = 2e8;
    assert!(w.validate().is_err());

    // Finger Scale Parsing
    let mut w = valid.clone();
    w.finger_penalty_scale = "1.0".into(); // Wrong count
    assert!(w.validate().is_err());

    let mut w = valid.clone();
    w.finger_penalty_scale = "1.0,1.0,1.0,1.0,bad".into(); // Parse error
    assert!(w.validate().is_err());

    let mut w = valid.clone();
    w.finger_penalty_scale = "1.0,1.0,1.0,1.0,NaN".into(); // Finite check
    assert!(w.validate().is_err());

    // Getters
    let w = valid.clone();
    assert_eq!(w.get_finger_penalty_scale(), [0.0; 5]); // Default empty string -> 0.0s

    let mut w = valid.clone();
    w.finger_penalty_scale = "1.0,1.0,1.0,1.0,1.0".into();
    assert_eq!(w.get_finger_penalty_scale(), [1.0; 5]);

    assert!(w.allowed_hand_balance_deviation() >= 0.0);

    let mut w = valid.clone();
    w.comfortable_scissors = "01, 12".into();
    let scissors = w.get_comfortable_scissors();
    assert_eq!(scissors.len(), 2);
    assert_eq!(scissors[0], (0, 1));
}

#[test]
fn test_layout_definitions() {
    let d = LayoutDefinitions::default();
    let bigrams = d.get_critical_bigrams();
    assert!(!bigrams.is_empty());
    assert_eq!(bigrams[0], [b't', b'h']);
}

// =============================================================================
// 2. GEOMETRY (geometry.rs)
// =============================================================================

#[test]
fn test_geometry_validation_exhaustive() {
    let mut geom = KeyboardGeometry::default();
    assert!(geom.validate().is_err()); // Empty

    // Basic Valid
    geom.keys = vec![KeyNode::default()];
    geom.prime_slots = vec![0];
    assert!(geom.validate().is_ok());

    // Too many keys
    geom.keys = vec![KeyNode::default(); 201];
    assert!(geom.validate().is_err());
    geom.keys = vec![KeyNode::default()]; // Reset

    // Slot Overlaps
    geom.med_slots = vec![0]; // Overlap Prime
    assert!(geom.validate().is_err());
    geom.med_slots = vec![];

    geom.low_slots = vec![0]; // Overlap Prime
    assert!(geom.validate().is_err());
    geom.low_slots = vec![];

    // Incomplete Slots
    geom.keys.push(KeyNode::default()); // 2 keys
                                        // Index 1 not assigned
    assert!(geom.validate().is_err());
    geom.keys.pop(); // Reset

    // Out of Bounds
    geom.prime_slots = vec![1];
    assert!(geom.validate().is_err());
    geom.prime_slots = vec![0];

    // Invalid Key Props
    geom.keys[0].w = 0.0;
    assert!(geom.validate().is_err());
    geom.keys[0].w = 1.0;

    geom.keys[0].hand = 2;
    assert!(geom.validate().is_err());
    geom.keys[0].hand = 0;

    geom.keys[0].finger = 5;
    assert!(geom.validate().is_err());
}

#[test]
fn test_keyboard_definition_parsing() {
    // JSON
    let json = r#"{ "meta": { "name": "Test" }, "geometry": { "keys": [], "prime_slots": [], "med_slots": [], "low_slots": [], "home_row": 1 } }"#;
    let def = KeyboardDefinition::parse(json, None).unwrap();
    assert_eq!(def.meta.name, "Test");

    // KLE
    let kle = r#"[{"name": "KLE"}, [{"a":7}, "A"]]"#;
    let def_kle = KeyboardDefinition::parse(kle, Some("Imported")).unwrap();
    assert_eq!(def_kle.meta.name, "Imported");
    assert_eq!(def_kle.geometry.keys.len(), 1);

    // Invalid
    assert!(KeyboardDefinition::parse("invalid", None).is_err());
}

// =============================================================================
// 3. PROTOCOL & JOB (protocol.rs, job.rs)
// =============================================================================

#[test]
fn test_cost_matrix_source() {
    let p = CostMatrixSource::Predefined("file.json".into());
    assert_eq!(format!("{}", p), "file.json");

    let c = CostMatrixSource::Custom("data".into());
    assert_eq!(format!("{}", c), "<custom_content>");

    assert_eq!(
        CostMatrixSource::default(),
        CostMatrixSource::Predefined("default_costmatrix.json".into())
    );
}

#[test]
fn test_key_constraint() {
    let kc = KeyConstraint::from_str("10:KC_A").unwrap();
    assert_eq!(kc.index, 10);
    assert_eq!(kc.key, "KC_A");

    assert!(KeyConstraint::from_str("").is_err());
    assert!(KeyConstraint::from_str("no_colon").is_err());
    assert!(KeyConstraint::from_str("abc:KC_A").is_err());
    assert!(KeyConstraint::from_str("10:").is_err());
}

#[test]
fn test_result_submission_timestamp() {
    let mut sub = ResultSubmission {
        version: 1,
        job_id: "j".into(),
        layout: "l".into(),
        score: 0.0,
        timestamp: 0,
        nonce: 0,
        node_id: "n".into(),
        signature: None,
    };

    // Too old
    assert!(sub.validate_timestamp().is_err());

    // Future
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    sub.timestamp = now + 1000;
    assert!(sub.validate_timestamp().is_err());

    // Valid
    sub.timestamp = now;
    assert!(sub.validate_timestamp().is_ok());
}

#[test]
fn test_job_request_validation_exhaustive() {
    let mut req = JobRequest {
        version: 1,
        definition: KeyboardDefinition::default(),
        weights: ScoringWeights::default(),
        params: SearchParams::default(),
        pinned_keys: vec![],
        corpora: vec![],
        cost_matrix: CostMatrixSource::default(),
        biometrics: vec![],
        parent_job_id: None,
        baseline_score: None,
        parents: vec![],
    };

    // Setup valid geometry
    req.definition.geometry.keys = vec![KeyNode::default()];
    req.definition.geometry.prime_slots = vec![0];

    // Keys Limit
    req.definition.geometry.keys = vec![KeyNode::default(); 201];
    assert!(req.validate().is_err());
    req.definition.geometry.keys = vec![KeyNode::default()];

    // Pins Limit
    req.pinned_keys = vec![
        KeyConstraint {
            index: 0,
            key: "A".into()
        };
        201
    ];
    assert!(req.validate().is_err());
    req.pinned_keys.clear();

    // Biometrics Limit
    req.biometrics = vec![
        BiometricSample {
            bigram: "ab".into(),
            ms: 10.0,
            timestamp: 0
        };
        10_001
    ];
    assert!(req.validate().is_err());
    req.biometrics.clear();

    // Cost Matrix Empty
    req.cost_matrix = CostMatrixSource::Predefined("".into());
    assert!(req.validate().is_err());
    req.cost_matrix = CostMatrixSource::Custom("".into());
    assert!(req.validate().is_err());
    req.cost_matrix = CostMatrixSource::Custom("invalid".into()); // No comma
    assert!(req.validate().is_err());

    // Pin Out of Bounds
    req.cost_matrix = CostMatrixSource::default();
    req.pinned_keys = vec![KeyConstraint {
        index: 5,
        key: "A".into(),
    }];
    assert!(req.validate().is_err());
}

#[test]
fn test_job_config_conversion() {
    let req = JobRequest {
        version: 1,
        definition: KeyboardDefinition::default(),
        weights: ScoringWeights::default(),
        params: SearchParams::default(),
        pinned_keys: vec![],
        corpora: vec![],
        cost_matrix: CostMatrixSource::default(),
        biometrics: vec![],
        parent_job_id: Some("p".into()),
        baseline_score: Some(1.0),
        parents: vec!["l".into()],
    };
    let conf: JobConfig = req.into();
    assert_eq!(conf.parent_job_id, Some("p".into()));
}

#[test]
fn test_job_identifier() {
    let geom = KeyboardGeometry::default();
    let weights = ScoringWeights::default();
    let params = SearchParams::default();
    let pins = vec![];
    let corpus = "default";
    let cost = CostMatrixSource::default();

    // Legacy wrapper
    let _ = JobIdentifier::from_parts(&geom, &weights, &params, &pins, corpus, &cost);

    // Error display
    let err = JobIdError::Serialize("fail".into());
    assert!(format!("{}", err).contains("fail"));
}

// =============================================================================
// 4. PARSING (parsing.rs)
// =============================================================================

#[test]
fn test_parsing_exhaustive() {
    // Simple
    assert_eq!(parse_key("A"), KeyAction::Simple("KC_A".into()));

    // Constants
    assert_eq!(parse_key("TRNS"), KeyAction::Transparent);
    assert_eq!(parse_key("NO"), KeyAction::NoOp);
    assert_eq!(parse_key("CW"), KeyAction::CapsWord);

    // Layers
    assert_eq!(parse_key("MO(1)"), KeyAction::LayerMomentary(1));
    assert_eq!(parse_key("TG(1)"), KeyAction::LayerToggle(1));
    assert_eq!(parse_key("TO(1)"), KeyAction::LayerOn(1));

    // Layer Limit
    assert!(matches!(parse_key("MO(32)"), KeyAction::Raw(_)));

    // Taps
    match parse_key("LSFT_T(KC_A)") {
        KeyAction::ModTap { mod_name, key } => {
            assert_eq!(mod_name, "LSFT");
            assert_eq!(key, "KC_A");
        }
        _ => panic!("ModTap failed"),
    }

    match parse_key("LT(1, KC_A)") {
        KeyAction::LayerTap { layer, key } => {
            assert_eq!(layer, 1);
            assert_eq!(key, "KC_A");
        }
        _ => panic!("LayerTap failed"),
    }

    // Sticky
    assert!(matches!(parse_key("SK(LSFT)"), KeyAction::StickyMod(_)));

    // Fallback / Raw
    assert!(matches!(parse_key("UNKNOWN(1)"), KeyAction::Raw(_)));

    // Length Limit
    let long = "A".repeat(40);
    assert!(matches!(parse_key(&long), KeyAction::Raw(s) if s.len() == 32));
}

// =============================================================================
// 5. STRUCT DEFAULTS (Coverage Fillers)
// =============================================================================

#[test]
fn test_struct_defaults() {
    let _ = JobResponse {
        job_id: "".into(),
        is_new: false,
    };
    let _ = JobQueueResponse {
        job_id: None,
        config: None,
    };
    let _ = PopulationResponse { layouts: vec![] };
    let _ = NodeRequest {
        version: 1,
        node_id: "".into(),
        cpu_model: "".into(),
        cores: 1,
        l2_cache_kb: None,
        ops_per_sec: 1.0,
        public_key: None,
    };
    let _ = NodeResponse {
        status: "".into(),
        tuning: TuningProfile {
            strategy: "".into(),
            batch_size: 1,
            thread_count: 1,
        },
    };
    let _ = SystemMetrics::default();
    let _ = JobStatus {
        job_id: "".into(),
        status: "".into(),
        active_nodes: 0,
        best_score: None,
        best_layout: None,
        total_samples: 0,
    };
    let _ = UserStatsStore::default();
    let _ = KeyboardMeta::default();
}
