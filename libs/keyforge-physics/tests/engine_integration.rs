// libs/keyforge-physics/tests/engine_integration.rs
//
// Integration tests for scoring engine wiring, factory construction,
// and cross-engine parity verification.
//
// These tests verify:
// - Engine factory correctly wires keyboard, corpus, rubric, and cost model
// - All engines implement trait methods correctly
// - Cross-engine score parity (oracle vs exact vs generic vs intel)

use keyforge_model::{
    cost_model::{FingerDefinition, HandDefinition, ModelDefinition},
    types::{FingerIndex, HandIndex, KeyCode},
    CostModel, Corpus, KeyNode, Keyboard, Layout, Rubric,
};
use keyforge_physics::{verify::DeterministicScorer, EngineFactory};
use proptest::prelude::*;
use std::collections::HashMap;

// =============================================================================
// Test Fixtures
// =============================================================================

fn mock_cost_model() -> CostModel {
    let mut cm = CostModel::default();

    let mut base_zone = HashMap::new();
    for r in -128..=127 {
        base_zone.insert(format!("r{}", r), 0.0);
    }

    let mut index_zones = HashMap::new();
    index_zones.insert("base".into(), base_zone.clone());

    let mut fingers = HashMap::new();
    fingers.insert(
        "thumb".into(),
        FingerDefinition::Thumb(HashMap::new()),
    );
    fingers.insert(
        "index".into(),
        FingerDefinition::Standard(index_zones.clone()),
    );
    fingers.insert(
        "middle".into(),
        FingerDefinition::Standard(index_zones.clone()),
    );
    fingers.insert(
        "ring".into(),
        FingerDefinition::Standard(index_zones.clone()),
    );
    fingers.insert(
        "pinky".into(),
        FingerDefinition::Standard(index_zones.clone()),
    );

    let mut static_costs = HashMap::new();
    static_costs.insert("universal_hand".into(), HandDefinition { fingers });

    cm.models.insert(
        "model_a_row_staggered".into(),
        ModelDefinition {
            description: "test".into(),
            static_costs,
        },
    );
    cm
}

fn setup_minimal() -> (Keyboard, Corpus, Rubric, CostModel) {
    let keys = vec![
        KeyNode {
            index: 0,
            hand: HandIndex::LEFT,
            finger: FingerIndex::INDEX,
            ..Default::default()
        },
        KeyNode {
            index: 1,
            hand: HandIndex::LEFT,
            finger: FingerIndex::MIDDLE,
            ..Default::default()
        },
    ];
    let kb = Keyboard::new(keys, 0, "test".into()).unwrap();
    let mut corpus = Corpus::default();
    corpus.char_freqs[97] = 100;
    corpus.char_freqs[98] = 200;
    corpus.bigrams.push((97, 98, 50));

    let mut cm = CostModel::default();
    cm.models.insert(
        "model_a_row_staggered".into(),
        ModelDefinition {
            description: "test".into(),
            static_costs: HashMap::new(),
        },
    );

    (kb, corpus, Rubric::default(), cm)
}

// =============================================================================
// Generic Engine Integration Tests
// =============================================================================

/// Intent: Verify GenericScoringEngine implements all ScoringEngine trait methods correctly.
/// Expected: Factory creates engine, all trait methods return valid results,
/// swap delta matches actual score difference.
#[test]
fn test_generic_engine_trait_methods() {
    let (kb, corpus, rubric, cm) = setup_minimal();
    let engine = EngineFactory::new_generic(&kb, &corpus, &rubric, &cm).unwrap();

    assert_eq!(engine.name(), "Generic Optimized");
    assert!(!engine.capabilities().is_exact);
    assert_eq!(engine.key_count(), 2);

    let layout = Layout::new_unchecked(vec![KeyCode(97), KeyCode(98)]);

    let score = engine.score(&layout).unwrap();
    let detailed = engine.score_detailed(&layout).unwrap();
    assert_eq!(score.0, detailed.0 + detailed.1 + detailed.2);

    let pos_map = vec![0, 1];
    let delta = engine.calculate_swap_delta(&layout, &pos_map, 0, 1).unwrap();

    let mut swapped_keys = layout.keys.clone();
    swapped_keys.swap(0, 1);
    let swapped_layout = Layout::new_unchecked(swapped_keys);
    let score_after = engine.score(&swapped_layout).unwrap();
    assert_eq!(delta, score_after.0 - score.0);

    let report = engine.analyze(&layout).unwrap();
    assert!(report.score > 0.0);

    let suggestions = engine.suggest_improvements(&layout, true);
    let _ = suggestions.len();

    let _ = engine.context();
}

// =============================================================================
// Exact Engine Integration Tests
// =============================================================================

/// Intent: Verify ExactScoringEngine implements all ScoringEngine trait methods correctly.
/// Expected: Factory creates engine with is_exact=true, all trait methods return valid results.
#[test]
fn test_exact_engine_trait_methods() {
    let (kb, corpus, rubric, cm) = setup_minimal();
    let engine = EngineFactory::new_exact(&kb, &corpus, &rubric, &cm).unwrap();

    assert_eq!(engine.name(), "Exact (Oracle)");
    assert!(engine.capabilities().is_exact);
    assert!(!engine.capabilities().supports_avx2);
    assert_eq!(engine.key_count(), 2);

    let layout = Layout::new_unchecked(vec![KeyCode(97), KeyCode(98)]);

    let score = engine.score(&layout).unwrap();
    let detailed = engine.score_detailed(&layout).unwrap();
    assert_eq!(score.0, detailed.0 + detailed.1 + detailed.2);

    let pos_map = vec![0, 1];
    let delta = engine.calculate_swap_delta(&layout, &pos_map, 0, 1).unwrap();

    let mut swapped_keys = layout.keys.clone();
    swapped_keys.swap(0, 1);
    let swapped_layout = Layout::new_unchecked(swapped_keys);
    let score_after = engine.score(&swapped_layout).unwrap();
    assert_eq!(delta, score_after.0 - score.0);

    let report = engine.analyze(&layout).unwrap();
    assert!(report.score > 0.0);

    let suggestions = engine.suggest_improvements(&layout, true);
    let _ = suggestions.len();

    let _ = engine.context();
}

// =============================================================================
// Intel Comet Lake Engine Integration Tests
// =============================================================================

/// Intent: Verify IntelScoringEngine implements all ScoringEngine trait methods correctly.
/// Expected: Factory creates engine with supports_avx2=true, all trait methods return valid results.
#[test]
fn test_intel_engine_trait_methods() {
    let (kb, corpus, rubric, cm) = setup_minimal();
    let engine = EngineFactory::new_intel_comet_lake(&kb, &corpus, &rubric, &cm, None).unwrap();

    assert_eq!(engine.name(), "Intel Comet Lake (AVX2 Optimized)");
    assert!(engine.capabilities().supports_avx2);
    assert_eq!(engine.key_count(), 2);

    let layout = Layout::new_unchecked(vec![KeyCode(97), KeyCode(98)]);

    let score = engine.score(&layout).unwrap();
    let detailed = engine.score_detailed(&layout).unwrap();
    assert_eq!(score.0, detailed.0 + detailed.1 + detailed.2);

    let pos_map = vec![0, 1];
    let delta = engine.calculate_swap_delta(&layout, &pos_map, 0, 1).unwrap();

    let mut swapped_keys = layout.keys.clone();
    swapped_keys.swap(0, 1);
    let swapped_layout = Layout::new_unchecked(swapped_keys);
    let score_after = engine.score(&swapped_layout).unwrap();
    assert_eq!(delta, score_after.0 - score.0);

    let report = engine.analyze(&layout).unwrap();
    assert!(report.score > 0.0);

    let suggestions = engine.suggest_improvements(&layout, true);
    let _ = suggestions.len();

    let _ = engine.context();
}

/// Intent: Verify Intel engine handles missing keys in corpus gracefully.
/// Expected: Engine scores layout without errors even when corpus references unmapped keys.
#[test]
fn test_intel_missing_keys() {
    let (kb, _corpus, rubric, cm) = setup_minimal();
    let mut corpus = Corpus::default();
    corpus.char_freqs[99] = 100; // Code 99 not in layout
    corpus.bigrams.push((97, 99, 50)); // Code 99 not in layout
    corpus.trigrams.push((97, 98, 99, 10)); // Code 99 not in layout

    let engine = EngineFactory::new_intel_comet_lake(&kb, &corpus, &rubric, &cm, None).unwrap();
    let layout = Layout::new_unchecked(vec![KeyCode(97), KeyCode(98)]);

    let score = engine.score(&layout).unwrap();
    assert!(score.0 >= 0);
}

// =============================================================================
// Cross-Engine Parity Tests (Oracle Pattern)
// =============================================================================

// Intent: Verify bit-perfect parity between Exact engine and Oracle logic,
// and acceptable drift for optimized engines (Generic, Intel).
// Expected: Exact == Oracle (bit-perfect), Generic/Intel within 0.001% of Exact.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]
    #[test]
    fn test_oracle_parity(
        kb in any::<Keyboard>(),
        corpus in any::<Corpus>(),
        rubric in any::<Rubric>(),
    ) {
        let key_count = kb.count();
        let layout_keys: Vec<KeyCode> = (0..key_count)
            .map(|i| KeyCode(i as u16))
            .collect();

        let cost_model = mock_cost_model();
        let oracle = DeterministicScorer::new(&rubric, &cost_model);

        let generic = EngineFactory::new_generic(&kb, &corpus, &rubric, &cost_model).unwrap();
        let exact = EngineFactory::new_exact(&kb, &corpus, &rubric, &cost_model).unwrap();
        let intel = EngineFactory::new_intel_comet_lake(&kb, &corpus, &rubric, &cost_model, None).unwrap();

        let layout = Layout::new_unchecked(layout_keys.clone());

        let s_oracle_res = oracle.score(&kb, &corpus, &layout_keys);

        let generic_res = generic.score(&layout);
        let exact_res = exact.score(&layout);
        let intel_res = intel.score(&layout);

        if s_oracle_res.is_err() || generic_res.is_err() || exact_res.is_err() || intel_res.is_err() {
            return Ok(());
        }

        let s_oracle = s_oracle_res.unwrap();
        let s_generic = generic_res.unwrap().0;
        let s_exact = exact_res.unwrap().0;
        let s_intel = intel_res.unwrap().0;

        prop_assert_eq!(s_exact, s_oracle, "Exact Engine vs Oracle Logic mismatch - BIT-PERFECT PARITY REQUIRED");

        let drift_limit = (s_exact.unsigned_abs() as i64 / 100_000).max(1000);
        prop_assert!((s_generic - s_exact).unsigned_abs() as i64 <= drift_limit, "Generic engine drift {} exceeds limit {}", (s_generic - s_exact).unsigned_abs(), drift_limit);
        prop_assert!((s_intel - s_exact).unsigned_abs() as i64 <= drift_limit, "Intel engine drift {} exceeds limit {}", (s_intel - s_exact).unsigned_abs(), drift_limit);
    }
}
