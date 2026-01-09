//! # KeyForge WASM Bindings
//!
//! WebAssembly bindings for the KeyForge scoring engine. This crate 
//! exposes a high-level JS interface for layout analysis and 
//! optimization in the browser.

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
use std::sync::Arc;
use keyforge_core::ScoringEngine;
use keyforge_core::loader::{AssetLoader, RawCostData};
use keyforge_model::Corpus;
use keyforge_model::config::{CorpusSource, ScoringWeights};
use keyforge_model::SearchConfig;
use keyforge_model::geometry::{KeyboardDefinition, KeyboardGeometry};
use keyforge_model::keycodes::KeycodeRegistry;
use keyforge_model::validator::{LayoutValidator, Validator};
use loader::InMemoryLoader;
use wasm_bindgen::prelude::*;

/// The primary entry point for using KeyForge in WebAssembly environments.
#[wasm_bindgen]
pub struct KeyforgeEngine {
    loader: InMemoryLoader,
    engine: Option<ScoringEngine>,
    registry: Option<Arc<KeycodeRegistry>>,
    geometry: Option<KeyboardGeometry>,
    search_config: Option<SearchConfig>,
}

impl Default for KeyforgeEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl KeyforgeEngine {
    /// Creates a new `KeyforgeEngine` instance.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();
        Self {
            loader: InMemoryLoader::default(),
            engine: None,
            registry: None,
            geometry: None,
            search_config: None,
        }
    }

    /// Loads a keyboard definition into the engine's memory.
    pub fn load_keyboard(&self, name: String, json_def: JsValue) -> Result<(), JsValue> {
        let def: KeyboardDefinition = serde_wasm_bindgen::from_value(json_def)?;
        def.validate().map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.loader.add_keyboard(name, def).map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(())
    }

    /// Loads the keycode registry into the engine's memory.
    pub fn load_keycodes(&self, json_registry: JsValue) -> Result<(), JsValue> {
        let reg: KeycodeRegistry = serde_wasm_bindgen::from_value(json_registry)?;
        reg.validate().map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.loader.set_keycodes(reg).map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(())
    }

    /// Loads a language corpus into the engine's memory.
    pub fn load_corpus(&self, name: String, json_corpus: JsValue) -> Result<(), JsValue> {
        let corpus: Corpus = serde_wasm_bindgen::from_value(json_corpus)?;
        corpus.validate().map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.loader.add_corpus(name, corpus).map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(())
    }

    /// Loads a physical cost matrix into the engine's memory.
    pub fn load_cost_matrix(&self, name: String, json_cost: JsValue) -> Result<(), JsValue> {
        let cost: RawCostData = serde_wasm_bindgen::from_value(json_cost)?;
        cost.validate().map_err(|e| JsValue::from_str(&e))?;
        self.loader.add_cost(name, cost).map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(())
    }

    /// Initializes a scoring session with the specified assets and weights.
    pub async fn init_session(
        &mut self,
        keyboard_name: String,
        corpus_source: JsValue,
        cost_matrix: String,
        weights: JsValue,
        params: JsValue,
    ) -> Result<(), JsValue> {
        // Note: We treat JS inputs as Protocol DTOs where possible, but here we need Model types for the Engine.
        // Since we don't have a full Protocol->Model adapter in WASM yet, we rely on serde compatibility.
        
        let w: ScoringWeights = serde_wasm_bindgen::from_value(weights)?;
        w.validate().map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        // Validate search parameters
        let p: keyforge_model::config::SearchParams = serde_wasm_bindgen::from_value(params)?;
        p.validate().map_err(|e| JsValue::from_str(&e))?;
        // Convert to domain config (hardcoded seed for WASM determinism unless passed?)
        // WASM usually wants determinism. Let's use 42 for now as we don't expose seed in params DTO yet?
        // Actually Params DTO might not have seed.
        let domain_config = conversion::to_domain_config(&p, 42);

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

        // Handle polymorphic corpus source (String ID or Array of Weighted Sources)
        let sources: Vec<CorpusSource> = if corpus_source.is_string() {
            let id = corpus_source.as_string().unwrap();
            vec![CorpusSource {
                id,
                weight: 1.0,
                hash: None,
            }]
        } else {
            serde_wasm_bindgen::from_value(corpus_source)?
        };

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

        let keyboard = keyforge_model::Keyboard::new(def.geometry.keys.clone(), def.geometry.home_row)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
            
        let rubric = conversion::to_domain_rubric(&w); 
        let overrides = cost.resolve(&def.geometry);

        let engine = ScoringEngine::new(&keyboard, &corpus, &rubric, &overrides)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        self.engine = Some(engine);
        self.registry = Some(reg);
        self.geometry = Some(def.geometry.clone());
        self.search_config = Some(domain_config);
        Ok(())
    }

    /// Analyzes a layout string and returns a comprehensive report.
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

use keyforge_protocol::JobRequest;

/// Validates a job request object passed from JavaScript.
#[wasm_bindgen]
pub fn validate_job_request(json_req: JsValue) -> Result<(), JsValue> {
    let req: JobRequest = serde_wasm_bindgen::from_value(json_req)?;
    req.validate().map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Validates the structure of a layout string.
#[wasm_bindgen]
pub fn validate_layout_structure(layout_str: String) -> Result<(), JsValue> {
    LayoutValidator::validate_structure(&layout_str).map_err(|e| JsValue::from_str(&e.to_string()))
}
