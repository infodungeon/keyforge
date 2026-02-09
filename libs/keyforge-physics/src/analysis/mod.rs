// libs/keyforge-physics/src/analysis/mod.rs

pub mod heuristics;

use crate::error::PhysicsError;
use crate::kernel::types::ValidatedLayout;
use crate::kernel::EngineContext;
use keyforge_model::{AnalysisReport, Layout, SwapSuggestion};
use tracing::instrument;

pub use heuristics::suggest_swaps;
pub use keyforge_model::layout::LayoutIdentity;

/// Identifies a layout by comparing it to known standards.
#[instrument]
pub fn identify(layout: &Layout) -> Option<LayoutIdentity> {
    layout.identify()
}

/// Analyzes a layout and returns a detailed ergonomic report.
///
/// # Errors
/// Returns `PhysicsError` if the layout is invalid for the current engine context.
pub fn analyze_with_context(
    ctx: &EngineContext,
    layout: &Layout,
) -> Result<AnalysisReport, PhysicsError> {
    let validated = ValidatedLayout::new(layout.keys(), ctx.key_count)?;
    crate::kernel::compute::analyze_layout(ctx, &validated)
}

/// Suggests improvements for the given layout.
#[must_use]
pub fn suggest_improvements_with_context(
    ctx: &EngineContext,
    layout: &Layout,
    include_thumbs: bool,
) -> Vec<SwapSuggestion> {
    suggest_swaps(ctx, layout, include_thumbs)
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;
    use crate::{EngineCompilationContext, EngineFactory};
    use keyforge_model::types::{FingerIndex, HandIndex, KeyCode, RowIndex};
    use keyforge_model::{Corpus, CostModel, KeyNode, Keyboard, Rubric};
    use std::collections::HashMap;
    use std::sync::Arc;

    fn setup_minimal() -> anyhow::Result<(Arc<Keyboard>, Arc<Corpus>, Arc<Rubric>, Arc<CostModel>)>
    {
        let keys = vec![
            KeyNode {
                index: KeyIndex::new(0),
                hand: HandIndex::LEFT,
                finger: FingerIndex::INDEX,
                row: RowIndex::new(0),
                ..Default::default()
            },
            KeyNode {
                index: KeyIndex::new(1),
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
        let sc = |v: f32| keyforge_model::types::Score::from_f32(v).unwrap_or_default();
        fingers.insert(
            "index".to_string(),
            keyforge_model::cost_model::FingerDefinition::Standard(
                keyforge_model::cost_model::FingerReach {
                    base: HashMap::from([(RowIndex::new(0), sc(1.0))]),
                    ..Default::default()
                },
            ),
        );
        fingers.insert(
            "middle".to_string(),
            keyforge_model::cost_model::FingerDefinition::Standard(
                keyforge_model::cost_model::FingerReach {
                    base: HashMap::from([(RowIndex::new(0), sc(1.0))]),
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
    fn test_identify_qwerty() -> anyhow::Result<()> {
        let qwerty = "Q W E R T Y U I O P A S D F G H J K L Z X C V B N M";
        let reg = keyforge_model::keycodes::KeycodeRegistry::new_with_alphas();
        let layout = keyforge_adapter::conversion::parse_layout_string(qwerty, 30, &reg)?;

        let identity = identify(&layout);
        assert!(identity.is_some());
        assert_eq!(
            identity
                .ok_or_else(|| anyhow::anyhow!("missing identity"))?
                .name,
            "Qwerty"
        );
        Ok(())
    }

    #[test]
    fn test_analyze_with_context() -> anyhow::Result<()> {
        let (keyboard, corpus, rubric, cost_model) = setup_minimal()?;
        let engine = EngineFactory::new_generic(&EngineCompilationContext {
            keyboard,
            corpus,
            rubric,
            cost_model,
            engine_config: keyforge_model::config::EngineConfig::default(),
        })?;

        let layout = Layout::new_unchecked(vec![KeyCode::new(97), KeyCode::new(98)]);
        let res = analyze_with_context(engine.context(), &layout);
        assert!(res.is_ok());
        Ok(())
    }
}
