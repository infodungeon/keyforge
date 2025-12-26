use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    // Optimization: Store up to 64 keys inline on the stack.
    // Most split/ortho boards are < 64 keys.
    // Larger boards will spill to heap automatically.
    pub keys: SmallVec<[u16; 64]>,
}

impl Layout {
    pub fn new(keys: Vec<u16>) -> Self {
        Self {
            keys: SmallVec::from_vec(keys),
        }
    }

    pub fn len(&self) -> usize {
        self.keys.len()
    }

    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }
}
