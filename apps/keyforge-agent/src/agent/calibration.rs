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

use keyforge_infra::AssetManager;
use keyforge_model::geometry::KeyboardDefinition;
use keyforge_model::types::path::SafePath;
use keyforge_model::{Corpus, CostModel, KeyCode, Keyboard, Layout, Rubric};
use keyforge_physics::{EngineCompilationContext, EngineFactory};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{info, warn};

use crate::agent::errors::AgentError;

#[derive(Serialize, Deserialize)]
struct CalibrationData {
    ips: f64,
    timestamp: u64,
    version: String,
}

fn default_cost_model() -> Result<CostModel, AgentError> {
    let json = r#"{
        "meta": { "version": "2.0", "description": "Calibration", "unit": "pts" },
        "models": {
            "model_a_row_staggered": {
                "description": "Calibration Model",
                "static_costs": {
                    "universal_hand": {
                        "thumb": { "pos_1": 100.0 },
                        "index": { "base": { "r0": 100.0 } },
                        "middle": { "base": { "r0": 100.0 } },
                        "ring": { "base": { "r0": 100.0 } },
                        "pinky": { "base": { "r0": 100.0 } }
                    }
                }
            }
        },
        "dynamic_rules": { "sequence_modifiers": {}, "penalties": {}, "constraints": {} }
    }"#;
    let dto: keyforge_protocol::CostModelDto = serde_json::from_str(json)
        .map_err(|e| AgentError::Calibration(format!("Corrupt default cost model: {e}")))?;
    Ok(dto.into())
}

pub async fn calibrate(
    assets: &AssetManager,
    data_root: &SafePath,
    config: &crate::models::CalibrationConfig,
) -> Result<f64, AgentError> {
    let cal_path = data_root.as_path().join("user/calibration.json");

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

    let kb_path = assets
        .ensure_keyboard("corne")
        .await
        .map_err(|e| AgentError::Calibration(format!("Failed to fetch reference keyboard: {e}")))?;

    let content = tokio::fs::read_to_string(&kb_path)
        .await
        .map_err(|e| AgentError::Calibration(format!("Failed to read keyboard: {e}")))?;

    let def_dto: keyforge_protocol::KeyboardDefinitionDto = serde_json::from_str(&content)
        .map_err(|e| AgentError::Calibration(format!("Invalid keyboard JSON: {e}")))?;
    let def: KeyboardDefinition = def_dto.into();

    let keyboard = Arc::new(Keyboard::new(
        def.geometry.keys().to_vec(),
        def.geometry.home_row(),
        def.meta.kb_type.clone(),
    )?);

    let ips = if config.duration_ms == 0 {
        info!("Skipping hardware calibration (duration_ms=0)");
        1_000_000.0
    } else {
        run_benchmark(&keyboard, config)?
    };

    let data = CalibrationData {
        ips,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };

    if let Some(parent) = cal_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let json = serde_json::to_string(&data)
        .map_err(|e| AgentError::Calibration(format!("Failed to serialize calibration: {e}")))?;
    tokio::fs::write(&cal_path, json).await?;

    info!("Calibration complete: {:.2} kOPS", ips / 1000.0);
    Ok(ips)
}

fn run_benchmark(
    keyboard: &Arc<Keyboard>,
    config: &crate::models::CalibrationConfig,
) -> Result<f64, AgentError> {
    let key_count = keyboard.keys().len();
    let corpus = Arc::new(Corpus::default());
    let rubric = Arc::new(Rubric::default());
    let cost_model = Arc::new(default_cost_model()?);

    let layout = Layout::new_unchecked(
        (0..key_count.try_into().unwrap_or_default())
            .map(KeyCode::new)
            .collect(),
    );

    let engine = EngineFactory::new_generic(&EngineCompilationContext {
        keyboard: keyboard.clone(),
        corpus,
        rubric,
        cost_model,
        engine_config: keyforge_model::config::EngineConfig::default(),
    })?;

    for _ in 0..config.warmup_iterations {
        let _ = engine.score(&layout);
    }

    let start = Instant::now();
    let duration = Duration::from_millis(config.duration_ms);
    let mut iterations: u64 = 0;
    let batch = config.batch_size;

    while start.elapsed() < duration {
        for _ in 0..batch {
            let _ = engine.score(&layout);
        }
        iterations += batch as u64;
    }

    let elapsed = start.elapsed().as_secs_f64();
    if elapsed == 0.0 {
        return Ok(0.0);
    }

    #[allow(clippy::cast_precision_loss)]
    Ok(iterations as f64 / elapsed)
}

#[allow(clippy::cast_precision_loss)]
pub fn measure_performance(config: &crate::models::CalibrationConfig) -> Result<f64, AgentError> {
    let mut keys = Vec::with_capacity(config.key_count);
    for i in 0..config.key_count {
        keys.push(keyforge_model::geometry::KeyNode {
            index: i.into(),
            x: keyforge_model::types::SpatialUnit::from_f32(i as f32),
            y: keyforge_model::types::SpatialUnit::default(),
            ..Default::default()
        });
    }
    let keyboard = Arc::new(Keyboard::new(
        keys,
        keyforge_model::types::RowIndex::new(0),
        "test".into(),
    )?);
    run_benchmark(&keyboard, config)
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;
    use crate::models::CalibrationConfig;

    #[test]
    fn test_performance_calibration() {
        let config = CalibrationConfig {
            key_count: 10,
            warmup_iterations: 1,
            duration_ms: 100,
            batch_size: 10,
        };
        let ops = measure_performance(&config).expect("Performance measurement failed");
        assert!(ops > 0.0, "Calibration should report positive throughput");
    }
}
