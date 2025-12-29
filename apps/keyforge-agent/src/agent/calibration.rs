use keyforge_core::EngineRequest;
use keyforge_model::{Corpus, KeyNode, Keyboard, Layout, Rubric, SearchConfig};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::info;

/// Measures approximate scoring throughput (iterations per second).
///
/// This is used to tune worker batching and to report capability to the Hive.
pub fn measure_performance() -> f64 {
    info!("calibrating physics engine");

    let key_count = 30;
    let keys: Vec<KeyNode> = (0..key_count)
        .map(|i| KeyNode {
            id: i,
            label: format!("k{}", i),
            hand: if i < 15 { 0 } else { 1 },
            finger: (i % 5) as u8,
            row: (i / 10) as i8,
            col: (i % 10) as i8,
            x: (i % 10) as f32,
            y: (i / 10) as f32,
            is_home: (10..20).contains(&i),
        })
        .collect();

    let keyboard = Keyboard::new(keys, 1);
    let corpus = Corpus::default();
    let rubric = Rubric::default();
    let config = SearchConfig::default();

    let layout = Layout::new_unchecked((0..key_count as u16).collect());

    let req = EngineRequest {
        keyboard: Arc::new(keyboard),
        corpus: Arc::new(corpus),
        rubric: Arc::new(rubric),
        config,
        initial_layout: Some(layout),
        pinned_keys: vec![],
        cost_overrides: vec![],
    };

    // warm
    for _ in 0..100 {
        let _ = keyforge_core::score(&req);
    }

    let start = Instant::now();
    let duration = Duration::from_millis(1000);
    let mut iterations: u64 = 0;
    let batch = 100;

    while start.elapsed() < duration {
        for _ in 0..batch {
            let _ = keyforge_core::score(&req);
        }
        iterations += batch;
    }

    let elapsed = start.elapsed().as_secs_f64();
    let sops = iterations as f64 / elapsed;

    info!("calibration_result_kops" = sops / 1000.0);

    sops
}
