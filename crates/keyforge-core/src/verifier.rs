use crate::keycodes::KeycodeRegistry;
use crate::layouts::layout_string_to_u16;
use crate::optimizer::mutation;
use crate::scorer::{ScoreDetails, Scorer};
use keyforge_protocol::config::Config; // UPDATED
use keyforge_protocol::geometry::KeyboardGeometry; // UPDATED
use std::path::Path;
use std::sync::Arc;

#[derive(Clone)]
pub struct Verifier {
    scorer: Arc<Scorer>,
    registry: Arc<KeycodeRegistry>,
}

impl Verifier {
    pub fn new(
        cost_path: &str,
        corpus_dir: &str,
        geometry: &KeyboardGeometry,
        config: Config,
        registry_path: &str,
    ) -> Result<Self, String> {
        let scorer = Scorer::new(cost_path, corpus_dir, geometry, config, false)
            .map_err(|e| e.to_string())?;

        let registry = if Path::new(registry_path).exists() {
            KeycodeRegistry::load_from_file(registry_path)
                .map_err(|e| format!("Failed to load registry: {}", e))?
        } else {
            KeycodeRegistry::new_with_defaults()
        };

        Ok(Self {
            scorer: Arc::new(scorer),
            registry: Arc::new(registry),
        })
    }

    pub fn from_components(scorer: Arc<Scorer>, registry: Arc<KeycodeRegistry>) -> Self {
        Self { scorer, registry }
    }

    pub fn verify(
        &self,
        layout_str: String,
        claimed_score: f32,
        tolerance: f32,
    ) -> Result<bool, String> {
        let details = self.score_details(layout_str);
        let diff = (details.layout_score - claimed_score).abs();

        if diff <= tolerance {
            Ok(true)
        } else {
            tracing::warn!(
                "Score verification mismatch. Claimed: {:.2}, Calculated: {:.2}, Diff: {:.2}",
                claimed_score,
                details.layout_score,
                diff
            );
            Ok(false)
        }
    }

    pub fn score_details(&self, layout_str: String) -> ScoreDetails {
        let key_count = self.scorer.key_count;
        let layout_codes = layout_string_to_u16(&layout_str, key_count, &self.registry);
        let pos_map = mutation::build_pos_map(&layout_codes);
        self.scorer.score_details(&pos_map, 3000)
    }
}
