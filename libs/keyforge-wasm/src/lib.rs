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

use keyforge_compute::{AssetLoader, InMemoryLoader};
use keyforge_model::config::CorpusSource;
use keyforge_model::geometry::KeyboardDefinition;
use keyforge_model::keycodes::KeycodeRegistry;
use keyforge_model::validator::Validator;
use keyforge_model::{Corpus, CostModel, Layout, Rubric};
use serde::Serialize;
use serde_wasm_bindgen::{from_value, to_value};
use std::sync::Arc;
use wasm_bindgen::prelude::*;

#[derive(Debug)]
#[wasm_bindgen]
pub struct KeyforgeEngine {
    loader: Arc<InMemoryLoader>,
}

impl Default for KeyforgeEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize)]
struct WasmError {
    kind: String,
    message: String,
}

fn to_js_error(e: &impl ToString) -> JsValue {
    let err = WasmError {
        kind: "KeyForgeError".into(),
        message: e.to_string(),
    };
    to_value(&err).unwrap_or_else(|_| JsValue::from_str(&err.message))
}

#[wasm_bindgen]
impl KeyforgeEngine {
    #[wasm_bindgen(constructor)]
    #[must_use]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();
        Self {
            loader: Arc::new(InMemoryLoader::new()),
        }
    }

    /// Injects a keyboard definition into the in-memory loader.
    ///
    /// # Errors
    ///
    /// Returns an error if the JSON is invalid or fails validation.
    #[wasm_bindgen(js_name = injectKeyboard)]
    pub fn inject_keyboard(&self, name: &str, json_val: JsValue) -> Result<(), JsValue> {
        let kb: KeyboardDefinition = from_value(json_val).map_err(|e| to_js_error(&e))?;
        kb.validate().map_err(|e| to_js_error(&e))?;
        self.loader.inject_keyboard(name, kb);
        Ok(())
    }

    /// Injects a corpus into the in-memory loader.
    ///
    /// # Errors
    ///
    /// Returns an error if the JSON is invalid or fails validation.
    #[wasm_bindgen(js_name = injectCorpus)]
    pub fn inject_corpus(&self, name: &str, json_val: JsValue) -> Result<(), JsValue> {
        let corpus: Corpus = from_value(json_val).map_err(|e| to_js_error(&e))?;
        corpus.validate().map_err(|e| to_js_error(&e))?;
        self.loader.inject_corpus(name, corpus);
        Ok(())
    }

    /// Injects a cost model into the in-memory loader.
    ///
    /// # Errors
    ///
    /// Returns an error if the JSON is invalid.
    #[wasm_bindgen(js_name = injectCostModel)]
    pub fn inject_cost_model(&self, name: &str, json_val: JsValue) -> Result<(), JsValue> {
        let model: CostModel = from_value(json_val).map_err(|e| to_js_error(&e))?;
        // CostModel doesn't have a validate() method yet, but serde handles structure.
        self.loader.inject_cost_model(name, model);
        Ok(())
    }

    /// Injects a keycode registry into the in-memory loader.
    ///
    /// # Errors
    ///
    /// Returns an error if the JSON is invalid or fails validation.
    #[wasm_bindgen(js_name = injectKeycodes)]
    pub fn inject_keycodes(&self, name: &str, json_val: JsValue) -> Result<(), JsValue> {
        let reg: KeycodeRegistry = from_value(json_val).map_err(|e| to_js_error(&e))?;
        reg.validate().map_err(|e| to_js_error(&e))?;
        self.loader.inject_keycodes(name, reg);
        Ok(())
    }

    /// Analyzes a layout using the injected assets and an optional rubric.
    ///
    /// # Errors
    ///
    /// Returns an error if any assets are missing or scoring fails.
    #[wasm_bindgen(js_name = analyzeLayout)]
    pub async fn analyze_layout(
        &self,
        keyboard_name: String,
        corpus_name: String,
        cost_model_name: String,
        layout_val: JsValue,
        rubric_val: JsValue,
    ) -> Result<JsValue, JsValue> {
        let layout: Layout = from_value(layout_val).map_err(|e| to_js_error(&e))?;
        let rubric: Rubric = if rubric_val.is_null() || rubric_val.is_undefined() {
            Rubric::default()
        } else {
            from_value(rubric_val).map_err(|e| to_js_error(&e))?
        };

        let kb = self
            .loader
            .load::<KeyboardDefinition>(&keyboard_name)
            .await
            .map_err(|e| to_js_error(&e))?;

        let corpus = self
            .loader
            .load_corpus(&[CorpusSource {
                id: corpus_name,
                weight: 1.0,
                hash: None,
            }])
            .await
            .map_err(|e| to_js_error(&e))?;

        let cost_model = self
            .loader
            .load::<CostModel>(&cost_model_name)
            .await
            .map_err(|e| to_js_error(&e))?;

        // Create Keyboard from Definition
        let keyboard = keyforge_model::Keyboard::new(
            kb.geometry.keys.clone(),
            kb.geometry.home_row,
            kb.meta.name.clone(),
        )
        .map_err(|e| to_js_error(&e))?;

        let engine = keyforge_physics::EngineFactory::new_generic(
            keyforge_physics::EngineCompilationContext {
                keyboard: &keyboard,
                corpus: &corpus,
                rubric: &rubric,
                cost_model: &cost_model,
            },
        )
        .map_err(|e| to_js_error(&e))?;

        let report = engine.analyze(&layout).map_err(|e| to_js_error(&e))?;

        to_value(&report).map_err(|e| to_js_error(&e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_inject_keyboard() {
        let engine = KeyforgeEngine::new();
        let kb_json = r#"{
            "meta": { "name": "Test" },
            "geometry": { "keys": [], "prime_slots": [], "med_slots": [], "low_slots": [], "home_row": 0 },
            "layouts": {}
        }"#;
        let val: serde_json::Value = serde_json::from_str(kb_json).unwrap();
        let js_val = to_value(&val).unwrap();

        // Should fail validation (empty keys)
        assert!(engine.inject_keyboard("test".into(), js_val).is_err());
    }
}
