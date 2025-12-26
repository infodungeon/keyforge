use crate::error::{PersistenceError, PersistenceResult};
use crate::project::Project;
use keyforge_adapter::conversion;
use keyforge_compute::Runtime;
use keyforge_core::ScoringEngine;
use keyforge_model::loader::AssetLoader;
use keyforge_protocol::{keycodes::KeycodeRegistry, CostMatrixSource};
use std::sync::Arc;

pub struct Compiler<'a> {
    loader: &'a dyn AssetLoader,
}

impl<'a> Compiler<'a> {
    pub fn new(loader: &'a dyn AssetLoader) -> Self {
        Self { loader }
    }

    pub fn compile(&self, project: &Project) -> PersistenceResult<Runtime> {
        let kb_def = self
            .loader
            .load_keyboard(&project.keyboard)
            .map_err(PersistenceError::Loader)?;

        let corpus = self
            .loader
            .load_corpus(&project.corpora)
            .map_err(PersistenceError::Loader)?;

        let raw_cost = match &project.cost_matrix {
            CostMatrixSource::Predefined(name) => self
                .loader
                .load_cost_matrix(name)
                .map_err(PersistenceError::Loader)?,
            CostMatrixSource::Custom(_) => {
                return Err(PersistenceError::Config(
                    "Custom CSV compilation not yet implemented".into(),
                ));
            }
        };

        let registry = self
            .loader
            .load_keycodes("keycodes.json")
            .map_err(PersistenceError::Loader)
            .unwrap_or_else(|_| KeycodeRegistry::new_with_defaults());

        let domain_kb = conversion::to_domain_keyboard(&kb_def.geometry);
        let domain_rubric = conversion::to_domain_rubric(&project.weights);
        let domain_config =
            conversion::to_domain_config(&project.params, project.seed.unwrap_or(42));

        let overrides = conversion::resolve_cost_matrix(&raw_cost.entries, &kb_def.geometry);

        let engine = ScoringEngine::new(&domain_kb, &corpus, &domain_rubric, &overrides);

        Ok(Runtime {
            engine: Arc::new(engine),
            registry: Arc::new(registry),
            search_config: domain_config,
        })
    }
}
