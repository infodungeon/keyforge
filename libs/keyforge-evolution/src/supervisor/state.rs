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

use super::traits::MutationAction;
use crate::errors::EvolutionError;
use keyforge_model::types::{ScalingFactor, Temperature};
use keyforge_model::Layout;

#[derive(Debug, Clone)]
pub struct SearchState {
    current_layout: Layout,
    pub current_score: i64,
    pos_map: Vec<keyforge_model::KeyIndex>,

    best_layout: Layout,
    pub best_score: i64,

    pub temperature: Temperature,
}

impl SearchState {
    /// Creates a new `SearchState` from an initial layout and score.
    ///
    /// # Errors
    /// Returns `EvolutionError::Config` if the layout key count exceeds the internal u16 limit.
    #[allow(clippy::cast_possible_truncation)]
    pub fn new(
        layout: Layout,
        score: i64,
        start_temp: Temperature,
    ) -> Result<Self, EvolutionError> {
        // INVARIANT: Key count must fit in u16 to use 65535 as sentinel
        if layout.len() >= 65535 {
            return Err(EvolutionError::Config("Key count exceeds u16 limit".into()));
        }

        // Optimize pos_map size to actual key range
        let max_code = layout.keys().iter().map(|k| k.raw()).max().unwrap_or(0);
        let map_size = (max_code as usize) + 1;

        // Initialize for required range
        let mut pos_map = vec![keyforge_model::types::KeyIndex::SENTINEL; map_size];
        for (i, &code) in layout.keys().iter().enumerate() {
            if (code.raw() as usize) < map_size {
                pos_map[code.raw() as usize] = keyforge_model::types::KeyIndex::new(i as u16);
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

    /// Returns the current layout under evaluation.
    #[must_use]
    pub fn layout(&self) -> &Layout {
        &self.current_layout
    }

    /// Returns the current position map.
    #[must_use]
    pub fn pos_map(&self) -> &[keyforge_model::types::KeyIndex] {
        &self.pos_map
    }

    /// Returns the best layout found so far in the search.
    #[must_use]
    pub fn best_layout(&self) -> &Layout {
        &self.best_layout
    }

    pub fn update_best(&mut self) {
        self.best_score = self.current_score;
        self.best_layout = self.current_layout.clone();
    }

    /// Applies a mutation to the current search state.
    ///
    /// # Errors
    /// Returns `EvolutionError::Internal` if key lookups or swaps fail, usually indicating
    /// state inconsistency.
    #[allow(clippy::similar_names)]
    pub fn apply_mutation(&mut self, action: MutationAction) -> Result<(), EvolutionError> {
        match action {
            MutationAction::Swap(a, b) => {
                self.current_layout.swap(a, b).map_err(|e| {
                    EvolutionError::Internal(format!("Swap failed in SearchState: {e}"))
                })?;
                let code_a = self.current_layout.get(a).ok_or_else(|| {
                    EvolutionError::Internal(format!("Key {a:?} missing after swap"))
                })?;
                let code_b = self.current_layout.get(b).ok_or_else(|| {
                    EvolutionError::Internal(format!("Key {b:?} missing after swap"))
                })?;

                // Safety: Update pos_map only if within tracked range
                let idx_ca = code_a.raw() as usize;
                let idx_cb = code_b.raw() as usize;
                if idx_ca < self.pos_map.len() {
                    self.pos_map[idx_ca] = a;
                }
                if idx_cb < self.pos_map.len() {
                    self.pos_map[idx_cb] = b;
                }
            }
            MutationAction::GroupSwap(a, b, c) => {
                // A -> B, B -> C, C -> A
                let code_a = self.current_layout.get(a).ok_or_else(|| {
                    EvolutionError::Internal(format!("Key {a:?} missing before group swap"))
                })?;
                let code_b = self.current_layout.get(b).ok_or_else(|| {
                    EvolutionError::Internal(format!("Key {b:?} missing before group swap"))
                })?;
                let code_c = self.current_layout.get(c).ok_or_else(|| {
                    EvolutionError::Internal(format!("Key {c:?} missing before group swap"))
                })?;

                self.current_layout.set(b, code_a).map_err(|e| {
                    EvolutionError::Internal(format!("Set B failed in group swap: {e}"))
                })?;
                self.current_layout.set(c, code_b).map_err(|e| {
                    EvolutionError::Internal(format!("Set C failed in group swap: {e}"))
                })?;
                self.current_layout.set(a, code_c).map_err(|e| {
                    EvolutionError::Internal(format!("Set A failed in group swap: {e}"))
                })?;

                let idx_ca = code_a.raw() as usize;
                let idx_cb = code_b.raw() as usize;
                let idx_cc = code_c.raw() as usize;

                if idx_ca < self.pos_map.len() {
                    self.pos_map[idx_ca] = b;
                }
                if idx_cb < self.pos_map.len() {
                    self.pos_map[idx_cb] = c;
                }
                if idx_cc < self.pos_map.len() {
                    self.pos_map[idx_cc] = a;
                }
            }
        }
        Ok(())
    }

    pub fn reheat_from_best(&mut self, start_temp: Temperature, reheat_factor: ScalingFactor) {
        self.temperature = Temperature::new(start_temp.raw() * reheat_factor.raw());
        self.current_layout = self.best_layout.clone();
        self.current_score = self.best_score;
    }
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;
    use keyforge_model::types::{KeyCode, KeyIndex};

    #[test]
    fn test_search_state_mutation_swap() {
        let layout = Layout::new_unchecked(vec![KeyCode::new(10), KeyCode::new(20)]);
        let mut state = SearchState::new(layout, 100, Temperature::new(1.0)).unwrap();

        state
            .apply_mutation(MutationAction::Swap(KeyIndex::new(0), KeyIndex::new(1)))
            .unwrap();

        assert_eq!(state.layout().keys()[0], KeyCode::new(20));
        assert_eq!(state.layout().keys()[1], KeyCode::new(10));
        assert_eq!(state.pos_map()[20], KeyIndex::new(0));
        assert_eq!(state.pos_map()[10], KeyIndex::new(1));
    }

    #[test]
    fn test_reheat_logic() {
        let layout = Layout::new_unchecked(vec![KeyCode::new(10)]);
        let mut state = SearchState::new(layout, 100, Temperature::new(0.1)).unwrap();
        state.best_score = 50; // Manual override for test

        state.reheat_from_best(Temperature::new(1.0), ScalingFactor::new(0.5));

        assert!((state.temperature.raw() - 0.5).abs() < f32::EPSILON);
        assert_eq!(state.current_score, 50);
    }

    #[test]
    fn test_state_reheat_zero_temp() {
        let layout = Layout::new_unchecked(vec![KeyCode::new(10)]);
        let mut state = SearchState::new(layout, 100, Temperature::new(0.1)).unwrap();
        state.reheat_from_best(Temperature::new(0.0), ScalingFactor::new(0.5));
        assert!((state.temperature.raw() - 0.0).abs() < f32::EPSILON);
    }
}