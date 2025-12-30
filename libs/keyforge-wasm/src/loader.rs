use keyforge_model::error::ForgeError;
use keyforge_core::loader::{AssetLoader, LoaderResult, RawCostData};
use keyforge_model::Corpus;

// Actually we need model types.
use keyforge_model::config::CorpusSource;
use keyforge_model::geometry::KeyboardDefinition;
use keyforge_model::keycodes::KeycodeRegistry;
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
    pub fn add_keyboard(&self, name: String, def: KeyboardDefinition) -> LoaderResult<()> {
        self.keyboards.write().map_err(|e| ForgeError::Internal(format!("RwLock poisoned: {}", e)))?.insert(name, def);
        Ok(())
    }

    pub fn add_corpus(&self, name: String, corpus: Corpus) -> LoaderResult<()> {
        self.corpora.write().map_err(|e| ForgeError::Internal(format!("RwLock poisoned: {}", e)))?.insert(name, corpus);
        Ok(())
    }

    pub fn add_cost(&self, name: String, cost: RawCostData) -> LoaderResult<()> {
        self.costs.write().map_err(|e| ForgeError::Internal(format!("RwLock poisoned: {}", e)))?.insert(name, cost);
        Ok(())
    }

    pub fn set_keycodes(&self, registry: KeycodeRegistry) -> LoaderResult<()> {
        *self.keycodes.write().map_err(|e| ForgeError::Internal(format!("RwLock poisoned: {}", e)))? = registry;
        Ok(())
    }
}

#[async_trait::async_trait]
impl AssetLoader for InMemoryLoader {
    async fn load_keyboard(&self, name: &str) -> LoaderResult<KeyboardDefinition> {
        self.keyboards
            .read()
            .map_err(|e| ForgeError::Internal(format!("RwLock poisoned: {}", e)))?
            .get(name)
            .cloned()
            .ok_or_else(|| {
                ForgeError::NotFound(format!("Keyboard '{}' not found in memory", name))
            })
    }

    async fn load_corpus(&self, sources: &[CorpusSource]) -> LoaderResult<Corpus> {
        if let Some(first) = sources.first() {
            self.corpora
                .read()
                .map_err(|e| ForgeError::Internal(format!("RwLock poisoned: {}", e)))?
                .get(&first.id)
                .cloned()
                .ok_or_else(|| {
                    ForgeError::NotFound(format!("Corpus '{}' not found in memory", first.id))
                })
        } else {
            Ok(Corpus::default())
        }
    }

    async fn load_cost_matrix(&self, filename: &str) -> LoaderResult<RawCostData> {
        self.costs
            .read()
            .map_err(|e| ForgeError::Internal(format!("RwLock poisoned: {}", e)))?
            .get(filename)
            .cloned()
            .ok_or_else(|| {
                ForgeError::NotFound(format!("Cost matrix '{}' not found in memory", filename))
            })
    }

    async fn load_keycodes(&self, _filename: &str) -> LoaderResult<KeycodeRegistry> {
        Ok(self.keycodes.read().map_err(|e| ForgeError::Internal(format!("RwLock poisoned: {}", e)))?.clone())
    }
}
