use keyforge_model::error::ForgeError;
use keyforge_model::loader::{AssetLoader, LoaderResult, RawCostData};
use keyforge_model::Corpus;
use keyforge_protocol::config::CorpusSource;
use keyforge_protocol::geometry::KeyboardDefinition;
use keyforge_protocol::keycodes::KeycodeRegistry;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Clone, Default)]
pub struct InMemoryLoader {
    keyboards: Arc<RwLock<HashMap<String, KeyboardDefinition>>>,
    corpora: Arc<RwLock<HashMap<String, Corpus>>>,
    costs: Arc<RwLock<HashMap<String, RawCostData>>>,
    keycodes: Arc<RwLock<KeycodeRegistry>>,
}

impl InMemoryLoader {
    pub fn add_keyboard(&self, name: String, def: KeyboardDefinition) {
        self.keyboards.write().unwrap().insert(name, def);
    }

    pub fn add_corpus(&self, name: String, corpus: Corpus) {
        self.corpora.write().unwrap().insert(name, corpus);
    }

    pub fn add_cost(&self, name: String, cost: RawCostData) {
        self.costs.write().unwrap().insert(name, cost);
    }

    pub fn set_keycodes(&self, registry: KeycodeRegistry) {
        *self.keycodes.write().unwrap() = registry;
    }
}

impl AssetLoader for InMemoryLoader {
    fn load_keyboard(&self, name: &str) -> LoaderResult<KeyboardDefinition> {
        self.keyboards
            .read()
            .unwrap()
            .get(name)
            .cloned()
            .ok_or_else(|| {
                ForgeError::NotFound(format!("Keyboard '{}' not found in memory", name))
            })
    }

    fn load_corpus(&self, sources: &[CorpusSource]) -> LoaderResult<Corpus> {
        if let Some(first) = sources.first() {
            self.corpora
                .read()
                .unwrap()
                .get(&first.id)
                .cloned()
                .ok_or_else(|| {
                    ForgeError::NotFound(format!("Corpus '{}' not found in memory", first.id))
                })
        } else {
            Ok(Corpus::default())
        }
    }

    fn load_cost_matrix(&self, filename: &str) -> LoaderResult<RawCostData> {
        self.costs
            .read()
            .unwrap()
            .get(filename)
            .cloned()
            .ok_or_else(|| {
                ForgeError::NotFound(format!("Cost matrix '{}' not found in memory", filename))
            })
    }

    fn load_keycodes(&self, _filename: &str) -> LoaderResult<KeycodeRegistry> {
        Ok(self.keycodes.read().unwrap().clone())
    }
}
