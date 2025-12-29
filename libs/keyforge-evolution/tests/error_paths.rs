
use keyforge_model::{Corpus, KeyNode, Keyboard, Rubric, SearchConfig};
use keyforge_physics::EngineRequest;
use std::sync::Arc;

#[test]
#[should_panic(expected = "Pinned key 99 not found in initial layout")]
fn test_panic_on_missing_pin() {
    // Setup minimal engine
    let keys = vec![
        KeyNode {
            id: 0,
            label: "k0".into(),
            hand: 0,
            finger: 0,
            row: 0,
            col: 0,
            x: 0.0,
            y: 0.0,
            is_home: false,
        },
        KeyNode {
            id: 1,
            label: "k1".into(),
            hand: 0,
            finger: 1,
            row: 0,
            col: 1,
            x: 1.0,
            y: 0.0,
            is_home: false,
        },
    ];
    let kb = Arc::new(Keyboard::new(keys, 0));
    let corpus = Arc::new(Corpus::default());
    let rubric = Arc::new(Rubric::default());
    


    // Config
    let config = SearchConfig::Annealing {
        steps: 10, start_temp: 1.0, end_temp: 0.1, seed: 42,
        patience: 10, reheats: 0, reheat_factor: 1.0,
    };

    // Pin key 99 (which is not in default 0..N layout)
    let pinned = vec![Some(99), None];

    let req = EngineRequest {
        keyboard: kb,
        corpus,
        rubric,
        config: config.clone(),
        initial_layout: None, // Defaults to [0, 1]
        pinned_keys: pinned,
        cost_overrides: vec![],
    };

    // This should panic
    keyforge_evolution::optimize(&req);
}
