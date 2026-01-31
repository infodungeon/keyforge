// libs/keyforge-physics/tests/parity_oracle.rs

use keyforge_model::types::{FingerIndex, HandIndex, KeyCode, RowIndex};
use keyforge_model::{Corpus, KeyNode, Keyboard, Layout, Rubric, Score};
use keyforge_physics::{EngineCompilationContext, EngineFactory, ScoringEngine};
use std::sync::Arc;

fn setup_test_keyboard() -> Arc<Keyboard> {
    let keys: Vec<_> = (0..30)
        .map(|i| KeyNode {
            index: i,
            label: format!("k{i}"),
            hand: HandIndex::new(u8::try_from(i % 2).unwrap_or(0)),
            finger: FingerIndex::new_unchecked(u8::try_from(i % 5).unwrap_or(0)),
            row: RowIndex::new(i8::try_from(i / 10).unwrap_or(0)),
            ..Default::default()
        })
        .collect();
    Arc::new(Keyboard::new(keys, RowIndex::new(1), "test".into()).unwrap())
}

fn setup_test_corpus() -> Arc<Corpus> {
    let mut corpus = Corpus::default();
    let mut freqs = corpus.char_freqs.to_vec();
    for i in 33..63 {
        freqs[i] = (i * 10) as u64;
    }
    corpus.char_freqs = Arc::from(freqs);

    let mut bigrams = Vec::new();
    for i in 33..62 {
        bigrams.push((i as u16, (i + 1) as u16, 100));
    }
    corpus.bigrams = Arc::from(bigrams);

    let mut trigrams = Vec::new();
    for i in 33..61 {
        trigrams.push((i as u16, (i + 1) as u16, (i + 2) as u16, 50));
    }
    corpus.trigrams = Arc::from(trigrams);

    Arc::new(corpus)
}

#[keyforge_testing_macros::kf_test]
#[test]
fn test_parity_oracle() {
    let kb = setup_test_keyboard();
    let corpus = setup_test_corpus();
    let rubric = Arc::new(Rubric::default());
    let cost_model = Arc::new(keyforge_model::testing::mock_cost_model());

    let ctx = EngineCompilationContext {
        keyboard: kb.clone(),
        corpus: corpus.clone(),
        rubric: rubric.clone(),
        cost_model: cost_model.clone(),
        engine_config: keyforge_model::config::EngineConfig::default(),
    };

    // 1. Compile all available engines
    let mut engines: Vec<Box<dyn ScoringEngine>> = Vec::new();

    // Always include Scalar
    engines.push(EngineFactory::new_scalar(&ctx).expect("Failed to create Scalar engine"));

    // Include Exact (Oracle)
    engines.push(EngineFactory::new_exact(&ctx).expect("Failed to create Exact engine"));

    // Hardware-specific engines (if supported on this host)
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if is_x86_feature_detected!("avx2") {
            engines.push(
                EngineFactory::new_intel_comet_lake(&ctx, None)
                    .expect("Failed to create AVX2 engine"),
            );
        }
        if is_x86_feature_detected!("avx512f") {
            engines.push(
                EngineFactory::new_intel_avx512(&ctx, None)
                    .expect("Failed to create AVX-512 engine"),
            );
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        engines
            .push(EngineFactory::new_arm_neon(&ctx, None).expect("Failed to create NEON engine"));
        if std::arch::is_aarch64_feature_detected!("sve") {
            engines
                .push(EngineFactory::new_arm_sve(&ctx, None).expect("Failed to create SVE engine"));
        }
    }

    println!("Verifying parity across {} engines", engines.len());

    // 2. Generate 10 standard layouts (shuffled unique character set)
    // Using a set of 30 unique characters to ensure 1:1 mapping and no ambiguity in key choice.
    let mut chars: Vec<u16> = (33..63).collect(); // 30 unique ASCII characters

    for i in 0..10 {
        fastrand::seed(i as u64);
        fastrand::shuffle(&mut chars);

        let keys: Vec<KeyCode> = chars.iter().map(|&c| KeyCode::new(c)).collect();
        let layout = Layout::new_unchecked(keys);

        let mut first_score: Option<Score> = None;
        let mut first_engine_name = "";

        for engine in &engines {
            let score = engine.score(&layout).expect("Scoring failed");

            if let Some(expected) = first_score {
                assert_eq!(
                    score.raw(),
                    expected.raw(),
                    "Parity mismatch at iteration {} between {} and {}! Score: {} vs {}",
                    i,
                    first_engine_name,
                    engine.name(),
                    score.raw(),
                    expected.raw()
                );
            } else {
                first_score = Some(score);
                first_engine_name = engine.name();
            }
        }
    }

    println!("✅ Parity Oracle Verification Passed.");
}
