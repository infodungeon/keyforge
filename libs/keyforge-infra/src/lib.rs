pub mod asset;
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
// Re-export from model now
pub use keyforge_model::loader::{AssetLoader, RawCostData};

pub use util::common::{
    calculate_file_hash, generate_cost_matrix_from_stats, load_keycode_registry, sanitize_filename,
};
pub use util::layout_parser::parse_layout_string_permissive_cached;
pub mod repo;
pub use repo::user_repo::UserRepo;
