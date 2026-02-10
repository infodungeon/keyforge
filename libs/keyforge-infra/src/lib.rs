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
//! Shared infrastructure, observability, logging, and common management services
//! for `KeyForge` components.

#![warn(missing_docs)]

/// Asset management and loading providers.
pub mod asset;
/// Configuration structures and environment variable handling.
pub mod config;
/// Error and Result types for infrastructure operations.
pub mod error;
/// Filesystem abstractions and utilities.
pub mod fs;
/// Hardware discovery and provider traits.
pub mod hardware;
/// Network client and protocol adapters.
pub mod net;
/// Shared utility functions and common logic.
pub mod util;

pub use asset::fs_provider::FsProvider;
pub use asset::manager::AssetManager;
pub use config::CommonConfig;
pub use error::{InfraError, InfraResult};
pub use fs::paths::resolve_root;
pub use keyforge_adapter::loader::LoaderResult;
pub use net::client::HiveClient;
pub use net::sync::{bootstrap_essentials, run_sync, SyncStats};

/// Returns the build-time Git hash and build date of the infrastructure library.
#[must_use]
pub fn get_build_info() -> (&'static str, &'static str) {
    (
        option_env!("GIT_HASH").unwrap_or("unknown"),
        option_env!("BUILD_DATE").unwrap_or("unknown"),
    )
}
