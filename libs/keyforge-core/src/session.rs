// libs/keyforge-core/src/session.rs

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

use keyforge_model::keycodes::KeycodeRegistry;
use keyforge_model::SearchConfig;
use keyforge_physics::ScoringEngine;
use std::sync::Arc;

/// A consolidated environment for scoring and optimization.
/// Holds the compiled physics engine and associated metadata.
#[derive(Clone, Debug)]
pub struct ScoringSession {
    /// The pre-compiled evaluation engine.
    pub engine: Arc<dyn ScoringEngine>,
    /// The registry used for resolving key labels.
    pub registry: Arc<KeycodeRegistry>,
    /// The search parameters used for this session.
    pub search_config: SearchConfig,
}

impl ScoringSession {
    /// Creates a new `ScoringSession` from the provided engine, registry, and config.
    #[must_use]
    pub fn new(
        engine: Arc<dyn ScoringEngine>,
        registry: Arc<KeycodeRegistry>,
        search_config: SearchConfig,
    ) -> Self {
        Self {
            engine,
            registry,
            search_config,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use keyforge_model::{Keyboard, Corpus, Rubric, CostModel};
    use keyforge_physics::EngineFactory;

    #[test]
    fn test_scoring_session_creation() {
        let kb = Keyboard::new(vec![keyforge_model::KeyNode::default()], 0, "test".into()).unwrap();
        let corpus = Corpus::default();
        let rubric = Rubric::default();
        let mut cm = CostModel::default();
        let mut fingers = std::collections::HashMap::new();
        fingers.insert("index".to_string(), keyforge_model::cost_model::FingerDefinition::Standard(
            std::collections::HashMap::from([("base".to_string(), std::collections::HashMap::from([("r0".to_string(), 1.0)]))])
        ));
        cm.models.insert("model_a_row_staggered".into(), keyforge_model::cost_model::ModelDefinition {
            description: "test".into(),
            static_costs: std::collections::HashMap::from([("universal_hand".to_string(), keyforge_model::cost_model::HandDefinition { fingers })]),
        });
        
        let engine = EngineFactory::new_exact(&kb, &corpus, &rubric, &cm).unwrap();
        let registry = Arc::new(KeycodeRegistry::new_with_defaults());
        let config = SearchConfig::Annealing {
            steps: 100,
            start_temp: 10.0,
            end_temp: 0.1,
            seed: 42,
            patience: 10,
            reheats: 0,
            reheat_factor: 0.5,
            include_thumbs: false,
        };
        
        let session = ScoringSession::new(Arc::from(engine), registry, config);
        assert_eq!(session.registry.definitions.len(), 2);
    }
}
