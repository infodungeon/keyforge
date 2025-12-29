use keyforge_model::Layout;
use super::traits::MutationAction;

#[derive(Debug, Clone)]
pub struct SearchState {
    current_layout: Layout,
    pub current_score: i64,
    pos_map: Vec<u16>, 

    best_layout: Layout,
    pub best_score: i64,

    pub temperature: f32,
}

impl SearchState {
    pub fn new(layout: Layout, score: i64, start_temp: f32) -> Self {
        // INVARIANT: Key count must fit in u16 to use 65535 as sentinel
        assert!(layout.keys.len() < 65535, "Key count exceeds u16 limit");

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

    pub fn layout(&self) -> &Layout {
        &self.current_layout
    }

    pub fn pos_map(&self) -> &[u16] {
        &self.pos_map
    }

    pub fn best_layout(&self) -> &Layout {
        &self.best_layout
    }

    pub fn update_best(&mut self) {
        self.best_score = self.current_score;
        self.best_layout = self.current_layout.clone();
    }

    pub fn apply_mutation(&mut self, action: MutationAction) {
        match action {
            MutationAction::Swap(a, b) => {
                self.current_layout.keys.swap(a.0, b.0);
                let code_a = self.current_layout.keys[a.0] as usize;
                let code_b = self.current_layout.keys[b.0] as usize;
                if code_a < self.pos_map.len() { self.pos_map[code_a] = a.0 as u16; }
                if code_b < self.pos_map.len() { self.pos_map[code_b] = b.0 as u16; }
            }
            MutationAction::GroupSwap(a, b, c) => {
                // A -> B, B -> C, C -> A
                let temp = self.current_layout.keys[c.0];
                self.current_layout.keys[c.0] = self.current_layout.keys[b.0];
                self.current_layout.keys[b.0] = self.current_layout.keys[a.0];
                self.current_layout.keys[a.0] = temp;

                let code_a = self.current_layout.keys[a.0] as usize;
                let code_b = self.current_layout.keys[b.0] as usize;
                let code_c = self.current_layout.keys[c.0] as usize;

                if code_a < self.pos_map.len() { self.pos_map[code_a] = a.0 as u16; }
                if code_b < self.pos_map.len() { self.pos_map[code_b] = b.0 as u16; }
                if code_c < self.pos_map.len() { self.pos_map[code_c] = c.0 as u16; }
            }
        }
    }

    pub fn reheat_from_best(&mut self, start_temp: f32, reheat_factor: f32) {
        self.temperature = start_temp * reheat_factor;
        self.current_layout = self.best_layout.clone();
        self.current_score = self.best_score;

        // Rebuild pos_map for best layout
        self.pos_map.fill(65535);
        for (i, &code) in self.current_layout.keys.iter().enumerate() {
            if (code as usize) < self.pos_map.len() {
                self.pos_map[code as usize] = i as u16;
            }
        }
    }
}
