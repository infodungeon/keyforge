use keyforge_model::AnalysisReport;
use keyforge_protocol::geometry::KeyboardGeometry;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub layout_name: String,
    pub score: AnalysisReport,
    pub geometry: KeyboardGeometry,
    pub heatmap: Vec<f32>,
    pub penalty_map: Vec<f32>,
}
