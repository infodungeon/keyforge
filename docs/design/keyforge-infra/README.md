# Design: KeyForge Infra

**Responsibility:** IO Adapters (Filesystem, Network, Database, Distributed Coordination).
**Tier:** 3 (The Adapter)

## 1. Asset Providers

The crate implements the `AssetLoader` trait from `keyforge-core` with multiple backends:

```mermaid
classDiagram
    class AssetLoader {
        <<trait>>
        +load~T: Asset~(id) Arc~T~
        +load_corpus(sources) Arc~Corpus~
    }

    class FsProvider {
        -root: PathBuf
        +new(root) FsProvider
        +get_corpus_hash(id) String
    }

    class CachingProvider~P: AssetLoader~ {
        -inner: P
        -cache: moka::Cache
        +new(inner) CachingProvider
    }

    class ValkeyProvider {
        -coordinator: ValkeyDistributedCoordinator
        +new(coordinator) ValkeyProvider
    }

    AssetLoader <|.. FsProvider
    AssetLoader <|.. CachingProvider
    AssetLoader <|.. ValkeyProvider
    CachingProvider --> AssetLoader : wraps
```

## 2. Asset Manager

The `AssetManager` orchestrates local + remote asset resolution:

```mermaid
sequenceDiagram
    participant App
    participant Manager as AssetManager
    participant FS as check_system_path / check_user_path
    participant Client as HiveClient

    App->>Manager: ensure_keyboard("corne")
    Manager->>FS: check_user_path("keyboards", "corne")
    alt Found
        FS-->>Manager: PathBuf
    else Not Found
        Manager->>FS: check_system_path("keyboards", "corne")
        alt Found
            FS-->>Manager: PathBuf
        else Not Found
            Manager->>Client: ensure_file(url, local_path)
            Client-->>Manager: Downloaded
            Manager-->>App: PathBuf
        end
    end
```

**Methods:**
- `ensure_keyboard(name)` - Ensure keyboard definition exists
- `ensure_cost_matrix(filename)` - Ensure cost matrix exists
- `ensure_corpus(corpus_id, expected_hash)` - Ensure corpus bundle exists
- `sync_job_assets(JobConfig)` - Sync all assets for a job

## 3. Distributed Coordinator

Trait for cluster-wide state management (heartbeats, manifests, pub/sub):

```mermaid
classDiagram
    class DistributedCoordinator {
        <<trait>>
        +get_bin(key) Option~Bytes~
        +set_bin(key, data)
        +scan_keys(pattern) Vec~String~
        +try_reserve_profile_update(cpu_signature) bool
        +update_heartbeat(node_id, telemetry)
        +get_heartbeat(node_id) Option~NodeTelemetry~
        +get_cluster_stats() (usize, f32)
        +publish_update(job_id, event)
        +set_manifest_entry(entry)
        +get_manifest_hash(asset_id) Option~String~
        +get_all_manifest_entries() HashMap
        +count_active_nodes() usize
        +check_and_set_nonce(node_id, nonce, ttl) bool
    }

    class ValkeyDistributedCoordinator {
        -client: fred::Client
        +connect(url) Result
    }

    DistributedCoordinator <|.. ValkeyDistributedCoordinator
```

## 4. Corpus Loading Pipeline

```mermaid
sequenceDiagram
    participant Client
    participant Cache as CachingProvider
    participant FS as FsProvider
    participant Disk

    Client->>Cache: load_corpus([Source("text/en_std")])
    
    Cache->>Cache: Check Memory (moka)
    alt Cache Hit
        Cache-->>Client: Arc(Corpus)
    else Cache Miss
        Cache->>FS: load_corpus(sources)
        
        loop For each segment [1grams, 2grams, 3grams, words]
            FS->>Disk: Read segment.mpk.zst
            Disk-->>FS: Compressed bytes
            FS->>FS: Decompress (Zstd) + Deserialize (MsgPack)
            FS->>FS: Merge into Corpus
        end

        FS->>FS: inject_synthetic_data (Enter, Backspace)
        FS->>FS: Sort N-grams (for binary search)
        FS->>FS: validate()
        
        FS-->>Cache: Corpus
        Cache->>Cache: Insert into moka cache
        Cache-->>Client: Arc(Corpus)
    end
```

## 5. Network Layer

### HiveClient

HTTP client for communicating with the control plane:

```rust
pub struct HiveClient {
    pub fn new(base_url, api_key) -> Self;
    pub fn asset_url(&self, path) -> String;
    // ... HTTP methods
}
```

### Sync Functions

| Function | Purpose |
|----------|---------|
| `bootstrap_essentials()` | Download minimum required assets |
| `generate_manifest()` | Generate local asset manifest |
| `run_sync()` | Full sync with remote server |
| `ensure_file(client, url, local, hash)` | Download single file |
| `ensure_corpus_bundle(...)` | Download corpus segments |
| `ensure_cost_matrix(...)` | Download cost matrix |

## 6. Filesystem Layer

### WorkspaceLock

Process-level exclusive lock using `fs2`:

```rust
pub struct WorkspaceLock {
    pub fn acquire(path) -> InfraResult<Self>;  // Exponential backoff retry
}
// Automatically releases on drop
```

### Utilities

| Function | Purpose |
|----------|---------|
| `initialize_workspace(root, mode)` | Create directory structure |
| `atomic_write(path, data)` | Write-then-rename for crash safety |
| `read_to_string_limited(path, limit)` | Bounded file read |
| `resolve_root()` | Find workspace root |
| `calculate_file_hash(path)` | SHA-256 hash |

### InitMode

```rust
pub enum InitMode {
    Create,    // Create if missing
    Verify,    // Check structure only
}
```

## 7. Error Handling

```mermaid
classDiagram
    class InfraError {
        <<enum>>
        Io(io::Error)
        Network(reqwest::Error)
        Serde(serde_json::Error)
        HashMismatch(expected, actual)
        LockError(String)
        Config(String)
        Validation(String)
    }
```

## 8. Module Structure

```
keyforge-infra/src/
├── asset/
│   ├── cache.rs           # Cache key generation
│   ├── caching_provider.rs # CachingProvider (moka)
│   ├── fs_provider.rs     # FsProvider (filesystem)
│   ├── manager.rs         # AssetManager
│   ├── resolver.rs        # Path resolution logic
│   └── valkey_provider.rs # ValkeyProvider
├── fs/
│   ├── init.rs            # initialize_workspace, InitMode
│   ├── io.rs              # atomic_write, read_to_string_limited
│   ├── listing.rs         # Directory listing utilities
│   ├── lock.rs            # WorkspaceLock
│   └── paths.rs           # resolve_root
├── net/
│   ├── client.rs          # HiveClient
│   ├── distributed.rs     # DistributedCoordinator, ValkeyDistributedCoordinator
│   ├── network.rs         # ensure_file, ensure_corpus_bundle
│   └── sync.rs            # bootstrap_essentials, run_sync, ServerManifest
├── util/
│   └── common.rs          # calculate_file_hash, sanitize_filename, etc.
├── config.rs              # Configuration structures
├── error.rs               # InfraError, InfraResult
└── lib.rs                 # Re-exports, get_build_info()
```

## 9. Build Info

Compile-time injected metadata:

```rust
pub fn get_build_info() -> (&'static str, &'static str) {
    (GIT_HASH, BUILD_DATE)
}
```
