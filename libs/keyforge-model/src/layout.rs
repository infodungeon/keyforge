use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LayoutError {
    #[error("Layout contains duplicate keys")]
    DuplicateKeys,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    // Optimization: Store up to 64 keys inline on the stack.
    pub keys: SmallVec<[u16; 64]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct KeyIndex(pub usize);

impl From<usize> for KeyIndex {
    fn from(i: usize) -> Self {
        Self(i)
    }
}

impl From<KeyIndex> for usize {
    fn from(k: KeyIndex) -> Self {
        k.0
    }
}

impl Layout {
    /// Creates a layout without validation.
    /// Use `try_from` for safe construction.
    pub fn new_unchecked(keys: Vec<u16>) -> Self {
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

impl TryFrom<Vec<u16>> for Layout {
    type Error = LayoutError;

    fn try_from(keys: Vec<u16>) -> Result<Self, Self::Error> {
        // Validation Logic: Check for duplicates
        for i in 0..keys.len() {
            for j in (i + 1)..keys.len() {
                if keys[i] == keys[j] {
                    return Err(LayoutError::DuplicateKeys);
                }
            }
        }

        Ok(Self {
            keys: SmallVec::from_vec(keys),
        })
    }
}
