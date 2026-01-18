// libs/keyforge-runner/src/lib.rs

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

use keyforge_compute::{Runtime, SessionBuilder};
use keyforge_core::loader::AssetLoader;
use keyforge_model::constants::ASSET_KEYCODES_FILENAME;
use keyforge_protocol::JobConfig;
use keyforge_model::{OptimizationResult, KeyCode};
use keyforge_core::{ScoringSession, ProgressCallback, EvolutionError};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

#[derive(Debug, Clone, Default)]
pub struct RunnerOptions {
    pub timeout_sec: u64,
    pub log_sampling_rate: usize,
    pub keycodes_file: String,
    pub seed: Option<u64>,
    pub threads: usize,
}

#[derive(Debug)]
pub struct OptimizationRunner;

impl OptimizationRunner {
    pub async fn prepare_session<L: AssetLoader>(
        loader: &L,
        config: &JobConfig,
        options: &RunnerOptions,
    ) -> anyhow::Result<ScoringSession> {
        let builder = SessionBuilder::new(loader)
            .with_keyboard_def(Arc::new(config.definition.clone()))
            .with_corpus(&config.corpora).await.map_err(|e| anyhow::anyhow!(e))?
            .with_cost_matrix(&config.cost_matrix).await.map_err(|e| anyhow::anyhow!(e))?
            .with_keycodes(&options.keycodes_file).await.map_err(|e| anyhow::anyhow!(e))?
            .with_rubric(keyforge_adapter::conversion::to_domain_rubric(&config.weights))
            .with_biometrics(config.biometrics.clone())
            .with_config(keyforge_model::SearchConfig::Annealing {
                steps: config.params.get_search_steps(),
                start_temp: config.params.get_temp_max(),
                end_temp: config.params.get_temp_min(),
                seed: options.seed.unwrap_or(config.params.seed.unwrap_or(42)),
                patience: config.params.get_search_patience(),
                reheats: config.params.get_reheats(),
                reheat_factor: config.params.get_reheat_factor(),
                include_thumbs: config.params.include_thumbs,
            });
            
        let session = builder.build().map_err(|e| anyhow::anyhow!(e))?;
        Ok(session)
    }

    pub async fn run<CB: ProgressCallback + 'static>(
        session: ScoringSession,
        _job_id: String,
        _stop_flag: Arc<AtomicBool>,
        callback: CB,
        _options: RunnerOptions,
        config: &JobConfig,
    ) -> Result<OptimizationResult, EvolutionError> {
        // Handle pinned keys conversion
        let pinned: Vec<Option<KeyCode>> = if config.pinned_keys.is_empty() {
            vec![]
        } else {
            let mut p = vec![None; session.engine.key_count()];
            for c in &config.pinned_keys {
                if (c.index.0 as usize) < p.len() {
                    // Resolve key label to code using registry
                    if let Some(code) = session.registry.get_code(&c.key) {
                        p[c.index.0 as usize] = Some(code);
                    }
                }
            }
            p
        };

        // Run optimization in blocking thread
        let engine = session.engine.clone();
        let search_config = session.search_config.clone();
        
        tokio::task::spawn_blocking(move || {
            keyforge_core::optimize_with_engine(
                engine,
                &search_config,
                callback,
                None, // Initial layout (random)
                Some(&pinned),
            )
        }).await.map_err(|e| EvolutionError::Config(format!("Task join error: {}", e)))?
    }
}

#[derive(Debug)]
pub struct Runner<'a, L: AssetLoader> {
    loader: &'a L,
}

impl<'a, L: AssetLoader> Runner<'a, L> {
    pub fn new(loader: &'a L) -> Self {
        Self { loader }
    }

    pub async fn prepare_job(&self, config: &JobConfig) -> anyhow::Result<Runtime> {
        let builder = SessionBuilder::new(self.loader)
            .with_keyboard_def(Arc::new(config.definition.clone()))
            .with_corpus(&config.corpora).await.map_err(|e| anyhow::anyhow!(e))?
            .with_cost_matrix(&config.cost_matrix).await.map_err(|e| anyhow::anyhow!(e))?
            .with_biometrics(config.biometrics.clone())
            .with_keycodes(ASSET_KEYCODES_FILENAME).await.map_err(|e| anyhow::anyhow!(e))?;
            
        let session = builder.build().map_err(|e| anyhow::anyhow!(e))?;
        Ok(Runtime::from(session))
    }
}