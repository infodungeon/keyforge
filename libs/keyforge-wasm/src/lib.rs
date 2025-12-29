mod loader;

use keyforge_adapter::conversion;
use keyforge_core::ScoringEngine;
use keyforge_model::loader::{AssetLoader, RawCostData};
use keyforge_model::Corpus;
use keyforge_protocol::config::{CorpusSource, ScoringWeights, SearchParams};
use keyforge_protocol::geometry::KeyboardDefinition;
use keyforge_protocol::keycodes::KeycodeRegistry;
use loader::InMemoryLoader;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct KeyforgeEngine {
    loader: InMemoryLoader,
    engine: Option<ScoringEngine>,
    registry: Option<KeycodeRegistry>,
    geometry: Option<keyforge_protocol::geometry::KeyboardGeometry>,
}

impl Default for KeyforgeEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl KeyforgeEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();
        Self {
            loader: InMemoryLoader::default(),
            engine: None,
            registry: None,
            geometry: None,
        }
    }

    pub fn load_keyboard(&self, name: String, json_def: JsValue) -> Result<(), JsValue> {
        let def: KeyboardDefinition = serde_wasm_bindgen::from_value(json_def)?;
        self.loader.add_keyboard(name, def);
        Ok(())
    }

    pub fn load_keycodes(&self, json_registry: JsValue) -> Result<(), JsValue> {
        let reg: KeycodeRegistry = serde_wasm_bindgen::from_value(json_registry)?;
        self.loader.set_keycodes(reg);
        Ok(())
    }

    pub fn load_corpus(&self, name: String, json_corpus: JsValue) -> Result<(), JsValue> {
        let corpus: Corpus = serde_wasm_bindgen::from_value(json_corpus)?;
        self.loader.add_corpus(name, corpus);
        Ok(())
    }

    pub fn load_cost_matrix(&self, name: String, json_cost: JsValue) -> Result<(), JsValue> {
        let cost: RawCostData = serde_wasm_bindgen::from_value(json_cost)?;
        self.loader.add_cost(name, cost);
        Ok(())
    }

    pub fn init_session(
        &mut self,
        keyboard_name: String,
        corpus_name: String,
        cost_matrix: String,
        weights: JsValue,
        _params: JsValue,
    ) -> Result<(), JsValue> {
        let w: ScoringWeights = serde_wasm_bindgen::from_value(weights)?;
        let _p: SearchParams = serde_wasm_bindgen::from_value(_params)?;

        // Load assets from in-memory loader
        let def = self
            .loader
            .load_keyboard(&keyboard_name)
            .map_err(|e| e.to_string())?;

        let reg = self
            .loader
            .load_keycodes("keycodes.json")
            .map_err(|e| e.to_string())?;

        let sources = [CorpusSource {
            id: corpus_name,
            weight: 1.0,
            hash: None,
        }];

        let corpus = self
            .loader
            .load_corpus(&sources)
            .map_err(|e| e.to_string())?;

        let cost = self
            .loader
            .load_cost_matrix(&cost_matrix)
            .map_err(|e| e.to_string())?;

        let keyboard = conversion::to_domain_keyboard(&def.geometry);
        let rubric = conversion::to_domain_rubric(&w);
        let overrides = conversion::resolve_cost_matrix(&cost.entries, &def.geometry);

        let engine = ScoringEngine::new(&keyboard, &corpus, &rubric, &overrides)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        self.engine = Some(engine);
        self.registry = Some(reg);
        self.geometry = Some(def.geometry);
        Ok(())
    }

    pub fn analyze_layout(&self, layout_str: String) -> Result<JsValue, JsValue> {
        let engine = self
            .engine
            .as_ref()
            .ok_or_else(|| JsValue::from_str("Session not initialized"))?;
        let registry = self
            .registry
            .as_ref()
            .ok_or_else(|| JsValue::from_str("Keycodes not loaded"))?;

        let key_count = engine.key_count();
        let layout = conversion::parse_layout_string(&layout_str, key_count, registry)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let report = engine.analyze(&layout);
        Ok(serde_wasm_bindgen::to_value(&report)?)
    }
}

use keyforge_protocol::{JobRequest, LayoutValidator, Validator};

#[wasm_bindgen]
pub fn validate_job_request(json_req: JsValue) -> Result<(), JsValue> {
    let req: JobRequest = serde_wasm_bindgen::from_value(json_req)?;
    req.validate().map_err(|e| JsValue::from_str(&e))
}

#[wasm_bindgen]
pub fn validate_layout_structure(layout_str: String) -> Result<(), JsValue> {
    LayoutValidator::validate_structure(&layout_str).map_err(|e| JsValue::from_str(&e))
}
