// libs/keyforge-protocol/src/ports.rs

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

//! Hexagonal Port definitions for orchestration and strategy pluggability.

use async_trait::async_trait;
use keyforge_model::error::ForgeError;
use keyforge_model::{AnalysisReport, KeyCode, Layout, OptimizationResult, Score};

/// Outbound Port for layout scoring.
/// 
/// Implementing adapters (e.g. keyforge-physics) must provide bit-perfect 
/// deterministic scoring based on the physical model.
pub trait ScoringEngine: Send + Sync + std::fmt::Debug {
    /// Returns the name of the engine.
    fn name(&self) -> &'static str;

    /// Returns the number of keys supported.
    fn key_count(&self) -> usize;

    /// Calculates the deterministic score for a layout.
    fn score(&self, layout: &Layout) -> Result<Score, ForgeError>;

    /// Calculates detailed score components (monogram, bigram, trigram).
    fn score_detailed(&self, layout: &Layout) -> Result<(i64, i64, i64), ForgeError>;

    /// Generates a detailed ergonomic report.
    fn analyze(&self, layout: &Layout) -> Result<AnalysisReport, ForgeError>;
}

/// Inbound Port for layout optimization.
/// 
/// Implementing adapters (e.g. keyforge-evolution) must provide iterative 
/// improvement of a layout toward a minimum score.
#[async_trait]
pub trait LayoutOptimizer: Send + Sync + std::fmt::Debug {
    /// Executes the optimization loop.
    async fn optimize(
        &self,
        engine: &dyn ScoringEngine,
        initial_layout: Layout,
        pinned_keys: &[Option<KeyCode>],
    ) -> Result<OptimizationResult, ForgeError>;
}
