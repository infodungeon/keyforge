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
use keyforge_infra::listing;

#[test]
fn list_cost_matrices_includes_mpk_zst_and_excludes_non_cost_weights() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();

    // Create expected directory structure
    std::fs::create_dir_all(root.join("system/weights")).unwrap();
    std::fs::create_dir_all(root.join("system/keyboards")).unwrap();
    std::fs::create_dir_all(root.join("user/weights")).unwrap();

    // A real cost matrix can be either json or mpk.zst
    std::fs::write(root.join("system/weights/cost_matrix.mpk.zst"), b"dummy").unwrap();
    std::fs::write(root.join("user/weights/custom.json"), b"{\"entries\":[]}").unwrap();

    // These are *not* cost matrices and must be excluded (by being in a different dir)
    std::fs::write(root.join("system/keyboards/ortho_split.mpk.zst"), b"dummy").unwrap();
    std::fs::write(root.join("system/keyboards/row_stagger.mpk.zst"), b"dummy").unwrap();
    std::fs::write(root.join("system/keyboards/testing.mpk.zst"), b"dummy").unwrap();

    let list = listing::list_cost_matrices(root).unwrap();

    assert!(list.contains(&"cost_matrix".to_string()));
    assert!(list.contains(&"custom".to_string()));
    assert!(!list.contains(&"ortho_split".to_string()));
    assert!(!list.contains(&"row_stagger".to_string()));
    assert!(!list.contains(&"testing".to_string()));
}
