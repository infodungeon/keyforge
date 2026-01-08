// apps/keyforge-agent/src/agent/calibration.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use keyforge_core::EngineRequest;
use keyforge_model::types::{ColIndex, FingerIndex, HandIndex, RowIndex};
use keyforge_model::{Corpus, KeyNode, Keyboard, Layout, Rubric, SearchConfig, KeyCode};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::info;

use crate::agent::errors::AgentError;

/// Measures approximate scoring throughput (iterations per second).
///
/// This is used to tune worker batching and to report capability to the Hive.
///
/// # Errors
/// Returns `AgentError::Calibration` if the physics engine cannot be initialized.
pub fn measure_performance() -> Result<f64, AgentError> {
    info!("calibrating physics engine");

    let key_count = 30;
    let keys: Vec<KeyNode> = (0..key_count)
        .map(|i| KeyNode {
            index: i,
            label: format!("k{}", i),
            hand: HandIndex(if i < 15 { 0 } else { 1 }),
            finger: FingerIndex((i % 5) as u8),
            row: RowIndex((i / 10) as i8),
            col: ColIndex((i % 10) as i8),
            x: (i % 10) as f32,
            y: (i / 10) as f32,
            is_home: (10..20).contains(&i),
            ..Default::default()
        })
        .collect();

    let keyboard = Keyboard::new(keys, 1).map_err(|e| AgentError::Calibration(e.to_string()))?;
    let corpus = Corpus::default();
    let rubric = Rubric::default();
    let config = SearchConfig::default();

    let layout = Layout::new_unchecked((0..key_count as u16).map(KeyCode).collect());

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

    Ok(sops)
}