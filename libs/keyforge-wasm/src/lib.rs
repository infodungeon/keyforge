// libs/keyforge-wasm/src/lib.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You    may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use keyforge_adapter::loader::{AssetLoader, InMemoryLoader};
use keyforge_model::config::CorpusSource;
use keyforge_model::geometry::KeyboardDefinition;
use keyforge_model::keycodes::KeycodeRegistry;
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
    code: String,
    message: String,
}

impl From<keyforge_model::error::ForgeError> for WasmError {
    fn from(e: keyforge_model::error::ForgeError) -> Self {
        use keyforge_model::error::ForgeError;
        let (code, msg) = match e {
            ForgeError::Io(s) => ("IO_ERROR", s),
            ForgeError::Serde(s) | ForgeError::Serialization(s) => ("SERIALIZATION_ERROR", s),
            ForgeError::Physics(pe) => ("PHYSICS_VIOLATION", pe.to_string()),
            ForgeError::PhysicsCompute(s) => ("COMPUTE_ERROR", s),
            ForgeError::Evolution(s) => ("EVOLUTION_ERROR", s),
            ForgeError::Persistence(s) => ("PERSISTENCE_ERROR", s),
            ForgeError::Validation(s) => ("VALIDATION_ERROR", s),
            ForgeError::NotFound(s) => ("NOT_FOUND", s),
            ForgeError::Internal(s) => ("INTERNAL_ERROR", s),
            ForgeError::InvalidData(s) => ("INVALID_DATA", s),
            ForgeError::Config(s) => ("CONFIG_ERROR", s),
            ForgeError::Projection(s) => ("PROJECTION_ERROR", s),
            ForgeError::Model(me) => ("MODEL_ERROR", me.to_string()),
            ForgeError::Infrastructure(s) => ("INFRASTRUCTURE_ERROR", s),
        };

        Self {
            kind: "KeyForgeError".into(),
            code: code.into(),
            message: msg,
        }
    }
}

impl From<String> for WasmError {
    fn from(s: String) -> Self {
        Self {
            kind: "ValidationError".into(),
            code: "VALIDATION_FAILED".into(),
            message: s,
        }
    }
}

fn to_js_error(e: impl Into<WasmError>) -> JsValue {
    let err: WasmError = e.into();
    to_value(&err).unwrap_or_else(|_| JsValue::from_str(&err.message))
}

fn map_serde_error(e: &serde_wasm_bindgen::Error) -> JsValue {
    let err = WasmError {
        kind: "SerializationError".into(),
        code: "JS_CONVERSION".into(),
        message: e.to_string(),
    };
    to_value(&err).unwrap_or_else(|_| JsValue::from_str(&err.message))
}

fn map_physics_error(e: &keyforge_physics::PhysicsError) -> JsValue {
    let err = WasmError {
        kind: "PhysicsError".into(),
        code: "PHYSICS_VIOLATION".into(),
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
    /// Returns an error if the JSON value cannot be deserialized into a `KeyboardDefinition`.
    #[wasm_bindgen(js_name = injectKeyboard)]
    pub fn inject_keyboard(&self, name: &str, json_val: JsValue) -> Result<(), JsValue> {
        let kb: KeyboardDefinition = from_value(json_val).map_err(|e| map_serde_error(&e))?;
        self.loader.inject(name, kb);
        Ok(())
    }

    /// Injects a corpus into the in-memory loader.
    ///
    /// # Errors
    ///
    /// Returns an error if the JSON value cannot be deserialized into a `Corpus`.
    #[wasm_bindgen(js_name = injectCorpus)]
    pub fn inject_corpus(&self, name: &str, json_val: JsValue) -> Result<(), JsValue> {
        let corpus: Corpus = from_value(json_val).map_err(|e| map_serde_error(&e))?;
        self.loader.inject(name, corpus);
        Ok(())
    }

    /// Injects a cost model into the in-memory loader.
    ///
    /// # Errors
    ///
    /// Returns an error if the JSON value cannot be deserialized into a `CostModel`.
    #[wasm_bindgen(js_name = injectCostModel)]
    pub fn inject_cost_model(&self, name: &str, json_val: JsValue) -> Result<(), JsValue> {
        let model_dto: keyforge_protocol::CostModelDto =
            from_value(json_val).map_err(|e| map_serde_error(&e))?;
        self.loader.inject(name, model_dto);
        Ok(())
    }

    /// Injects a keycode registry into the in-memory loader.
    ///
    /// # Errors
    ///
    /// Returns an error if the JSON value cannot be deserialized into a `KeycodeRegistry`.
    #[wasm_bindgen(js_name = injectKeycodes)]
    pub fn inject_keycodes(&self, name: &str, json_val: JsValue) -> Result<(), JsValue> {
        let reg: KeycodeRegistry = from_value(json_val).map_err(|e| map_serde_error(&e))?;
        self.loader.inject(name, reg);
        Ok(())
    }

    /// Analyzes a layout using the injected assets and an optional rubric.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The layout or rubric cannot be deserialized.
    /// - The requested keyboard, corpus, or cost model is not found in the loader.
    /// - There is a failure in keyboard instantiation or engine compilation.
    /// - The physics analysis fails.
    #[wasm_bindgen(js_name = analyzeLayout)]
    pub async fn analyze_layout(
        &self,
        keyboard_name: String,
        corpus_name: String,
        cost_model_name: String,
        layout_val: JsValue,
        rubric_val: JsValue,
    ) -> Result<JsValue, JsValue> {
        let layout: Layout = from_value(layout_val).map_err(|e| map_serde_error(&e))?;
        let rubric: Rubric = if rubric_val.is_null() || rubric_val.is_undefined() {
            Rubric::default()
        } else {
            from_value(rubric_val).map_err(|e| map_serde_error(&e))?
        };

        let kb_def = self
            .loader
            .load::<KeyboardDefinition>(&keyboard_name)
            .await
            .map_err(to_js_error)?;

        let corpus = self
            .loader
            .load_corpus(&[CorpusSource {
                id: corpus_name,
                weight: 1.0,
                hash: None,
            }])
            .await
            .map_err(to_js_error)?;

        let cost_model_dto = self
            .loader
            .load::<keyforge_protocol::CostModelDto>(&cost_model_name)
            .await
            .map_err(to_js_error)?;
        let cost_model: Arc<CostModel> = Arc::new((*cost_model_dto).clone().into());

        let keyboard = Arc::new(
            keyforge_model::Keyboard::new(
                kb_def.geometry.keys().to_vec(),
                kb_def.geometry.home_row(),
                kb_def.meta.kb_type.clone(),
            )
            .map_err(to_js_error)?,
        );

        let engine = keyforge_physics::EngineFactory::new_generic(
            &keyforge_physics::EngineCompilationContext {
                keyboard,
                corpus: corpus.clone(),
                rubric: Arc::new(rubric.clone()),
                cost_model: cost_model.clone(),
                engine_config: keyforge_model::config::EngineConfig::default(),
            },
        )
        .map_err(|e| map_physics_error(&e))?;

        let report = engine.analyze(&layout).map_err(|e| map_physics_error(&e))?;

        to_value(&report).map_err(|e| map_serde_error(&e))
    }
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_inject_keyboard() -> anyhow::Result<()> {
        let engine = KeyforgeEngine::new();
        let kb_json = r#"{
            "meta": { "name": "Test" },
            "geometry": { "keys": [], "prime_slots": [], "med_slots": [], "low_slots": [], "home_row": 0 },
            "layouts": {}
        }"#;
        let val: serde_json::Value = serde_json::from_str(kb_json)?;
        let js_val = to_value(&val).map_err(|e| anyhow::anyhow!("{}", e))?;

        assert!(engine.inject_keyboard("test", js_val).is_err());
        Ok(())
    }
}
