# Design: KeyForge Persistence

**Responsibility:** Project state management, user data storage, and request compilation.
**Tier:** 3 (The Adapter)

## 1. The Compiler Pattern

We distinguish between the **Stored State** (`Project`, `Config`) and the **Executable State** (`EngineRequest`).

```mermaid
sequenceDiagram
    participant CLI
    participant Compiler as compile_request
    participant Loader as AssetLoader
    participant Adapter as keyforge-adapter

    CLI->>Compiler: compile_request(loader, config, kb_id, corpora_ids)
    
    Compiler->>Loader: load<KeyboardDefinition>(kb_id)
    Loader-->>Compiler: Arc<KeyboardDefinition>
    
    Compiler->>Loader: load_corpus(sources)
    Loader-->>Compiler: Arc<Corpus>
    
    Compiler->>Loader: load<KeycodeRegistry>("keycodes")
    Loader-->>Compiler: Arc<KeycodeRegistry>
    
    Compiler->>Loader: load<CostModel>("cost_matrix")
    Loader-->>Compiler: Arc<CostModel>
    
    Compiler->>Adapter: to_domain_keyboard(geometry)
    Adapter-->>Compiler: Keyboard
    
    Compiler->>Adapter: to_domain_rubric(weights)
    Adapter-->>Compiler: Rubric
    
    Compiler->>Adapter: resolve_constraints(...)
    Adapter-->>Compiler: pinned_keys
    
    Compiler-->>CLI: EngineRequest
```

## 2. Project Definition

A persistable definition of an optimization experiment:

```mermaid
classDiagram
    class Project {
        +ProjectMeta meta
        +String keyboard
        +Vec~CorpusSource~ corpora
        +ScoringWeights weights
        +SearchParams params
        +Vec~KeyConstraint~ constraints
        +CostMatrixSource cost_matrix
        +Option~u64~ seed
    }

    class ProjectMeta {
        +String name
        +String version
        +String author
    }

    Project *-- ProjectMeta
```

**Format:** JSON-serializable. Contains paths/IDs ("corne", "text/en_std") and settings.

## 3. User Repository

`UserRepo` manages user-specific persistent data on the local filesystem:

```mermaid
classDiagram
    class UserRepo {
        -root: PathBuf
        +new(root) UserRepo
        +save_layout(kb_id, name, layout)
        +delete_layout(kb_id, name)
        +get_layouts(kb_id) HashMap
        +record_biometrics(samples) String
        +get_biometrics() Vec~BiometricSample~
        +reset_biometrics()
        +generate_profile() String
        +save_keyboard_definition(filename, def)
    }

    class UserLayoutStore {
        +layouts: HashMap~String, HashMap~String, String~~
    }

    UserRepo --> UserLayoutStore : manages
```

### Biometrics Storage

- **Format:** JSONL (newline-delimited JSON) at `user/user_stats.jsonl`
- **Locking:** Exclusive file lock during append
- **Streaming:** `load_stats_streaming()` for memory-efficient processing
- **Profile Generation:** Requires `MIN_BIOMETRIC_SAMPLES` before generating `personal_cost.json`

## 4. Autosave Service

Background service for debounced session persistence:

```mermaid
classDiagram
    class AutoSaveService {
        -path: PathBuf
        -state: Arc~Mutex~AutoSaveState~~
        +new(root_path) AutoSaveService
        +load() Option~SessionSnapshot~
        +schedule_save(snapshot)
        +flush(force)
    }

    class AutoSaveState {
        +pending: Option~SessionSnapshot~
        +last_save: Instant
    }

    class SessionSnapshot {
        +keyboard: String
        +layout_name: String
        +layout_string: String
        +corpus: String
        +cost_matrix: String
        +timestamp: u64
    }

    class PersistedSession {
        +snapshot: SessionSnapshot
        +checksum: String
        +verify() bool
    }

    AutoSaveService --> AutoSaveState
    AutoSaveService --> SessionSnapshot
    PersistedSession --> SessionSnapshot
```

### Debounce Strategy

- **Interval:** 2 seconds minimum between saves
- **Atomic Writes:** Uses `tempfile` + rename
- **Checksum:** SHA-256 for integrity verification
- **Legacy Support:** Can load old format without checksum

## 5. Error Handling

```mermaid
classDiagram
    class PersistenceError {
        <<enum>>
        Io(io::Error)
        Serde(serde_json::Error)
        ProjectNotFound(String)
        InvalidState(String)
        AssetLoad(String)
        Validation(String)
        Adapter(String)
    }
```

## 6. Module Structure

```
keyforge-persistence/src/
├── repo/
│   ├── mod.rs
│   └── user_repo.rs      # UserRepo, UserLayoutStore
├── store/
│   ├── mod.rs
│   └── autosave.rs       # AutoSaveService, SessionSnapshot
├── compiler.rs           # compile_request()
├── error.rs              # PersistenceError
├── project.rs            # Project, ProjectMeta
└── lib.rs                # Re-exports
```

## 7. Data Files

| Path | Format | Purpose |
|------|--------|---------|
| `user/user_layouts.json` | JSON | Saved layouts per keyboard |
| `user/user_stats.jsonl` | JSONL | Biometric samples log |
| `user/personal_cost.json` | JSON | Generated personalized cost profile |
| `user/keyboards/*.json` | JSON | Custom keyboard definitions |
| `session.json` | JSON | Autosaved session snapshot |
