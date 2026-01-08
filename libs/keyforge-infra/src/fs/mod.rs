// libs/keyforge-infra/src/fs/mod.rs

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

/// Workspace and system directory initialization.
pub mod init;
/// Atomic file writes and limited reading.
pub mod io;
/// Asset discovery and directory listing.
pub mod listing;
/// Process-level file locking for the workspace.
pub mod lock;
/// Path resolution and workspace root discovery.
pub mod paths;
