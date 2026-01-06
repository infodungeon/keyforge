use keyforge_model::{Layout, KeyCode};
use proptest::prelude::*;
use std::collections::HashSet;

proptest! {
    #[test]
    fn test_layout_uniqueness_invariant(keys in prop::collection::vec(0u16..100, 0..50)) {
        let mut set = HashSet::new();
        let has_dupes = keys.iter().any(|&k| !set.insert(k));

        let key_codes: Vec<KeyCode> = keys.into_iter().map(KeyCode).collect();
        let result = Layout::try_from(key_codes);

        if has_dupes {
            prop_assert!(result.is_err());
        } else {
            prop_assert!(result.is_ok());
        }
    }
}