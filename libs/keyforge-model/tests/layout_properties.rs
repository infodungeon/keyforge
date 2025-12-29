use keyforge_model::Layout;
use proptest::prelude::*;
use std::collections::HashSet;

proptest! {
    #[test]
    fn test_layout_uniqueness_invariant(keys in prop::collection::vec(0u16..100, 0..50)) {
        // Check if the input has duplicates
        let mut set = HashSet::new();
        let has_dupes = keys.iter().any(|&k| !set.insert(k));

        let result = Layout::try_from(keys);

        if has_dupes {
            prop_assert!(result.is_err(), "Layout should reject duplicates");
        } else {
            prop_assert!(result.is_ok(), "Layout should accept unique keys");
        }
    }
}
