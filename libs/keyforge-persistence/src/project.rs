// libs/keyforge-persistence/src/project.rs

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

//! Project metadata and persistence definitions.
//!
//! This module re-exports domain configuration types from `keyforge-model`
//! and provides persistence-specific aliases and extensions.

pub use keyforge_model::{Config, ProjectMeta};

/// Alias for `Config` when used in a persistable project context.
pub type Project = Config;

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;

    #[test]
    fn test_project_alias_consistency() {
        let project = Project::default();
        assert_eq!(project.keyboard, "ortho_30");
        assert!(!project.corpora.is_empty());
    }
}
