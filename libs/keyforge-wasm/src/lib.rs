// Copyright (c) 2025 KeyForge Contributors
//
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
mod loader;

use keyforge_adapter::conversion;
use keyforge_core::ScoringEngine;
use keyforge_core::loader::{AssetLoader, RawCostData};
use keyforge_model::Corpus;
use keyforge_protocol::config::{ScoringWeights, SearchParams};
use keyforge_model::config::CorpusSource;
use keyforge_model::geometry::{KeyboardDefinition, KeyboardGeometry};
use keyforge_model::keycodes::KeycodeRegistry;
use loader::InMemoryLoader;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct KeyforgeEngine {
    loader: InMemoryLoader,
    engine: Option<ScoringEngine>,
    registry: Option<KeycodeRegistry>,
    geometry: Option<KeyboardGeometry>,
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
        self.loader.add_keyboard(name, def).map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(())
    }

    pub fn load_keycodes(&self, json_registry: JsValue) -> Result<(), JsValue> {
        let reg: KeycodeRegistry = serde_wasm_bindgen::from_value(json_registry)?;
        self.loader.set_keycodes(reg).map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(())
    }

    pub fn load_corpus(&self, name: String, json_corpus: JsValue) -> Result<(), JsValue> {
        let corpus: Corpus = serde_wasm_bindgen::from_value(json_corpus)?;
        self.loader.add_corpus(name, corpus).map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(())
    }

    pub fn load_cost_matrix(&self, name: String, json_cost: JsValue) -> Result<(), JsValue> {
        let cost: RawCostData = serde_wasm_bindgen::from_value(json_cost)?;
        self.loader.add_cost(name, cost).map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(())
    }

    pub async fn init_session(
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
            .await
            .map_err(|e| e.to_string())?;

        let reg = self
            .loader
            .load_keycodes("keycodes.json")
            .await
            .map_err(|e| e.to_string())?;

        let sources = [CorpusSource {
            id: corpus_name,
            weight: 1.0,
            hash: None,
        }];

        let corpus = self
            .loader
            .load_corpus(&sources)
            .await
            .map_err(|e| e.to_string())?;

        let cost = self
            .loader
            .load_cost_matrix(&cost_matrix)
            .await
            .map_err(|e| e.to_string())?;

        // def is Model. Construct keyforge_model::Keyboard directly.
        let keyboard = keyforge_model::Keyboard::new(def.geometry.keys.clone(), def.geometry.home_row)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
            
        let rubric = conversion::to_domain_rubric(&w); // w is still Protocol DTO because we deserialize it from JS?
        // Wait, imported 'ScoringWeights' is now keyforge_model::config::ScoringWeights (Line 7 replacement).
        // So 'w' is Model.
        // conversion::to_domain_rubric takes Protocol.
        // So, if we deserialize directly to Model, we don't need adapter!
        // Is 'Rubric' the same as 'ScoringWeights'? No. Rubric is the internal engine representation.
        // Adapter converts Protocol(ScoringWeights) -> Model(Rubric).
        // Does Model(ScoringWeights) -> Model(Rubric) exist?
        // Probably implicit or we need adapter to handle Model input too?
        // Actually, adapter::to_domain_rubric takes 'config::ScoringWeights'. 'config' alias in adapter refers to Protocol.
        
        // Wait. 'w' logic:
        // let w: ScoringWeights = serde_wasm_bindgen::from_value(weights)?;
        // If imports changed to Model, w is Model.
        // to_domain_rubric wants Protocol.
        // This is a mismatch.
        
        // SOLUTION: keep imports for 'ScoringWeights', 'SearchParams' as PROTOCOL in lib.rs, OR convert Model->Rubric manually?
        // Adapter logic for rubric is complex (normalizing weights).
        // It's better to treat JS input as PROTOCOL DTOs.
        
        // Let's REVERT import changes partially? 
        // No, mixed imports are messy.
        // Let's use Fully Qualified syntax for Protocol types where needed.
        
        // But for 'AssetLoader', we MUST use Model.
        
        // Let's use explicit conversion function?
        // If I have Model::ScoringWeights, can I use it?
        // Adapter expects Protocol::ScoringWeights.
        // Model and Protocol Sharing: Structs are identical.
        // I can just cast or transmute? Unsafe.
        // Or just import imports as Protocol and convert for Loader?
        
        // BUT Loader expects Model::CorpusSource.
        // So 'init_session' has MIXED requirements.
        // JS inputs -> Protocol DTOs.
        // Loader -> Model inputs.
        
        // Correct approach:
        // Deserialize JS -> Protocol DTOs.
        // Convert Protocol DTOs -> Model DTOs (for Loader) OR Domain Objects (for Engine).
        
        // SO: imports in lib.rs should probably stay PROTOCOL for JS inputs.
        // But internal fields should use MODEL.
        
        // Let's fix lib.rs imports to be specific.
        
        let overrides = cost.resolve(&def.geometry);

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

        let report = engine.analyze(&layout)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
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
