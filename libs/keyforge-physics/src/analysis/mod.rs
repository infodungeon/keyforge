// libs/keyforge-physics/src/analysis/mod.rs

pub mod fingerprint;
pub mod heuristics;

use crate::error::PhysicsError;
use crate::kernel::types::ValidatedLayout;
use crate::kernel::EngineContext;
use keyforge_model::{AnalysisReport, Layout, SwapSuggestion};
use tracing::instrument;

pub use fingerprint::{Fingerprinter, LayoutIdentity};
pub use heuristics::suggest_swaps;

#[instrument]
pub fn identify(layout: &Layout) -> Option<LayoutIdentity> {
    Fingerprinter::identify(layout)
}

/// Analyzes a layout and returns a detailed ergonomic report.
///
/// # Errors
/// Returns `PhysicsError` if the layout is invalid for the current engine context.
pub fn analyze_with_context(
    ctx: &EngineContext,
    layout: &Layout,
) -> Result<AnalysisReport, PhysicsError> {
    let validated = ValidatedLayout::new(&layout.keys, ctx.key_count)?;
    Ok(crate::kernel::compute::analyze_layout(ctx, &validated))
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
        let kb = Arc::new(Keyboard::new(keys, 0, "test".into()).unwrap());
        let corpus = Arc::new(Corpus::default());
        let rubric = Arc::new(Rubric::default());
        let mut cm = CostModel::default();
        let mut fingers = HashMap::new();
        fingers.insert(
            "index".to_string(),
            keyforge_model::cost_model::FingerDefinition::Standard(
                keyforge_model::cost_model::FingerReach {
                    base: HashMap::from([(RowIndex(0), 1.0)]),
                    ..Default::default()
                },
            ),
        );
        fingers.insert(
            "middle".to_string(),
            keyforge_model::cost_model::FingerDefinition::Standard(
                keyforge_model::cost_model::FingerReach {
                    base: HashMap::from([(RowIndex(0), 1.0)]),
                    ..Default::default()
                },
            ),
        );
        cm.models.insert(
            "model_a_row_staggered".into(),
            keyforge_model::cost_model::ModelDefinition {
                description: "test".into(),
                static_costs: HashMap::from([(
                    "universal_hand".to_string(),
                    keyforge_model::cost_model::HandDefinition { fingers },
                )]),
            },
        );
        (kb, corpus, rubric, Arc::new(cm))
    }

    #[test]
    fn test_identify_qwerty() {
        let qwerty = "Q W E R T Y U I O P A S D F G H J K L Z X C V B N M";
        let reg = keyforge_model::KeycodeRegistry::new_with_alphas();
        let layout = keyforge_adapter::conversion::parse_layout_string(qwerty, 30, &reg).unwrap();

        let identity = identify(&layout);
        assert!(identity.is_some());
        assert_eq!(identity.unwrap().name, "Qwerty");
    }

    #[test]
    fn test_analyze_with_context() {
        let (keyboard, corpus, rubric, cost_model) = setup_minimal();
        let engine = EngineFactory::new_generic(&EngineCompilationContext {
            keyboard,
            corpus,
            rubric,
            cost_model,
        })
        .unwrap();

        let layout = Layout::new_unchecked(vec![KeyCode(97), KeyCode(98)]);
        let res = analyze_with_context(engine.context(), &layout);
        assert!(res.is_ok());
    }
}
