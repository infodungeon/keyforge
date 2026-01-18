// libs/keyforge-infra/src/net/mod.rs

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

/// Specialized HTTP client for communicating with the `KeyForge` Hive.
pub mod client;
/// High-level network operations for asset fetching and verification.
pub mod network;
/// Workspace synchronization logic for matching local data with the server.
pub mod sync;
/// Distributed orchestration and coordination (e.g., via Valkey).
pub mod distributed; 
