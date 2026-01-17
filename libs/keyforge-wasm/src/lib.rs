// libs/keyforge-wasm/src/lib.rs

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

use keyforge_core::loader::AssetLoader;
use keyforge_model::config::CorpusSource;
use keyforge_model::geometry::KeyboardDefinition;
use keyforge_model::keycodes::KeycodeRegistry;
use keyforge_model::{Corpus, Layout, Rubric, CostModel};
use keyforge_model::validator::Validator;
use keyforge_physics::ScoringEngine;
use loader::InMemoryLoader;
use serde_wasm_bindgen::{from_value, to_value};
use std::sync::Arc;
use wasm_bindgen::prelude::*;

#[derive(Debug)]
#[wasm_bindgen]
pub struct KeyforgeEngine {
    loader: Arc<InMemoryLoader>,
}

#[wasm_bindgen]
impl KeyforgeEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();
        Self {
            loader: Arc::new(InMemoryLoader::new()),
        }
    }

    /// Injects a keyboard definition into the in-memory loader.
    #[wasm_bindgen(js_name = injectKeyboard)]
    pub fn inject_keyboard(&self, name: String, json_val: JsValue) -> Result<(), JsValue> {
        let kb: KeyboardDefinition = from_value(json_val)?;
        kb.validate().map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.loader.inject_keyboard(name, kb);
        Ok(())
    }

    /// Injects a corpus into the in-memory loader.
    #[wasm_bindgen(js_name = injectCorpus)]
    pub fn inject_corpus(&self, name: String, json_val: JsValue) -> Result<(), JsValue> {
        let corpus: Corpus = from_value(json_val)?;
        corpus.validate().map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.loader.inject_corpus(name, corpus);
        Ok(())
    }

    /// Injects a cost model into the in-memory loader.
    #[wasm_bindgen(js_name = injectCostModel)]
    pub fn inject_cost_model(&self, name: String, json_val: JsValue) -> Result<(), JsValue> {
        let model: CostModel = from_value(json_val)?;
        // CostModel doesn't have a validate() method yet, but serde handles structure.
        self.loader.inject_cost_model(name, model);
        Ok(())
    }

    /// Injects a keycode registry into the in-memory loader.
    #[wasm_bindgen(js_name = injectKeycodes)]
    pub fn inject_keycodes(&self, name: String, json_val: JsValue) -> Result<(), JsValue> {
        let reg: KeycodeRegistry = from_value(json_val)?;
        reg.validate().map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.loader.inject_keycodes(name, reg);
        Ok(())
    }

    /// Analyzes a layout using the injected assets.
    #[wasm_bindgen(js_name = analyzeLayout)]
    pub async fn analyze_layout(
        &self,
        keyboard_name: String,
        corpus_name: String,
        cost_model_name: String,
        layout_val: JsValue,
    ) -> Result<JsValue, JsValue> {
        let layout: Layout = from_value(layout_val)?;
        
        let kb = self.loader.load_keyboard(&keyboard_name).await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
            
        let corpus = self.loader.load_corpus(&[CorpusSource { id: corpus_name, weight: 1.0, hash: None }]).await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
            
        let cost_model = self.loader.load_cost_model(&cost_model_name).await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        // Create Keyboard from Definition
        let keyboard = keyforge_model::Keyboard::new(
            kb.geometry.keys.clone(),
            kb.geometry.home_row,
        ).map_err(|e| JsValue::from_str(&e.to_string()))?;

        let engine = ScoringEngine::new(&keyboard, &corpus, &Rubric::default(), &cost_model)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let report = engine.analyze(&layout)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(to_value(&report)?)
    }
}
