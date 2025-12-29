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

    #[test]
    fn test_layout_clone_eq() {
        let l1 = Layout::new(vec![1, 2, 3]);
        let l2 = l1.clone();
        assert_eq!(l1, l2);
        
        let l3 = Layout::new(vec![1, 2, 4]);
        assert_ne!(l1, l3);
    }

    #[test]
    fn test_layout_debug() {
        let l = Layout::new(vec![10, 20]);
        let s = format!("{:?}", l);
        assert!(s.contains("Layout"));
        assert!(s.contains("10"));
        assert!(s.contains("20"));
    }

    #[test]
    fn test_layout_serde() {
        let l = Layout::new(vec![1, 2, 3]);
        let json = serde_json::to_string(&l).expect("Failed to serialize");
        let l_de: Layout = serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(l, l_de);
    }
}
