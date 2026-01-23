// libs/keyforge-export/src/viz/mod.rs

pub mod physics;

#[derive(Debug, Clone)]
pub struct VizTheme {
    pub bg_color: String,
    pub key_fill: String,
    pub key_stroke: String,
    pub key_stroke_width: f32,
    pub home_row_fill: String,
    pub text_fill: String,
    pub font_size: f32,
}

impl Default for VizTheme {
    fn default() -> Self {
        Self {
            bg_color: "#f8f9fa".into(),
            key_fill: "#ffffff".into(),
            key_stroke: "#dee2e6".into(),
            key_stroke_width: 0.5,
            home_row_fill: "#e9ecef".into(),
            text_fill: "#495057".into(),
            font_size: 3.0,
        }
    }
}
