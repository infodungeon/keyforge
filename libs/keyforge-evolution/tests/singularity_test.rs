use keyforge_evolution::supervisor::AnnealingConfig;
use keyforge_evolution::supervisor::strategies::{CoolingAnnealing, GroupMutation};
use keyforge_evolution::supervisor::Optimizer;
use keyforge_model::{Corpus, KeyNode, Keyboard, Rubric};
use keyforge_physics::ScoringEngine;

fn setup_engine() -> ScoringEngine {
    let keys: Vec<_> = (0..2)
        .map(|i| KeyNode {
            id: i,
            label: format!("k{}", i),
            hand: 0,
            finger: 0,
            row: 0,
            col: i as i8,
            x: i as f32,
            y: 0.0,
            is_home: false,
        })
        .collect();
    let kb = Keyboard::new(keys, 0);
    let corpus = Corpus::default();
    ScoringEngine::new(&kb, &corpus, &Rubric::default(), &[]).unwrap()
}

#[test]
fn test_singularity_zero_temp_execution() {
    let engine = setup_engine();
    // Zero temp = Greedy search. Should not NaN.
    let config = AnnealingConfig::new(100, 0.0, 0.0, 42, 10, 0, 1.0).unwrap();

    let mut optimizer = Optimizer::new(
        &engine,
        config,
        GroupMutation {
            unlocked_indices: vec![0, 1],
        },
        CoolingAnnealing,
        keyforge_evolution::supervisor::traits::RealTimeKeeper,
    );

    let result = optimizer.run(None, keyforge_evolution::NoOpCallback);
    assert_eq!(result.keys.len(), 2);
}

#[test]
#[should_panic(expected = "Start temp must be > 0 to enable reheating")]
fn test_singularity_reheat_validation() {
    // Should fail because reheats=1 but start_temp=0.0
    AnnealingConfig::new(100, 0.0, 0.0, 42, 10, 1, 1.0).unwrap();
}
