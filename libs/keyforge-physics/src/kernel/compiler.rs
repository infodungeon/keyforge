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
pub struct Compiler;

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
        let geo_out = geo_stage.execute(Arc::new(kb.clone()))?;

        // Stage 2: Costs
        // Task-phys-rev-025: Derive model key from KB notes or metadata
        let model_key = if kb.kb_type.to_lowercase().contains("ortho") {
            Some("model_ortho")
        } else {
            Some("model_a_row_staggered")
        };

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
        for (bigram, &val) in &cost_model.dynamic_rules.sequence_modifiers {
            if bigram.len() == 2 {
                let bytes = bigram.as_bytes();
                let key = (u16::from(bytes[0]), u16::from(bytes[1]));
                sequence_modifiers.insert(
                    key,
                    Score::from_f32(val).map_err(|e| PhysicsError::InvalidInput { message: e })?,
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
                char_freqs: corpus_out.char_freqs.into(),
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
                trigram_mid_starts: corpus_out.trigram_mid_starts.into(),
                trigram_mid_others1: corpus_out.trigram_mid_others1.into(),
                trigram_mid_others2: corpus_out.trigram_mid_others2.into(),
                trigram_mid_freqs: corpus_out.trigram_mid_freqs.into(),
            },
            all_bigrams: corpus.bigrams.clone().into(),
            all_trigrams: corpus.trigrams.clone().into(),
            penalty_redirect: Score::from_f32(rubric.redirect)
                .map_err(|e| PhysicsError::InvalidInput { message: e })?,
            bonus_roll: Score::from_f32(rubric.roll_bonus)
                .map_err(|e| PhysicsError::InvalidInput { message: e })?,
            bonus_roll_out: Score::from_f32(rubric.roll_out_bonus)
                .map_err(|e| PhysicsError::InvalidInput { message: e })?,
            sequence_modifiers: Arc::new(sequence_modifiers),
        };

        ctx.verify()?;
        Ok(ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use keyforge_model::{
        types::{FingerIndex, HandIndex, RowIndex},
        KeyNode,
    };

    fn setup_test_cost_model() -> CostModel {
        let mut cm = CostModel::default();
        let mut fingers = std::collections::HashMap::new();
        let mut base_r0 = std::collections::HashMap::new();
        base_r0.insert("r0".to_string(), 1.0);
        let mut zones = std::collections::HashMap::new();
        zones.insert("base".to_string(), base_r0);
        fingers.insert(
            "index".to_string(),
            keyforge_model::cost_model::FingerDefinition::Standard(zones),
        );

        cm.models.insert(
            "model_a_row_staggered".into(),
            keyforge_model::cost_model::ModelDefinition {
                description: "test".into(),
                static_costs: std::collections::HashMap::from([(
                    "universal_hand".to_string(),
                    keyforge_model::cost_model::HandDefinition { fingers },
                )]),
            },
        );
        cm
    }

    #[test]
    fn test_compiler_empty_corpus() {
        let keys = vec![KeyNode {
            index: 0,
            hand: HandIndex(0),
            finger: FingerIndex::INDEX,
            row: RowIndex(0),
            ..Default::default()
        }];
        let kb = Keyboard::new(keys, 0, "test".into()).unwrap();
        let corpus = Corpus::default();
        let rubric = Rubric::default();
        let cost_model = setup_test_cost_model();

        let res = Compiler::compile(&kb, &corpus, &rubric, &cost_model);
        assert!(res.is_ok());
        let ctx = res.unwrap();
        assert_eq!(ctx.key_count, 1);
        assert!(ctx.corpus.char_freqs.iter().all(|&f| f == 0));
    }

    #[test]
    fn test_compiler_missing_cost_model() {
        let kb = Keyboard::new(
            vec![KeyNode {
                finger: FingerIndex::INDEX,
                ..Default::default()
            }],
            0,
            "test".into(),
        )
        .unwrap();
        let corpus = Corpus::default();
        let cost_model = CostModel::default();

        let res = Compiler::compile(&kb, &corpus, &Rubric::default(), &cost_model);
        assert!(res.is_err());
    }

    #[test]
    fn test_compiler_invalid_score_values() {
        let keys = vec![KeyNode {
            finger: FingerIndex::INDEX,
            ..Default::default()
        }];
        let kb = Keyboard::new(keys, 0, "test".into()).unwrap();
        let corpus = Corpus::default();
        let mut rubric = Rubric::default();
        rubric.redirect = f32::NAN; // Trigger error

        let cost_model = setup_test_cost_model();

        let res = Compiler::compile(&kb, &corpus, &rubric, &cost_model);
        assert!(res.is_err());
        match res.err().unwrap() {
            PhysicsError::CalculationError(_) | PhysicsError::InvalidInput { .. } => {}
            e => panic!("Wrong error type: {e:?}"),
        }
    }

    #[test]
    fn test_compiler_invalid_sequence_modifier() {
        let keys = vec![KeyNode {
            finger: FingerIndex::INDEX,
            ..Default::default()
        }];
        let kb = Keyboard::new(keys, 0, "test".into()).unwrap();
        let corpus = Corpus::default();
        let rubric = Rubric::default();

        let mut cost_model = setup_test_cost_model();
        cost_model
            .dynamic_rules
            .sequence_modifiers
            .insert("ab".into(), f32::NAN);

        let res = Compiler::compile(&kb, &corpus, &rubric, &cost_model);
        assert!(res.is_err());
        match res.err().unwrap() {
            PhysicsError::CalculationError(_) | PhysicsError::InvalidInput { .. } => {}
            e => panic!("Wrong error type: {e:?}"),
        }
    }
}
