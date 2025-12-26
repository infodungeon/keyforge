use crate::error::KeyForgeError;
use crate::Corpus;
use keyforge_protocol::config::CorpusSource;
use keyforge_protocol::geometry::KeyboardDefinition;
use keyforge_protocol::keycodes::KeycodeRegistry;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawCostData {
    pub entries: Vec<(String, String, f32)>,
}

pub type LoaderResult<T> = Result<T, KeyForgeError>;

pub trait AssetLoader: Send + Sync {
    fn load_keyboard(&self, name: &str) -> LoaderResult<KeyboardDefinition>;
    fn load_corpus(&self, sources: &[CorpusSource]) -> LoaderResult<Corpus>;
    fn load_cost_matrix(&self, filename: &str) -> LoaderResult<RawCostData>;
    fn load_keycodes(&self, filename: &str) -> LoaderResult<KeycodeRegistry>;
}
