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

use super::types::Score;
use super::EngineContext;
use keyforge_model::{Corpus, Keyboard, Rubric, CostModel};
use crate::errors::PhysicsError;
use std::sync::Arc;
use tracing::{info, instrument};

use super::stages;
use stages::CompilationStage;
use stages::geometry::GeometryStage;
use stages::costs::CostStage;
use stages::corpus::CorpusStage;

pub struct Compiler;

use std::collections::HashMap;

impl Compiler {
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
        let geo_stage = GeometryStage;
        let geo_out = geo_stage.execute(Arc::new(kb.clone()))?;

        // Stage 2: Costs
        let cost_stage = CostStage { kb, rubric, cost_model };
        let cost_out = cost_stage.execute(())?;

        // Stage 3: Corpus
        let corpus_stage = CorpusStage { corpus, rubric };
        let corpus_out = corpus_stage.execute(())?;

        // Stage 4: Key Pre-computation (New for task-phys-011)
        let mut unique_keys_set = std::collections::HashSet::new();
        
        // Collect from monograms
        for (i, &freq) in corpus_out.char_freqs.iter().enumerate() {
            #[allow(clippy::cast_possible_truncation)]
            if freq > 0 { unique_keys_set.insert(i as u16); }
        }
        
        // Collect from Bigrams
        for &(c1, c2, _) in &corpus.bigrams {
            unique_keys_set.insert(c1);
            unique_keys_set.insert(c2);
        }

        // Collect from Trigrams
        for &(c1, c2, c3, _) in &corpus.trigrams {
            unique_keys_set.insert(c1);
            unique_keys_set.insert(c2);
            unique_keys_set.insert(c3);
        }

        let mut sorted_unique_keys: Vec<u16> = unique_keys_set.into_iter().collect();
        sorted_unique_keys.sort_unstable();
        
        let mut key_rank_map = HashMap::with_capacity(sorted_unique_keys.len());
        for (rank, &key) in sorted_unique_keys.iter().enumerate() {
            key_rank_map.insert(key, rank);
        }

        let mut sequence_modifiers = HashMap::new();
        for (bigram, &val) in &cost_model.dynamic_rules.sequence_modifiers {
            if bigram.len() == 2 {
                let bytes = bigram.as_bytes();
                let key = (u16::from(bytes[0]), u16::from(bytes[1]));
                sequence_modifiers.insert(key, Score::from_f32(val));
            }
        }

        info!("Engine compilation complete.");

        Ok(EngineContext {
            key_count,
            hands: geo_out.hands,
            fingers: geo_out.fingers,
            rows: geo_out.rows,
            cols: geo_out.cols,
            cost_matrix: cost_out.cost_matrix,
            dist_matrix: geo_out.dist_matrix,
            key_home_distances: geo_out.key_home_distances,
            key_costs: cost_out.key_costs,
            char_freqs: corpus_out.char_freqs,
            bigram_starts: corpus_out.bigram_starts,
            bigram_others: corpus_out.bigram_others,
            bigram_freqs: corpus_out.bigram_freqs,
            bigram_rev_starts: corpus_out.bigram_rev_starts,
            bigram_rev_others: corpus_out.bigram_rev_others,
            bigram_rev_freqs: corpus_out.bigram_rev_freqs,
            trigram_starts: corpus_out.trigram_starts,
            trigram_others1: corpus_out.trigram_others1,
            trigram_others2: corpus_out.trigram_others2,
            trigram_freqs: corpus_out.trigram_freqs,
            trigram_mid_starts: corpus_out.trigram_mid_starts,
            trigram_mid_others1: corpus_out.trigram_mid_others1,
            trigram_mid_others2: corpus_out.trigram_mid_others2,
            trigram_mid_freqs: corpus_out.trigram_mid_freqs,
            trigram_end_starts: corpus_out.trigram_end_starts,
            trigram_end_others1: corpus_out.trigram_end_others1,
            trigram_end_others2: corpus_out.trigram_end_others2,
            trigram_end_freqs: corpus_out.trigram_end_freqs,
            all_bigrams: corpus.bigrams.clone(),
            all_trigrams: corpus.trigrams.clone(),
            penalty_redirect: Score::from_f32(rubric.redirect),
            penalty_skip: Score::ZERO,
            bonus_roll: Score::from_f32(rubric.roll_bonus),
            sequence_modifiers,
            sorted_unique_keys,
            key_rank_map,
        })
    }
}
