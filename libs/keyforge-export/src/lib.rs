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

//! # KeyForge Export
//!
//! Provides functionality for exporting keyboard layouts to various 
//! firmware and configuration formats.

/// QMK firmware configuration exporter.
pub mod qmk;
/// VIA firmware (JSON) exporter.
pub mod via;
/// ZMK firmware (devicetree) exporter.
pub mod zmk;

use anyhow::Result;

/// A trait for types that can export keymaps to varied keyboard firmware formats.
pub trait Exporter {
    /// Generates the source code or configuration for a specific firmware format.
    ///
    /// - `layout_name`: The human-readable name of the layout.
    /// - `keys`: A slice of key labels or tokens to be exported.
    fn generate(&self, layout_name: &str, keys: &[String]) -> Result<String>;
}
