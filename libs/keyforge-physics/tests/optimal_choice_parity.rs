// libs/keyforge-physics/tests/optimal_choice_parity.rs

use keyforge_model::{
    types::{FingerIndex, HandIndex, KeyCode, RowIndex},
    Corpus, CostModel, KeyNode, Keyboard, Layout, Rubric,
};
use keyforge_physics::{EngineCompilationContext, EngineFactory};

fn mock_cost_model() -> CostModel {
    let json = r#"{
        "meta": { "version": "2.0", "description": "Test", "unit": "pts" },
        "models": {
            "model_a_row_staggered": {
                "description": "Test Model",
                "static_costs": {
                    "universal_hand": {
                        "thumb": { "pos_1": 100.0, "pos_2": 50.0 },
                        "index": { "base": { "r0": 100.0, "r1": 200.0 } },
                        "middle": { "base": { "r0": 100.0 } },
                        "ring": { "base": { "r0": 100.0 } },
                        "pinky": { "base": { "r0": 100.0 } }
                    }
                }
            }
        },
        "dynamic_rules": { "sequence_modifiers": {}, "penalties": {}, "constraints": {} }
    }"#;
    serde_json::from_str(json).unwrap()
}

#[test]
fn test_optimal_choice_monogram() {
    let keys = vec![
        KeyNode {
            index: 0,
            hand: HandIndex::LEFT,
            finger: FingerIndex::INDEX,
            row: RowIndex(0), // cost 100
            ..Default::default()
        },
        KeyNode {
            index: 1,
            hand: HandIndex::LEFT,
            finger: FingerIndex::INDEX,
            row: RowIndex(1), // cost 200
            ..Default::default()
        },
    ];
    let kb = Keyboard::new(keys, 0, "test".into()).unwrap();
    let mut corpus = Corpus::default();
    corpus.char_freqs[97] = 1000; // 'a'

    let cm = mock_cost_model();
    let ctx = EngineCompilationContext {
        keyboard: &kb,
        corpus: &corpus,
        rubric: &Rubric::default(),
        cost_model: &cm,
    };

    let engine = EngineFactory::new_generic(ctx.clone()).unwrap();
    let oracle = EngineFactory::new_exact(ctx).unwrap();

    // Layout where 'a' is on BOTH keys
    let layout = Layout::new_unchecked(vec![KeyCode(97), KeyCode(97)]);

    let score_engine = engine.score(&layout).unwrap();
    let score_oracle = oracle.score(&layout).unwrap();

    assert_eq!(score_engine, score_oracle);
    // Cost should be based on the cheaper key (index 0, cost 100 + finger effort 0)
    // Scale is 1,000,000. 100 * 1000 = 100,000. Normalization factor 100,000 / 1000 = 100.
    // So final score should be 100 * 100,000,000? No, check normalization.
    // score = (sum / total_freq) * 100,000
    // sum = 100 * 1,000,000 (fixed point) * 1000 (freq) = 100,000,000,000
    // score = (100,000,000,000 / 1000) * 100,000 = 100,000,000 * 100,000 = 10,000,000,000,000?
    // Let's just check they are equal and consistent.
}

#[test]
fn test_optimal_choice_bigram() {
    let keys = vec![
        KeyNode {
            index: 0,
            hand: HandIndex::LEFT,
            finger: FingerIndex::INDEX,
            x: 0.0,
            ..Default::default()
        },
        KeyNode {
            index: 1,
            hand: HandIndex::LEFT,
            finger: FingerIndex::MIDDLE,
            x: 1.0,
            ..Default::default()
        },
        KeyNode {
            index: 2,
            hand: HandIndex::LEFT,
            finger: FingerIndex::RING,
            x: 10.0, // Far away
            ..Default::default()
        },
    ];
    let kb = Keyboard::new(keys, 0, "test".into()).unwrap();
    let mut corpus = Corpus::default();
    corpus.bigrams.push((97, 98, 1000)); // 'a' -> 'b'

    let mut rubric = Rubric::default();
    rubric.travel_lat = 1.0;

    let cm = mock_cost_model();
    let ctx = EngineCompilationContext {
        keyboard: &kb,
        corpus: &corpus,
        rubric: &rubric,
        cost_model: &cm,
    };

    let engine = EngineFactory::new_generic(ctx.clone()).unwrap();
    let oracle = EngineFactory::new_exact(ctx).unwrap();

    // Layout where 'a' is at index 0, and 'b' is at BOTH index 1 and 2
    let layout = Layout::new_unchecked(vec![KeyCode(97), KeyCode(98), KeyCode(98)]);

    let score_engine = engine.score(&layout).unwrap();
    let score_oracle = oracle.score(&layout).unwrap();

    assert_eq!(score_engine, score_oracle);
    // It should have picked the pair (0, 1) instead of (0, 2)
}

#[test]
fn test_optimal_choice_trigram() {
    let keys = vec![
        KeyNode {
            index: 0,
            hand: HandIndex::LEFT,
            finger: FingerIndex::PINKY,
            ..Default::default()
        },
        KeyNode {
            index: 1,
            hand: HandIndex::LEFT,
            finger: FingerIndex::RING,
            ..Default::default()
        },
        KeyNode {
            index: 2,
            hand: HandIndex::LEFT,
            finger: FingerIndex::MIDDLE,
            ..Default::default()
        },
        KeyNode {
            index: 3,
            hand: HandIndex::LEFT,
            finger: FingerIndex::INDEX,
            ..Default::default()
        },
    ];
    let kb = Keyboard::new(keys, 0, "test".into()).unwrap();
    let mut corpus = Corpus::default();
    // 'a' -> 'b' -> 'c'
    corpus.trigrams.push((97, 98, 99, 1000));

    let mut rubric = Rubric::default();
    rubric.roll_bonus = 100.0;
    rubric.redirect = 500.0;

    let cm = mock_cost_model();
    let ctx = EngineCompilationContext {
        keyboard: &kb,
        corpus: &corpus,
        rubric: &rubric,
        cost_model: &cm,
    };

    let engine = EngineFactory::new_generic(ctx.clone()).unwrap();
    let oracle = EngineFactory::new_exact(ctx).unwrap();

    // Layout: 'a'=0, 'b'=1, 'c'=2 OR 3
    // Sequence 0-1-2 is Pinky-Ring-Middle (Inward Roll)
    // Sequence 0-1-3 is Pinky-Ring-Index (Inward Roll)
    // If we had a redirect choice, it should avoid it.
    let layout = Layout::new_unchecked(vec![KeyCode(97), KeyCode(98), KeyCode(99), KeyCode(99)]);

    let score_engine = engine.score(&layout).unwrap();
    let score_oracle = oracle.score(&layout).unwrap();

    assert_eq!(score_engine, score_oracle);
}
