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
use keyforge_core::{EvolutionError, ProgressCallback, ScoringSession};
use keyforge_model::constants::ASSET_KEYCODES_FILENAME;
use keyforge_model::{KeyCode, OptimizationResult};
use keyforge_protocol::JobConfig;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

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
    /// Prepares a scoring session with all required assets.
    ///
    /// # Errors
    ///
    /// Returns `anyhow::Error` if any assets (corpus, cost matrix, keycodes) fail to load.
    pub async fn prepare_session<L: AssetLoader>(
        loader: &L,
        config: &JobConfig,
        options: &RunnerOptions,
    ) -> anyhow::Result<ScoringSession> {
        let builder = SessionBuilder::new(loader)
            .with_keyboard_def(Arc::new(config.definition.clone()))
            .with_corpus(&config.corpora)
            .await
            .map_err(|e| anyhow::anyhow!(e))?
            .with_cost_matrix(&config.cost_matrix)
            .await
            .map_err(|e| anyhow::anyhow!(e))?
            .with_keycodes(&options.keycodes_file)
            .await
            .map_err(|e| anyhow::anyhow!(e))?
            .with_rubric(keyforge_adapter::conversion::to_domain_rubric(
                &config.weights,
            ))
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

    /// Runs the optimization process.
    ///
    /// # Errors
    ///
    /// Returns `EvolutionError` if optimization fails.
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
                &engine,
                &search_config,
                callback,
                None, // initial_layout
                Some(pinned.as_slice()),
            )
        })
        .await
        .map_err(|e| EvolutionError::Config(format!("Task join error: {e}")))?
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

    /// Prepares a job runtime.
    ///
    /// # Errors
    ///
    /// Returns `anyhow::Error` if any assets fail to load.
    pub async fn prepare_job(&self, config: &JobConfig) -> anyhow::Result<Runtime> {
        let builder = SessionBuilder::new(self.loader)
            .with_keyboard_def(Arc::new(config.definition.clone()))
            .with_corpus(&config.corpora)
            .await
            .map_err(|e| anyhow::anyhow!(e))?
            .with_cost_matrix(&config.cost_matrix)
            .await
            .map_err(|e| anyhow::anyhow!(e))?
            .with_biometrics(config.biometrics.clone())
            .with_keycodes(ASSET_KEYCODES_FILENAME)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;

        let session = builder.build().map_err(|e| anyhow::anyhow!(e))?;
        Ok(Runtime::from(session))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use keyforge_model::Asset;
    use std::any::Any;
    use keyforge_model::geometry::{KeyboardDefinition, KeyboardGeometry};
    use keyforge_model::keycodes::KeycodeRegistry;
    use keyforge_model::{Corpus, CostModel};
    use keyforge_core::loader::LoaderResult;

    #[derive(Debug)]
    struct MockLoader;
    #[async_trait::async_trait]
    impl AssetLoader for MockLoader {
        async fn load<T: Asset>(&self, _id: &str) -> LoaderResult<Arc<T>> {
            let mut kb = KeyboardDefinition::default();
            kb.geometry.keys.push(keyforge_model::KeyNode::default());
            kb.geometry.prime_slots.push(keyforge_model::types::KeyIndex(0));
            
            let any_kb = Arc::new(kb) as Arc<dyn Any + Send + Sync>;
            if let Ok(arc) = any_kb.downcast::<T>() { return Ok(arc); }

            let json = r#"{
                "meta": { "version": "2.0", "description": "T", "unit": "pts" },
                "models": { "model_a_row_staggered": { "description": "t", "static_costs": {} } },
                "dynamic_rules": { "sequence_modifiers": {}, "penalties": {}, "constraints": {} }
            }"#;
            let model: CostModel = serde_json::from_str(json).unwrap();
            let any_model = Arc::new(model) as Arc<dyn Any + Send + Sync>;
            if let Ok(arc) = any_model.downcast::<T>() { return Ok(arc); }

            let any_kc = Arc::new(KeycodeRegistry::default()) as Arc<dyn Any + Send + Sync>;
            if let Ok(arc) = any_kc.downcast::<T>() { return Ok(arc); }

            Err(keyforge_model::error::ForgeError::NotFound(_id.to_string()))
        }
        async fn load_corpus(&self, _sources: &[keyforge_model::config::CorpusSource]) -> LoaderResult<Arc<Corpus>> {
            Ok(Arc::new(Corpus::default()))
        }
    }

    #[tokio::test]
    async fn test_runner_lifecycle() {
        struct NoOpCallback;
        impl keyforge_core::ProgressCallback for NoOpCallback {
            fn on_progress(&self, _epoch: usize, _score: f32, _layout: &[KeyCode], _ips: f32) -> bool { true }
        }

        let loader = MockLoader;
        let mut config = JobConfig::default();
        config.definition.geometry.keys.push(keyforge_model::KeyNode::default());
        config.definition.geometry.prime_slots.push(keyforge_model::types::KeyIndex(0));
        
        let options = RunnerOptions { keycodes_file: "kc".into(), ..Default::default() };
        
        let session = OptimizationRunner::prepare_session(&loader, &config, &options).await.unwrap();
        assert_eq!(session.engine.key_count(), 1);

        let stop = Arc::new(AtomicBool::new(false));
        let res = OptimizationRunner::run(session, "job".into(), stop, NoOpCallback, options, &config).await.unwrap();
        assert!(res.score >= 0.0);
    }

    #[tokio::test]
    async fn test_runner_prepare_job() {
        let loader = MockLoader;
        let runner = Runner::new(&loader);
        let mut config = JobConfig::default();
        config.definition.geometry.keys.push(keyforge_model::KeyNode::default());
        config.definition.geometry.prime_slots.push(keyforge_model::types::KeyIndex(0));
        
        let rt = runner.prepare_job(&config).await.unwrap();
        assert_eq!(rt.engine.key_count(), 1);
    }

    #[tokio::test]
    async fn test_runner_pinned_keys() {
        struct NoOpCallback;
        impl keyforge_core::ProgressCallback for NoOpCallback {
            fn on_progress(&self, _epoch: usize, _score: f32, _layout: &[KeyCode], _ips: f32) -> bool { true }
        }

        let _loader = MockLoader;
        let mut config = JobConfig::default();
        config.pinned_keys.push(keyforge_model::KeyConstraint {
            index: keyforge_model::KeyIndex(0),
            key: "SPACE".to_string(),
        });
        
        let registry = KeycodeRegistry::new(vec![
            keyforge_model::keycodes::KeycodeDefinition {
                code: KeyCode(0),
                id: "SPACE".into(),
                label: " ".into(),
                aliases: vec![],
            }
        ]);
        
        let mut cm = CostModel::default();
        cm.models.insert("model_a_row_staggered".into(), keyforge_model::cost_model::ModelDefinition {
            description: "test".into(),
            static_costs: std::collections::HashMap::new(),
        });

        let session = ScoringSession {
            engine: keyforge_physics::EngineFactory::new_exact(&keyforge_model::Keyboard::new(vec![keyforge_model::KeyNode::default()], 0).unwrap(), &Corpus::default(), &keyforge_model::Rubric::default(), &cm).unwrap().into(),
            registry: Arc::new(registry),
            search_config: keyforge_model::SearchConfig::default(),
        };

        let stop = Arc::new(AtomicBool::new(false));
        let options = RunnerOptions::default();
        let res = OptimizationRunner::run(session, "job".into(), stop, NoOpCallback, options, &config).await.unwrap();
        assert!(res.score >= 0.0);
    }
}
