use keyforge_model::Layout;

#[derive(Debug, Clone)]
pub struct SearchState {
    pub current_layout: Layout,
    pub current_score: i64,
    pub pos_map: Vec<u16>, // Changed from [u8; 256]

    pub best_layout: Layout,
    pub best_score: i64,

    pub temperature: f32,
}

impl SearchState {
    pub fn new(layout: Layout, score: i64, start_temp: f32) -> Self {
        // Initialize for full u16 range
        let mut pos_map = vec![65535u16; 65536];
        for (i, &code) in layout.keys.iter().enumerate() {
            pos_map[code as usize] = i as u16;
        }

        Self {
            current_layout: layout.clone(),
            current_score: score,
            pos_map,
            best_layout: layout,
            best_score: score,
            temperature: start_temp,
        }
    }
}
