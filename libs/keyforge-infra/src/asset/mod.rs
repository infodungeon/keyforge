// libs/keyforge-infra/src/asset/mod.rs

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

/// Filesystem-based asset provider for local development and testing.
pub mod fs_provider;
/// High-level orchestration for fetching and ensuring asset presence.
pub mod manager;
/// Tiered caching provider for high-performance asset reads.
pub mod caching_provider;
/// Distributed asset provider backed by an external data store (e.g., Valkey).
pub mod valkey_provider;

pub use valkey_provider::ValkeyProvider;
