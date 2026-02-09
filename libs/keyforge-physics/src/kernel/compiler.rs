// libs/keyforge-physics/src/kernel/compiler.rs

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

use super::EngineContext;
use crate::error::PhysicsError;
use keyforge_model::{Corpus, CostModel, Keyboard, Rubric};
use std::sync::Arc;
use tracing::{info, instrument};

use super::stages;
use stages::corpus::CorpusStage;
use stages::costs::CostStage;
use stages::geometry::GeometryStage;
use stages::CompilationStage;

#[derive(Debug)]
pub(crate) struct Compiler;

use std::collections::HashMap;

impl Compiler {
    /// Compiles the scoring engine.
    ///
    /// # Errors
    /// Returns `PhysicsError` if:
    /// - Geometry compilation fails.
    /// - Cost model processing fails.
    /// - Corpus processing fails.
    /// - Sequence modifiers contain invalid values.
    /// - Configuration values (like redirect penalty) are invalid.
    /// - The final context verification fails.
    #[instrument(skip_all)]
    pub fn compile(
        kb: &Keyboard,
        corpus: &Corpus,
        rubric: &Rubric,
        cost_model: &CostModel,
    ) -> Result<EngineContext, PhysicsError> {
        let key_count = kb.count();
        info!(key_count = key_count, "Compiling scoring engine...");

        // Stage 1: Geometry
        let geo_stage = GeometryStage { rubric };
        let geo_out = geo_stage.execute(kb)?;

        // Stage 2: Costs
        // Use preferred model if found, otherwise pick first available.
        let model_key = cost_model.preferred_model_key();

        let cost_stage = CostStage {
            kb,
            rubric,
            cost_model,
            model_key,
        };
        let cost_out = cost_stage.execute(())?;

        // Stage 3: Corpus
        let corpus_stage = CorpusStage { corpus, rubric };
        let corpus_out = corpus_stage.execute(())?;

        let mut sequence_modifiers = HashMap::new();
        for (bigram, &val) in &cost_model.dynamic_rules().sequence_modifiers {
            if bigram.len() == 2 {
                let bytes = bigram.as_bytes();
                let key = (u16::from(bytes[0]), u16::from(bytes[1]));
                sequence_modifiers.insert(
                    key,
                    val,
                );
            }
        }

        info!("Engine compilation complete.");

        let ctx = EngineContext {
            key_count,
            geometry: super::GeometryData {
                hands: geo_out.hands.into(),
                fingers: geo_out.fingers.into(),
                rows: geo_out.rows.into(),
                cols: geo_out.cols.into(),
                cost_matrix: cost_out.cost_matrix.into(),
                dist_matrix: geo_out.dist_matrix.into(),
                key_home_distances: geo_out.key_home_distances.into(),
                key_costs: cost_out.key_costs.into(),
            },
            corpus: super::CorpusData {
                char_freqs: corpus_out.char_freqs,
                bigram_starts: corpus_out.bigram_starts.into(),
                bigram_others: corpus_out.bigram_others.into(),
                bigram_freqs: corpus_out.bigram_freqs.into(),
                bigram_rev_starts: corpus_out.bigram_rev_starts.into(),
                bigram_rev_others: corpus_out.bigram_rev_others.into(),
                bigram_rev_freqs: corpus_out.bigram_rev_freqs.into(),
                trigram_starts: corpus_out.trigram_starts.into(),
                trigram_others1: corpus_out.trigram_others1.into(),
                trigram_others2: corpus_out.trigram_others2.into(),
                trigram_freqs: corpus_out.trigram_freqs.into(),
            },
            all_bigrams: corpus.bigrams.clone(),
            all_trigrams: corpus.trigrams.clone(),
            penalty_redirect: rubric.redirect(),
            bonus_roll: rubric.roll_bonus(),
            bonus_roll_out: rubric.roll_out_bonus(),
            sequence_modifiers: Arc::new(sequence_modifiers),
        };

        ctx.verify()?;
        Ok(ctx)
    }
}
