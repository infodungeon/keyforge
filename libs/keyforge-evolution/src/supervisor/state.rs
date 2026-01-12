// libs/keyforge-evolution/src/supervisor/state.rs

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

use keyforge_model::Layout;
use super::traits::MutationAction;
use crate::errors::EvolutionError;

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
    pub fn new(layout: Layout, score: i64, start_temp: f32) -> Result<Self, EvolutionError> {
        // INVARIANT: Key count must fit in u16 to use 65535 as sentinel
        if layout.keys.len() >= 65535 {
            return Err(EvolutionError::Config("Key count exceeds u16 limit".into()));
        }

        // Optimize pos_map size to actual key range
        let max_code = layout.keys.iter().map(|k| k.0).max().unwrap_or(0);
        let map_size = (max_code as usize) + 1;

        // Initialize for required range
        let mut pos_map = vec![65535u16; map_size];
        for (i, &code) in layout.keys.iter().enumerate() {
            if (code.0 as usize) < map_size {
                pos_map[code.0 as usize] = i as u16;
            }
        }

        Ok(Self {
            current_layout: layout.clone(),
            current_score: score,
            pos_map,
            best_layout: layout,
            best_score: score,
            temperature: start_temp,
        })
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
                let idx_a = usize::from(a);
                let idx_b = usize::from(b);
                self.current_layout.keys.swap(idx_a, idx_b);
                let code_a = self.current_layout.keys[idx_a];
                let code_b = self.current_layout.keys[idx_b];
                if (code_a.0 as usize) < self.pos_map.len() { self.pos_map[code_a.0 as usize] = a.0; }
                if (code_b.0 as usize) < self.pos_map.len() { self.pos_map[code_b.0 as usize] = b.0; }
            }
            MutationAction::GroupSwap(a, b, c) => {
                let idx_a = usize::from(a);
                let idx_b = usize::from(b);
                let idx_c = usize::from(c);
                
                // A -> B, B -> C, C -> A
                let temp = self.current_layout.keys[idx_c];
                self.current_layout.keys[idx_c] = self.current_layout.keys[idx_b];
                self.current_layout.keys[idx_b] = self.current_layout.keys[idx_a];
                self.current_layout.keys[idx_a] = temp;

                let code_a = self.current_layout.keys[idx_a];
                let code_b = self.current_layout.keys[idx_b];
                let code_c = self.current_layout.keys[idx_c];

                if (code_a.0 as usize) < self.pos_map.len() { self.pos_map[code_a.0 as usize] = a.0; }
                if (code_b.0 as usize) < self.pos_map.len() { self.pos_map[code_b.0 as usize] = b.0; }
                if (code_c.0 as usize) < self.pos_map.len() { self.pos_map[code_c.0 as usize] = c.0; }
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
            if (code.0 as usize) < self.pos_map.len() {
                self.pos_map[code.0 as usize] = i as u16;
            }
        }
    }
}
