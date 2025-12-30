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
pub mod asset;
pub mod config;
pub mod error;
pub mod fs;
pub mod net;
pub mod util;

pub use error::{InfraError, InfraResult};

// Re-exports
pub use fs::init::{self, InitMode};
pub use fs::io::{atomic_write, read_to_string_limited};
pub use fs::listing;
pub use fs::lock::WorkspaceLock;
pub use fs::paths::resolve_root;

pub use net::client::HiveClient;
pub use net::network::{ensure_corpus_bundle, ensure_cost_matrix, ensure_file};
pub use net::sync::{bootstrap_essentials, generate_manifest, run_sync, ServerManifest, SyncStats};

pub use asset::fs_provider::FsProvider;
pub use asset::manager::AssetManager;
pub use asset::caching_provider::CachingProvider;
// Re-export from model now
pub use keyforge_core::loader::{AssetLoader, RawCostData};

pub use util::common::{
    calculate_file_hash, generate_cost_profile, load_keycode_registry, sanitize_filename,
};
pub use util::layout_parser::parse_layout_string_permissive_cached;
pub mod repo;
pub use repo::user_repo::UserRepo;

include!(concat!(env!("OUT_DIR"), "/build_info.rs"));

pub fn get_build_info() -> (&'static str, &'static str) {
    (GIT_HASH, BUILD_DATE)
}
