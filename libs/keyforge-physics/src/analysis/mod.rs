// libs/keyforge-physics/src/analysis/mod.rs

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

pub mod fingerprint;
pub mod heuristics;

#[keyforge_testing_macros::kf_test]
mod tests {
    use crate::{EngineCompilationContext, EngineFactory};
    use keyforge_model::{
        types::{ColIndex, FingerIndex, HandIndex, KeyCode, RowIndex},
        Corpus, CostModel, KeyNode, Keyboard, Layout, Rubric,
    };

    fn setup_kb(size: usize) -> Keyboard {
        let keys: Vec<KeyNode> = (0..size)
            .map(|i| KeyNode {
                index: i,
                label: format!("k{i}"),
                hand: HandIndex((i % 2) as u8),
                finger: FingerIndex::new_unchecked((i % 5) as u8),
                row: RowIndex((i / 10) as i8),
                col: ColIndex((i % 10) as i8),
                x: (i % 10) as f32,
                y: (i / 10) as f32,
                is_home: false,
                ..Default::default()
            })
            .collect();
        Keyboard::new(keys, 1, "test".into()).unwrap()
    }

    fn mock_cost_model() -> CostModel {
        let json = r#"{
            "meta": { "version": "2.0", "description": "Test", "unit": "pts" },
            "models": {
                "model_a_row_staggered": {
                    "description": "Test Model",
                    "static_costs": {
                        "universal_hand": {
                            "thumb": { "pos_1": 100.0 },
                            "index": { "base": { "r0": 100.0, "r1": 100.0, "r2": 100.0 } },
                            "middle": { "base": { "r0": 100.0, "r1": 100.0, "r2": 100.0 } },
                            "ring": { "base": { "r0": 100.0, "r1": 100.0, "r2": 100.0 } },
                            "pinky": { "base": { "r0": 100.0, "r1": 100.0, "r2": 100.0 } }
                        }
                    }
                }
            },
            "dynamic_rules": { "sequence_modifiers": {}, "penalties": {}, "constraints": {} }
        }"#;
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn test_metric_detection_sfb_scissors() {
        let keys = vec![
            KeyNode {
                index: 0,
                hand: HandIndex(0),
                finger: FingerIndex::new_unchecked(1),
                row: RowIndex(0),
                ..Default::default()
            },
            KeyNode {
                index: 1,
                hand: HandIndex(0),
                finger: FingerIndex::new_unchecked(1),
                row: RowIndex(0),
                ..Default::default()
            },
            KeyNode {
                index: 2,
                hand: HandIndex(0),
                finger: FingerIndex::new_unchecked(2),
                row: RowIndex(2),
                ..Default::default()
            },
        ];
        let kb_manual = Keyboard::new(keys, 1, "test".into()).unwrap();

        let mut corpus_manual = Corpus::default();
        corpus_manual.bigrams.push((0, 1, 100)); // SFB
        corpus_manual.bigrams.push((0, 2, 100)); // Scissor

        let cost_model = mock_cost_model();
        let engine = EngineFactory::new_generic(EngineCompilationContext {
            keyboard: &kb_manual,
            corpus: &corpus_manual,
            rubric: &Rubric::default(),
            cost_model: &cost_model,
        })
        .unwrap();
        let layout = Layout::new_unchecked(vec![KeyCode(0), KeyCode(1), KeyCode(2)]);

        let report = engine.analyze(&layout).unwrap();

        assert!(report.sfb_total > 0.0, "Should detect SFBs");
        assert!(report.scissors > 0.0, "Should detect Scissors");
        assert!(!report.top_sfbs.is_empty());
        assert!(!report.top_scissors.is_empty());
    }

    #[test]
    fn test_metric_detection_rolls_redirects() {
        let keys: Vec<KeyNode> = (0..5)
            .map(|i| KeyNode {
                index: i,
                hand: HandIndex(0),
                finger: FingerIndex::new_unchecked(i as u8),
                ..Default::default()
            })
            .collect();
        let kb = Keyboard::new(keys, 0, "test".into()).unwrap();

        let layout = Layout::new_unchecked(vec![
            KeyCode(97),
            KeyCode(98),
            KeyCode(99),
            KeyCode(100),
            KeyCode(101),
        ]);
        let mut corpus = Corpus::default();

        corpus.trigrams.push((101, 100, 99, 100));
        corpus.trigrams.push((99, 100, 99, 100));

        let rubric = Rubric {
            roll_bonus: 10.0,
            redirect: 50.0,
            trigram_coverage: 1.0,
            trigram_limit: 100,
            ..Rubric::default()
        };

        let cost_model = mock_cost_model();
        let engine = EngineFactory::new_generic(EngineCompilationContext {
            keyboard: &kb,
            corpus: &corpus,
            rubric: &rubric,
            cost_model: &cost_model,
        })
        .unwrap();
        let report = engine.analyze(&layout).unwrap();

        assert!(report.rolls > 0.0, "Expected rolls");
        assert!(report.redirects > 0.0, "Expected redirects");
    }

    #[test]
    fn test_heatmap_and_penalty_map() {
        let kb = setup_kb(5);
        let mut corpus = Corpus::default();

        corpus.char_freqs[97] = 1000;
        corpus.char_freqs[98] = 1000;
        corpus.bigrams.push((97, 98, 500));

        let cost_model = mock_cost_model();
        let engine = EngineFactory::new_generic(EngineCompilationContext {
            keyboard: &kb,
            corpus: &corpus,
            rubric: &Rubric::default(),
            cost_model: &cost_model,
        })
        .unwrap();
        let layout = Layout::new_unchecked(vec![
            KeyCode(97),
            KeyCode(98),
            KeyCode(99),
            KeyCode(100),
            KeyCode(101),
        ]);

        let report = engine.analyze(&layout).unwrap();

        assert!(report.heatmap[0] > 0.0);
        assert!(report.heatmap[1] > 0.0);
        assert_eq!(report.heatmap[2], 0.0);

        assert!(report.penalty_map[0] > 0.0);
        assert!(report.penalty_map[1] > 0.0);
    }

    #[test]
    fn test_lateral_sfb_mechanics() {
        let keys = vec![
            KeyNode {
                index: 0,
                col: ColIndex(0),
                ..Default::default()
            },
            KeyNode {
                index: 1,
                col: ColIndex(1),
                ..Default::default()
            },
        ];
        let kb = Keyboard::new(keys, 0, "test".into()).unwrap();

        let mut corpus = Corpus::default();
        corpus.bigrams.push((0, 1, 1));

        let mut rubric = Rubric::default();
        rubric.sfb_base = 100.0;
        rubric.sfb_lateral = 200.0;

        let cost_model = mock_cost_model();
        let engine = EngineFactory::new_generic(EngineCompilationContext {
            keyboard: &kb,
            corpus: &corpus,
            rubric: &rubric,
            cost_model: &cost_model,
        })
        .unwrap();
        let layout = Layout::new_unchecked(vec![KeyCode(0), KeyCode(1)]);

        let score = engine.score(&layout).unwrap().to_f32();

        assert!(score >= 200.0);
        assert!(score > 150.0);
    }

    #[test]
    fn test_lateral_stretch() {
        let keys = vec![
            KeyNode {
                index: 0,
                finger: FingerIndex::new_unchecked(1),
                col: ColIndex(0),
                ..Default::default()
            },
            KeyNode {
                index: 1,
                finger: FingerIndex::new_unchecked(2),
                col: ColIndex(2),
                ..Default::default()
            },
        ];
        let kb = Keyboard::new(keys, 0, "test".into()).unwrap();

        let mut corpus = Corpus::default();
        corpus.bigrams.push((0, 1, 1));

        let mut rubric = Rubric::default();
        rubric.sfb_lateral = 500.0;

        let cost_model = mock_cost_model();
        let engine = EngineFactory::new_generic(EngineCompilationContext {
            keyboard: &kb,
            corpus: &corpus,
            rubric: &rubric,
            cost_model: &cost_model,
        })
        .unwrap();
        let layout = Layout::new_unchecked(vec![KeyCode(0), KeyCode(1)]);

        let score = engine.score(&layout).unwrap().to_f32();
        assert!(score >= 500.0);
    }

    #[test]
    fn test_top_metrics_ranking() {
        let keys = vec![
            KeyNode {
                index: 0,
                hand: HandIndex(0),
                finger: FingerIndex::new_unchecked(1),
                ..Default::default()
            },
            KeyNode {
                index: 1,
                hand: HandIndex(0),
                finger: FingerIndex::new_unchecked(1),
                ..Default::default()
            },
            KeyNode {
                index: 2,
                hand: HandIndex(0),
                finger: FingerIndex::new_unchecked(1),
                ..Default::default()
            },
        ];
        let kb = Keyboard::new(keys, 0, "test".into()).unwrap();

        let layout = Layout::new_unchecked(vec![KeyCode(97), KeyCode(98), KeyCode(99)]);

        let mut corpus = Corpus::default();
        corpus.char_freqs[97] = 1000;
        corpus.char_freqs[98] = 1000;
        corpus.char_freqs[99] = 1000;

        corpus.bigrams.push((97, 98, 400));
        corpus.bigrams.push((97, 99, 300));
        corpus.bigrams.push((98, 99, 100));
        corpus.bigrams.push((97, 97, 500));

        let cost_model = mock_cost_model();
        let engine = EngineFactory::new_generic(EngineCompilationContext {
            keyboard: &kb,
            corpus: &corpus,
            rubric: &Rubric::default(),
            cost_model: &cost_model,
        })
        .unwrap();
        let report = engine.analyze(&layout).unwrap();

        let sfbs = report.top_sfbs;
        assert_eq!(sfbs.len(), 3);

        assert_eq!(sfbs[0].keys, "ab");
        assert_eq!(sfbs[1].keys, "ac");
        assert_eq!(sfbs[2].keys, "bc");

        assert!(sfbs[0].freq > sfbs[1].freq);
        assert!(sfbs[1].freq > sfbs[2].freq);
    }

    #[test]
    fn test_repeat_not_sfb() {
        let keys = vec![
            KeyNode {
                index: 0,
                hand: HandIndex(0),
                finger: FingerIndex::new_unchecked(1),
                ..Default::default()
            },
            KeyNode {
                index: 1,
                hand: HandIndex(0),
                finger: FingerIndex::new_unchecked(1),
                row: RowIndex(0),
                ..Default::default()
            },
        ];
        let kb = Keyboard::new(keys, 0, "test".into()).unwrap();
        let layout = Layout::new_unchecked(vec![KeyCode(97), KeyCode(98)]);

        let mut corpus = Corpus::default();
        corpus.char_freqs[97] = 1000;
        corpus.char_freqs[98] = 1000;

        corpus.bigrams.push((97, 97, 100));
        corpus.bigrams.push((97, 98, 100));

        let cost_model = mock_cost_model();
        let engine = EngineFactory::new_generic(EngineCompilationContext {
            keyboard: &kb,
            corpus: &corpus,
            rubric: &Rubric::default(),
            cost_model: &cost_model,
        })
        .unwrap();
        let report = engine.analyze(&layout).unwrap();

        assert_eq!(report.top_sfbs.len(), 1);
        assert_eq!(report.top_sfbs[0].keys, "ab");
        assert!(report.sfb_total > 0.0);
    }

    #[test]
    fn test_thumb_exclusion_from_scissors_and_stretch() {
        let keys = vec![
            KeyNode {
                index: 0,
                hand: HandIndex(0),
                finger: FingerIndex::new_unchecked(0),
                row: RowIndex(0),
                col: ColIndex(0),
                ..Default::default()
            },
            KeyNode {
                index: 1,
                hand: HandIndex(0),
                finger: FingerIndex::new_unchecked(1),
                row: RowIndex(2),
                col: ColIndex(0),
                ..Default::default()
            },
            KeyNode {
                index: 2,
                hand: HandIndex(0),
                finger: FingerIndex::new_unchecked(1),
                row: RowIndex(0),
                col: ColIndex(2),
                ..Default::default()
            },
        ];
        let kb = Keyboard::new(keys, 0, "test".into()).unwrap();

        let layout = Layout::new_unchecked(vec![KeyCode(116), KeyCode(105), KeyCode(115)]);

        let mut corpus = Corpus::default();
        corpus.char_freqs[116] = 1000;
        corpus.char_freqs[105] = 1000;
        corpus.char_freqs[115] = 1000;

        corpus.bigrams.push((116, 105, 500));
        corpus.bigrams.push((116, 115, 500));

        let mut rubric = Rubric::default();
        rubric.penalty_scissor = 1000.0;
        rubric.sfb_lateral = 1000.0;
        rubric.threshold_scissor_row_diff = 2;

        let cost_model = mock_cost_model();
        let engine = EngineFactory::new_generic(EngineCompilationContext {
            keyboard: &kb,
            corpus: &corpus,
            rubric: &rubric,
            cost_model: &cost_model,
        })
        .unwrap();

        let score = engine.score(&layout).unwrap().to_f32();
        assert!(score.is_finite());

        let report = engine.analyze(&layout).unwrap();
        assert_eq!(
            report.scissors, 0.0,
            "Should detect 0 scissors for thumb interactions"
        );
        assert!(
            report.top_scissors.is_empty(),
            "Top scissors should be empty"
        );
    }
}
