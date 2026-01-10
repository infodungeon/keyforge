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

use keyforge_model::SearchConfig;
use keyforge_model::keycodes::KeycodeRegistry;
use keyforge_physics::ScoringEngine;
use std::sync::Arc;

/// A consolidated environment for scoring and optimization.
/// Holds the compiled physics engine and associated metadata.
#[derive(Clone, Debug)]
pub struct ScoringSession {
    /// The pre-compiled evaluation engine.
    pub engine: Arc<ScoringEngine>,
    /// The registry used for resolving key labels.
    pub registry: Arc<KeycodeRegistry>,
    /// The search parameters used for this session.
    pub search_config: SearchConfig,
}

impl ScoringSession {
    /// Creates a new `ScoringSession` from the provided engine, registry, and config.
    pub fn new(
        engine: Arc<ScoringEngine>,
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
