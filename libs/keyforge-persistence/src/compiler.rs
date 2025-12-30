// Copyright (c) 2025 KeyForge Contributors
//
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
use crate::error::PersistenceResult;
use crate::project::Project;
use keyforge_compute::{Runtime, SessionBuilder};
use keyforge_core::loader::AssetLoader;

/// Orchestrates the compilation of a [Project] into a [Runtime].
/// This involves loading keyboards, corpora, and cost matrices via the [AssetLoader].
/// Refactored to use [SessionBuilder] for core compilation logic.
pub struct Compiler<'a> {
    builder: SessionBuilder<'a>,
}

impl<'a> Compiler<'a> {
    /// Creates a new compiler with the given asset loader.
    pub fn new(loader: &'a dyn AssetLoader) -> Self {
        Self {
            builder: SessionBuilder::new(loader),
        }
    }

    /// Compiles the project into a ready-to-use search runtime.
    pub async fn compile(&self, project: &Project) -> PersistenceResult<Runtime> {
        let session = self.builder.build(
            &project.keyboard,
            &project.corpora,
            &project.weights,
            &project.params,
            "keycodes.json",
            &project.cost_matrix,
            project.seed,
        ).await.map_err(crate::error::PersistenceError::Loader)?;

        Ok(Runtime::from(session))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use keyforge_model::error::ForgeError;
    use keyforge_core::loader::{AssetLoader, LoaderResult, RawCostData};
    use keyforge_model::Corpus;
    use keyforge_protocol::config::CorpusSource;
    use keyforge_protocol::geometry::KeyboardDefinition;
    use keyforge_protocol::keycodes::KeycodeRegistry;

    struct FailingLoader;
    #[async_trait::async_trait]
    impl AssetLoader for FailingLoader {
        async fn load_keyboard(&self, _name: &str) -> LoaderResult<KeyboardDefinition> {
            Err(ForgeError::NotFound("kb".into()))
        }
        async fn load_corpus(&self, _sources: &[CorpusSource]) -> LoaderResult<Corpus> {
            Err(ForgeError::NotFound("corpus".into()))
        }
        async fn load_cost_matrix(&self, _filename: &str) -> LoaderResult<RawCostData> {
            Err(ForgeError::NotFound("cost".into()))
        }
        async fn load_keycodes(&self, _filename: &str) -> LoaderResult<KeycodeRegistry> {
            Ok(KeycodeRegistry::new_with_defaults())
        }
    }

    #[tokio::test]
    async fn test_compile_keyboard_fail() {
        let loader = FailingLoader;
        let compiler = Compiler::new(&loader);
        let project = Project::default();
        let result = compiler.compile(&project).await;
        assert!(matches!(result, Err(crate::error::PersistenceError::Loader(_))));
    }
}
