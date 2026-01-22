// libs/keyforge-model/src/utils/mod.rs

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

//! Internal utilities for the keyforge-model crate.

pub mod json;

/// Helper for serde `skip_serializing_if` to satisfy `ts-rs` parser.
///
/// This is required because `ts-rs` sometimes struggles to infer optionality
/// from `Option<T>` alone when generating interaction definitions.
#[allow(clippy::ref_option)]
pub(crate) fn is_none<T>(option: &Option<T>) -> bool {
    option.is_none()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_none() {
        let none: Option<i32> = None;
        let some = Some(10);
        assert!(is_none(&none));
        assert!(!is_none(&some));
    }
}
