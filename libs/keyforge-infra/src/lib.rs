// libs/keyforge-infra/src/lib.rs

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

//! # `KeyForge` Infrastructure
//!
//! Infrastructure and cross-cutting concerns for `KeyForge`. This crate
//! provides utilities for networking, filesystem operations, and asset
//! management.

/// Asset management and providers (filesystem, caching, Valkey).
pub mod asset;
/// Common configuration structures.
pub mod config;
/// Infrastructure-specific error types.
pub mod error;
/// Filesystem operations, workspace initialization, and locking.
pub mod fs;
/// Network clients, synchronization, and distributed coordination.
pub mod net;
/// Common utility functions and parsers.
pub mod util;

pub use error::{InfraError, InfraResult};

// Re-exports
pub use fs::init;
pub use fs::init::{initialize_workspace, InitMode};
pub use fs::io::{atomic_write, read_to_string_limited};
pub use fs::listing;
pub use fs::lock::WorkspaceLock;
pub use fs::paths::resolve_root;

pub use net::client::HiveClient;
pub use net::distributed::{DistributedCoordinator, ValkeyDistributedCoordinator};
pub use net::network::{ensure_corpus_bundle, ensure_cost_matrix, ensure_file};
pub use net::sync::{bootstrap_essentials, generate_manifest, run_sync, ServerManifest, SyncStats};

pub use asset::caching_provider::CachingProvider;
pub use asset::fs_provider::FsProvider;
pub use asset::manager::AssetManager;
pub use asset::ValkeyProvider; // ADDED

pub use keyforge_model::loader::AssetLoader;

pub use util::common::{calculate_file_hash, load_keycode_registry, sanitize_filename};

include!(concat!(env!("OUT_DIR"), "/build_info.rs"));

/// Returns the build information (Git hash and build date) injected during compilation.
#[must_use]
pub fn get_build_info() -> (&'static str, &'static str) {
    (GIT_HASH, BUILD_DATE)
}
