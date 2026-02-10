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
    use keyforge_model::types::{FingerIndex, HandIndex, KeyCode, KeyIndex, RowIndex};
    use keyforge_model::{Corpus, CostModel, KeyNode, Keyboard, Rubric};
    use std::collections::HashMap;
    use std::sync::Arc;

    fn setup_minimal() -> anyhow::Result<(Arc<Keyboard>, Arc<Corpus>, Arc<Rubric>, Arc<CostModel>)>
    {
        let keys = vec![
            KeyNode {
                index: KeyIndex(0),
                hand: HandIndex::LEFT,
                finger: FingerIndex::INDEX,
                row: RowIndex::new(0),
                ..Default::default()
            },
            KeyNode {
                index: KeyIndex(1),
                hand: HandIndex::LEFT,
                finger: FingerIndex::MIDDLE,
                row: RowIndex::new(0),
                ..Default::default()
            },
        ];
        let kb = Arc::new(Keyboard::new(keys, RowIndex::new(0), "test".into())?);
        let corpus = Arc::new(Corpus::default());
        let rubric = Arc::new(Rubric::default());
        let mut cost_model = keyforge_model::cost_model::CostModel::default();
        let mut fingers = HashMap::new();
        let sc = |v: i64| keyforge_model::types::Score::from_scaled_i64(v);
        fingers.insert(
            "index".to_string(),
            keyforge_model::cost_model::FingerDefinition::Standard(
                keyforge_model::cost_model::FingerReach {
                    base: HashMap::from([(RowIndex::new(0), sc(1_000_000))]),
                    ..Default::default()
                },
            ),
        );
        fingers.insert(
            "middle".to_string(),
            keyforge_model::cost_model::FingerDefinition::Standard(
                keyforge_model::cost_model::FingerReach {
                    base: HashMap::from([(RowIndex::new(0), sc(1_000_000))]),
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
        Ok((kb, corpus, rubric, Arc::new(cost_model)))
    }

    #[test]
    fn test_suggest_swaps_smoke() -> anyhow::Result<()> {
        let (keyboard, corpus, rubric, cost_model) = setup_minimal()?;
        let engine = EngineFactory::new_generic(&EngineCompilationContext {
            keyboard,
            corpus,
            rubric,
            cost_model,
            engine_config: keyforge_model::config::EngineConfig::default(),
        })?;

        let layout = Layout::new_unchecked(vec![KeyCode::new(97), KeyCode::new(98)]);
        let res = suggest_swaps(engine.context(), &layout, false);
        assert!(res.is_empty()); // Placeholder
        Ok(())
    }
}
