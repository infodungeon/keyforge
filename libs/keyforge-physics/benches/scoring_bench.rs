// libs/keyforge-physics/benches/scoring_bench.rs
#![allow(clippy::unwrap_used, clippy::expect_used)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use keyforge_model::{Corpus, CostModel, KeyCode, KeyNode, Keyboard, Layout, Rubric};
use keyforge_physics::{EngineCompilationContext, EngineFactory};
use std::sync::Arc;

fn bench_scoring(c: &mut Criterion) {
    let keys = vec![KeyNode::default()];
    let kb = Arc::new(Keyboard::new(keys, keyforge_model::types::RowIndex(0), "test".into()).unwrap());
    let cp = Arc::new(Corpus::default());
    let rubric = Arc::new(Rubric::default());
    let cm = Arc::new(CostModel::default());

    let engine = EngineFactory::new_generic(&EngineCompilationContext {
        keyboard: kb,
        corpus: cp,
        rubric,
        cost_model: cm,
        engine_config: keyforge_model::config::EngineConfig::default(),
    })
    .expect("Failed to compile engine");

    let layout = Layout::new_unchecked(vec![KeyCode(0); engine.key_count()]);

    c.bench_function("scalar_score", |b| {
        b.iter(|| engine.score(black_box(&layout)));
    });
}

criterion_group!(benches_group, bench_scoring);
criterion_main!(benches_group);
