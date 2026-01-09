// apps/keyforge-agent/src/agent/calibration.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use keyforge_core::EngineRequest;
use keyforge_infra::AssetManager;
use keyforge_model::{Corpus, Keyboard, Layout, Rubric, SearchConfig, KeyCode};
use keyforge_model::geometry::KeyboardDefinition;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{info, warn};
use serde::{Deserialize, Serialize};

use crate::agent::errors::AgentError;

#[derive(Serialize, Deserialize)]
struct CalibrationData {
    ips: f64,
    timestamp: u64,
    version: String,
}

/// Measures approximate scoring throughput (iterations per second).
///
/// This function:
/// 1. Checks for a cached `calibration.json`.
/// 2. If missing, ensures the "corne" keyboard asset is available.
/// 3. Runs a physics benchmark.
/// 4. Persists the result.
pub async fn calibrate(assets: &AssetManager, data_root: &Path) -> Result<f64, AgentError> {
    let cal_path = data_root.join("user/calibration.json");

    // 1. Check Cache
    if cal_path.exists() {
        if let Ok(content) = tokio::fs::read_to_string(&cal_path).await {
            if let Ok(data) = serde_json::from_str::<CalibrationData>(&content) {
                info!("Loaded cached calibration: {:.2} kOPS", data.ips / 1000.0);
                return Ok(data.ips);
            }
        }
        warn!("Invalid calibration file, recalibrating...");
    }

    info!("Starting hardware calibration...");

    // 2. Ensure Asset (Corne)
    let kb_path = assets.ensure_keyboard("corne").await
        .map_err(|e| AgentError::Calibration(format!("Failed to fetch reference keyboard: {}", e)))?;

    // 3. Load & Parse
    let content = tokio::fs::read_to_string(&kb_path).await
        .map_err(|e| AgentError::Calibration(format!("Failed to read keyboard: {}", e)))?;
    
    let def: KeyboardDefinition = serde_json::from_str(&content)
        .map_err(|e| AgentError::Calibration(format!("Invalid keyboard JSON: {}", e)))?;

    let keyboard = Keyboard::new(def.geometry.keys, def.geometry.home_row)
        .map_err(|e| AgentError::Calibration(e.to_string()))?;

    // 4. Run Benchmark
    let ips = run_benchmark(keyboard)?;

    // 5. Persist
    let data = CalibrationData {
        ips,
        timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };

    if let Some(parent) = cal_path.parent() {
        tokio::fs::create_dir_all(parent).await.map_err(|e| AgentError::Resource(e.to_string()))?;
    }

    let json = serde_json::to_string(&data).unwrap();
    tokio::fs::write(&cal_path, json).await.map_err(|e| AgentError::Resource(e.to_string()))?;

    info!("Calibration complete: {:.2} kOPS", ips / 1000.0);
    Ok(ips)
}

fn run_benchmark(keyboard: Keyboard) -> Result<f64, AgentError> {
    let key_count = keyboard.keys.len();
    let corpus = Corpus::default();
    let rubric = Rubric::default();
    let search_config = SearchConfig::default();
    
    // Create a dummy layout matching the key count
    let layout = Layout::new_unchecked((0..key_count as u16).map(KeyCode).collect());

    let req = EngineRequest {
        keyboard: Arc::new(keyboard),
        corpus: Arc::new(corpus),
        rubric: Arc::new(rubric),
        config: search_config,
        initial_layout: Some(layout),
        pinned_keys: vec![],
        cost_overrides: vec![],
    };

    // Warmup
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
        iterations += batch as u64;
    }

    let elapsed = start.elapsed().as_secs_f64();
    if elapsed == 0.0 {
        return Ok(0.0);
    }
    
    Ok(iterations as f64 / elapsed)
}
