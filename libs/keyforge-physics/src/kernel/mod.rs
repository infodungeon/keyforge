// libs/keyforge-physics/src/kernel/mod.rs

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

pub mod compiler;
pub mod compute;
pub mod mechanics;
pub mod stages;
pub mod types;

use self::types::{ColIndex, FingerIndex, HandIndex, KeyCode, RowIndex, Score};

use std::collections::HashMap;

use std::sync::Arc;

/// Physical properties and pre-calculated distances for a keyboard.
#[derive(Debug, Clone)]
pub struct GeometryData {
    pub(crate) hands: Arc<[HandIndex]>,
    pub fingers: Arc<[FingerIndex]>,
    pub(crate) rows: Arc<[RowIndex]>,
    pub(crate) cols: Arc<[ColIndex]>,
    pub(crate) cost_matrix: Arc<[Score]>,
    pub(crate) dist_matrix: Arc<[Score]>,
    pub(crate) key_home_distances: Arc<[Score]>,
    pub(crate) key_costs: Arc<[Score]>,
}

/// Statistics and frequencies for characters and sequences.
#[derive(Debug, Clone)]
pub struct CorpusData {
    pub(crate) char_freqs: Arc<[u64]>,
    pub(crate) bigram_starts: Arc<[usize]>,
    pub(crate) bigram_others: Arc<[KeyCode]>,
    pub(crate) bigram_freqs: Arc<[u32]>,
    pub(crate) bigram_rev_starts: Arc<[usize]>,
    pub(crate) bigram_rev_others: Arc<[KeyCode]>,
    pub(crate) bigram_rev_freqs: Arc<[u32]>,
    pub(crate) trigram_starts: Arc<[usize]>,
    pub(crate) trigram_others1: Arc<[KeyCode]>,
    pub(crate) trigram_others2: Arc<[KeyCode]>,
    pub(crate) trigram_freqs: Arc<[u32]>,
    pub(crate) trigram_mid_starts: Arc<[usize]>,
    pub(crate) trigram_mid_others1: Arc<[KeyCode]>,
    pub(crate) trigram_mid_others2: Arc<[KeyCode]>,
    pub(crate) trigram_mid_freqs: Arc<[u32]>,
    pub(crate) trigram_end_starts: Arc<[usize]>,
    pub(crate) trigram_end_others1: Arc<[KeyCode]>,
    pub(crate) trigram_end_others2: Arc<[KeyCode]>,
    pub(crate) trigram_end_freqs: Arc<[u32]>,
}

/// Compiled, high-performance context used by the physics engine for scoring.
#[derive(Debug, Clone)]
pub struct EngineContext {
    pub(crate) key_count: usize,
    pub(crate) geometry: GeometryData,
    pub(crate) corpus: CorpusData,
    pub(crate) all_bigrams: Arc<[(u16, u16, u32)]>,
    pub(crate) all_trigrams: Arc<[(u16, u16, u16, u32)]>,
    pub(crate) penalty_redirect: Score,
    pub(crate) bonus_roll: Score,
    pub(crate) bonus_roll_out: Score,
    /// Custom modifiers for specific key sequences (Bigrams).
    pub(crate) sequence_modifiers: Arc<HashMap<(u16, u16), Score>>,
}

/// A "Parameter Object" grouping all data needed for a scoring pass.
/// Mandated by the `KeyForge` Engineering Manifesto to prevent argument-swapping.
#[derive(Debug)]
pub struct EvaluationContext<'a> {
    /// High-performance compiled context.
    pub engine: &'a EngineContext,
    /// Fast position lookup map for the current layout.
    pub pos_map: &'a self::compute::state::PosMap<'a>,
}

impl EngineContext {
    /// Verifies the internal consistency of the context data structures.
    ///
    /// # Errors
    /// Returns `PhysicsError::Config` if any vector length mismatches `key_count`.
    pub fn verify(&self) -> Result<(), crate::error::PhysicsError> {
        let kc = self.key_count;
        let g = &self.geometry;

        if g.hands.len() != kc || g.fingers.len() != kc || g.rows.len() != kc || g.cols.len() != kc
        {
            return Err(crate::error::PhysicsError::Config(
                "Geometry vector size mismatch".into(),
            ));
        }
        if g.cost_matrix.len() != kc * kc || g.dist_matrix.len() != kc * kc {
            return Err(crate::error::PhysicsError::Config(
                "Matrix size mismatch".into(),
            ));
        }
        if g.key_home_distances.len() != kc || g.key_costs.len() != kc {
            return Err(crate::error::PhysicsError::Config(
                "Static cost vector size mismatch".into(),
            ));
        }
        if self.corpus.char_freqs.len() != keyforge_model::constants::MAX_KEYCODE_SPACE {
            return Err(crate::error::PhysicsError::Config(format!(
                "Char freqs must be {}",
                keyforge_model::constants::MAX_KEYCODE_SPACE
            )));
        }
        Ok(())
    }
}
