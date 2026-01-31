// libs/keyforge-physics/src/analysis/heuristics.rs

use crate::kernel::EngineContext;
use keyforge_model::{Layout, SwapSuggestion};

/// Suggests optimal swaps for a layout using iterative heuristic refinement.
#[must_use]
pub fn suggest_swaps(
    _ctx: &EngineContext,
    _layout: &Layout,
    _include_thumbs: bool,
) -> Vec<SwapSuggestion> {
    // Current placeholder implementation
    vec![]
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;
    use crate::{EngineCompilationContext, EngineFactory};
    use keyforge_model::types::{FingerIndex, HandIndex, KeyCode, RowIndex};
    use keyforge_model::{Corpus, CostModel, KeyNode, Keyboard, Rubric};
    use std::collections::HashMap;
    use std::sync::Arc;

    fn setup_minimal() -> (Arc<Keyboard>, Arc<Corpus>, Arc<Rubric>, Arc<CostModel>) {
        let keys = vec![
            KeyNode {
                index: 0,
                hand: HandIndex::LEFT,
                finger: FingerIndex::INDEX,
                row: RowIndex(0),
                ..Default::default()
            },
            KeyNode {
                index: 1,
                hand: HandIndex::LEFT,
                finger: FingerIndex::MIDDLE,
                row: RowIndex(0),
                ..Default::default()
            },
        ];
        let kb = Arc::new(Keyboard::new(keys, RowIndex::new(0), "test".into()).unwrap());
        let corpus = Arc::new(Corpus::default());
        let rubric = Arc::new(Rubric::default());
        let mut cost_model = keyforge_model::cost_model::CostModel::default();
        let mut fingers = HashMap::new();
        fingers.insert(
            "index".to_string(),
            keyforge_model::cost_model::FingerDefinition::Standard(
                keyforge_model::cost_model::FingerReach {
                    base: HashMap::from([(RowIndex::new(0), 1.0)]),
                    ..Default::default()
                },
            ),
        );
        fingers.insert(
            "middle".to_string(),
            keyforge_model::cost_model::FingerDefinition::Standard(
                keyforge_model::cost_model::FingerReach {
                    base: HashMap::from([(RowIndex::new(0), 1.0)]),
                    ..Default::default()
                },
            ),
        );
        cost_model.models.insert(
            "model_a_row_staggered".into(),
            keyforge_model::cost_model::ModelDefinition {
                description: "test".into(),
                static_costs: HashMap::from([(
                    "universal_hand".to_string(),
                    keyforge_model::cost_model::HandDefinition { fingers },
                )]),
            },
        );
        (kb, corpus, rubric, Arc::new(cost_model))
    }

    #[test]
    fn test_suggest_swaps_smoke() {
        let (keyboard, corpus, rubric, cost_model) = setup_minimal();
        let engine = EngineFactory::new_generic(&EngineCompilationContext {
            keyboard,
            corpus,
            rubric,
            cost_model,
            engine_config: keyforge_model::config::EngineConfig::default(),
        })
        .unwrap();

        let layout = Layout::new_unchecked(vec![KeyCode::new(97), KeyCode::new(98)]);
        let res = suggest_swaps(engine.context(), &layout, false);
        assert!(res.is_empty()); // Placeholder
    }
}
