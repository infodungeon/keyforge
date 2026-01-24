// libs/keyforge-physics/benches/scoring_bench.rs
#![allow(clippy::unwrap_used, clippy::expect_used)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use keyforge_model::{Corpus, CostModel, KeyCode, KeyNode, Keyboard, Layout, Rubric};
use keyforge_physics::{EngineCompilationContext, EngineFactory};

fn bench_scoring(c: &mut Criterion) {
    let keys = vec![KeyNode::default()];
    let kb = Keyboard::new(keys, 0, "test".into()).unwrap();
    let corpus = Corpus::default();
    let rubric = Rubric::default();
    let cost_model = CostModel::default();

    let engine = EngineFactory::new_generic(EngineCompilationContext {
        keyboard: &kb,
        corpus: &corpus,
        rubric: &rubric,
        cost_model: &cost_model,
    })
    .expect("Failed to compile engine");

    let layout = Layout::new_unchecked(vec![KeyCode(0); engine.key_count()]);

    c.bench_function("scalar_score", |b| {
        b.iter(|| engine.score(black_box(&layout)));
    });
}

criterion_group!(benches_group, bench_scoring);
criterion_main!(benches_group);
