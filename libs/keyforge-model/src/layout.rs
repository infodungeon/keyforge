use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    // Optimization: Store up to 64 keys inline on the stack.
    // Most split/ortho boards are < 64 keys.
    // Larger boards will spill to heap automatically.
    pub keys: SmallVec<[u16; 64]>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_helpers() {
        let l = Layout::new(vec![1, 2, 3]);
        assert_eq!(l.len(), 3);
        assert!(!l.is_empty());

        let empty = Layout::new(vec![]);
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);
    }
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
