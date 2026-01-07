// libs/keyforge-infra/src/lib.rs

pub mod asset;
pub mod config;
pub mod error;
pub mod fs;
pub mod net;
pub mod util;

pub use error::{InfraError, InfraResult};

// Re-exports
pub use fs::init;
pub use fs::init::{InitMode, initialize_workspace};
pub use fs::io::{atomic_write, read_to_string_limited};
pub use fs::listing;
pub use fs::lock::WorkspaceLock;
pub use fs::paths::resolve_root;

pub use net::client::HiveClient;
pub use net::network::{ensure_corpus_bundle, ensure_cost_matrix, ensure_file};
pub use net::sync::{bootstrap_essentials, generate_manifest, run_sync, ServerManifest, SyncStats};
pub use net::distributed::DistributedCoordinator; 

pub use asset::fs_provider::FsProvider;
pub use asset::manager::AssetManager;
pub use asset::caching_provider::CachingProvider;
pub use asset::ValkeyProvider; // ADDED

pub use keyforge_core::loader::{AssetLoader, RawCostData};

pub use util::common::{
    calculate_file_hash, generate_cost_profile, load_keycode_registry, sanitize_filename,
};
pub use util::layout_parser::parse_layout_string_permissive_cached;

include!(concat!(env!("OUT_DIR"), "/build_info.rs"));

pub fn get_build_info() -> (&'static str, &'static str) {
    (GIT_HASH, BUILD_DATE)
}
