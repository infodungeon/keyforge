// apps/keyforge-hive/src/models.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use keyforge_model::AnalysisReport;
use keyforge_model::KeyboardGeometry;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub layout_name: String,
    pub score: AnalysisReport,
    pub geometry: KeyboardGeometry,
    pub heatmap: Vec<f32>,
    pub penalty_map: Vec<f32>,
}
