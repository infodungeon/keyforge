// Copyright (c) 2025 KeyForge Contributors
//
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